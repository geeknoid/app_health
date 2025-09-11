use crate::component::Component;
use crate::debouncer::Debouncer;
use crate::{ComponentMonitor, Filter, HealthState, Monitor, Report};
use core::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};

/// Aggregates health state from multiple components.
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
    let mut debouncer = Debouncer::new(debounce_delay);

    loop {
        let mut send_update = false;

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
                        send_update = debouncer.trigger();
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

            () = debouncer.ready() => {
                send_update = true;
            }
        }

        if send_update {
            let _ = health_tx.send(get_aggregate_health_state(&monitors));
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
