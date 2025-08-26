use crate::component::Component;
use crate::{ComponentMonitor, Filter, HealthState, Monitor, Report};
use core::pin::Pin;
use core::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Instant, Sleep, sleep};

/// Aggregates health reports from multiple components.
///
/// Individual components each get publishers to report their state to the component, which then reports to the aggregator. You can query the application's health by
/// creating a monitor, which lets you poll the health or be notified whenever the health changes.
#[derive(Debug)]
pub struct Aggregator {
    aggregator_tx: mpsc::UnboundedSender<AggregatorMessage>,
    health_rx: watch::Receiver<HealthState>,
}

/// Messages sent to the aggregator worker.
pub enum AggregatorMessage {
    ComponentCreated(ComponentMonitor),
    ComponentDropped,
    ComponentHealthChanged,
    GetReport(Filter, oneshot::Sender<Report>),
}

// we only send state updates at most once per second (TODO: should come from config)
const MIN_DEBOUNCE_INTERVAL: Duration = Duration::from_secs(1);

impl Aggregator {
    /// Create a new health aggregator.
    #[must_use]
    pub fn new() -> Self {
        let (aggregator_tx, aggregator_rx) = mpsc::unbounded_channel();
        let (health_tx, health_rx) = watch::channel(HealthState::Nominal);

        drop(tokio::spawn(aggregator_worker(aggregator_rx, health_tx, MIN_DEBOUNCE_INTERVAL)));

        Self { aggregator_tx, health_rx }
    }

    /// Create a new monitor to track the application's health over time.
    #[must_use]
    pub fn monitor(&self) -> Monitor {
        Monitor::new(self.aggregator_tx.downgrade(), self.health_rx.clone())
    }

    /// Create a new component.
    pub fn component(&self, name: impl AsRef<str>) -> Component {
        Component::new(name, self.aggregator_tx.downgrade())
    }
}

async fn aggregator_worker(
    mut aggregator_rx: mpsc::UnboundedReceiver<AggregatorMessage>,
    health_tx: watch::Sender<HealthState>,
    debounce_delay: Duration,
) {
    let mut monitors = Vec::new();
    let mut last_health_update = Instant::now();
    let mut debounced_health_update_timer: Pin<Box<Sleep>> = Box::pin(sleep(Duration::from_secs(0)));
    let mut debounced_health_update_timer_active = false;

    loop {
        tokio::select! {
            msg = aggregator_rx.recv() => {
                match msg {
                    Some(AggregatorMessage::ComponentCreated(monitor)) => {
                        monitors.push(monitor);
                    }

                    Some(AggregatorMessage::GetReport(filter, response_tx)) => {
                        // clean up any monitors that are duds
                        monitors.retain(ComponentMonitor::alive);

                        let mut report = Report::default();
                        for monitor in &monitors {
                            if let Some(component_report) = monitor.report(filter).await {
                                report.add_component_report(component_report);
                            }
                        }

                        // don't care if the receiver has gone away
                        let _ = response_tx.send(report);
                    }

                    Some(AggregatorMessage::ComponentHealthChanged) => {
                        let now = Instant::now();
                        let elapsed = now.duration_since(last_health_update);
                        if elapsed >= debounce_delay {
                            // Immediate update; cancel any pending timer.
                            debounced_health_update_timer_active = false;
                            last_health_update = now;
                            let _ = health_tx.send(get_aggregate_health_state(&monitors));
                        } else if !debounced_health_update_timer_active {
                            // Schedule a deferred update
                            let delay = debounce_delay - elapsed;
                            debounced_health_update_timer.as_mut().reset(now + delay);
                            debounced_health_update_timer_active = true;
                        }
                    }

                    Some(AggregatorMessage::ComponentDropped) => {
                        // clean up any monitors that are duds
                        monitors.retain(ComponentMonitor::alive);
                    }

                    None => {
                        // all senders have been dropped, so we exit
                        return;
                    }
                }
            }

            () = debounced_health_update_timer.as_mut(), if debounced_health_update_timer_active => {
                debounced_health_update_timer_active = false;
                last_health_update = Instant::now();
                let _ = health_tx.send(get_aggregate_health_state(&monitors));
            }
        }
    }
}

fn get_aggregate_health_state(monitors: &[ComponentMonitor]) -> HealthState {
    let mut state = HealthState::Nominal;
    for monitor in monitors {
        let component_state = monitor.state();
        if component_state > state {
            state = component_state;
        }
    }
    state
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}
