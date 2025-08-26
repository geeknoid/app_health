use crate::component::{Component, ComponentId, ComponentName};
use crate::tracker::{TrackerCallback, TrackerId};
use crate::tracker_manager::TrackerManager;
use crate::{ComponentReport, Filter, HealthState, OverallReport, PublisherMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Represents the health state of a publisher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublisherHealth {
    Nominal,
    Degraded(PublisherMessage),
    Critical(PublisherMessage),
    Down(PublisherMessage),
    Unrecoverable(PublisherMessage),
}

/// A request for the worker to do stuff...
pub enum ControlRequest {
    StartPublishing(ComponentId, PublisherHealth),
    ChangeHealth(ComponentId, PublisherHealth, PublisherHealth),
    StopPublishing(ComponentId, PublisherHealth),
    RegisterTracker(TrackerId, Box<dyn TrackerCallback>),
    UnregisterTracker(TrackerId),
}

/// A request for a health report sent by the aggregator to the background worker.
pub enum ReportRequest {
    Component(
        ComponentId,
        Filter,
        tokio::sync::oneshot::Sender<ComponentReport>,
    ),
    Overall(Filter, tokio::sync::oneshot::Sender<OverallReport>),
}

/// The background worker that processes health updates and requests for reports.
pub async fn run_background_worker(
    components_by_name: Arc<Mutex<HashMap<ComponentName, ComponentId>>>,
    mut control_rx: mpsc::UnboundedReceiver<ControlRequest>,
    mut report_rx: mpsc::Receiver<ReportRequest>,
    mut shutdown_rx: mpsc::Receiver<()>,
) {
    let mut components = HashMap::new();
    let mut trackers = TrackerManager::new();
    let mut change_since_last_tick = false;
    let mut last_health_state = HealthState::Nominal;

    // TODO: make the wakeup interval configurable
    let mut ticker = tokio::time::interval(core::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            Some(req) = control_rx.recv() => {
                match req {
                    ControlRequest::StartPublishing(id, health) => {
                        change_since_last_tick = true;
                        with_component(id, &components_by_name, &mut components, |component| {
                            component.add_publisher_health(health);
                        }).await;
                    }

                    ControlRequest::ChangeHealth(id, old_health, new_health) => {
                        change_since_last_tick = true;
                        with_component(id, &components_by_name, &mut components, |component| {
                            component.add_publisher_health(old_health);
                            component.remove_publisher_health(new_health);
                        }).await;
                    }

                    ControlRequest::StopPublishing(id, old_health) => {
                        change_since_last_tick = true;
                        with_component(id, &components_by_name, &mut components, |component| {
                            component.remove_publisher_health(old_health);
                        }).await;
                    }

                    ControlRequest::RegisterTracker(id, callback) => {
                        trackers.add_tracker(id, callback);
                    }

                    ControlRequest::UnregisterTracker(id) => {
                        trackers.remove_tracker(id);
                    }
                }
            }

            Some(req) = report_rx.recv() => {
                match req {
                    ReportRequest::Component(id, filter, response_tx) => {
                        with_component(id, &components_by_name, &mut components, |component| {
                            _ = response_tx.send(component.make_report(filter));
                        }).await;
                    }

                    ReportRequest::Overall(filter, response_tx) => {
                        let mut report = OverallReport::default();
                        for component in components.values_mut() {
                            report.add_component_report(component.make_report(filter));
                        }

                        _ = response_tx.send(report);
                    }
                }
            }

            _ = ticker.tick() => {
                if !change_since_last_tick {
                    continue;
                }

                change_since_last_tick = false;

                let mut state = HealthState::Nominal;
                for component in components.values_mut() {
                    let comp_state = component.state();
                    if comp_state > state {
                        state = comp_state;
                    }
                }

                if state != last_health_state {
                    last_health_state = state;
                    trackers.call_trackers(state).await;
                }
            }

            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
}

async fn with_component(
    id: ComponentId,
    components_by_name: &Arc<Mutex<HashMap<ComponentName, ComponentId>>>,
    components: &mut HashMap<ComponentId, Component>,
    f: impl FnOnce(&mut Component),
) {
    if let Some(component) = components.get_mut(&id) {
        f(component);
    } else {
        sync_components(components_by_name, components).await;
        f(components.get_mut(&id).unwrap());
    }
}

async fn sync_components(
    components_by_name: &Arc<Mutex<HashMap<ComponentName, ComponentId>>>,
    components: &mut HashMap<ComponentId, Component>,
) {
    let components_by_name = components_by_name.lock().await;
    for (name, id) in components_by_name.iter() {
        if !components.contains_key(id) {
            _ = components.insert(*id, Component::new(name.clone()));
        }
    }
}
