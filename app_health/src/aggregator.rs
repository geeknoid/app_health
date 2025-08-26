use crate::component::Component;
use crate::component_monitor::ComponentMonitor;
use crate::debouncer::Debouncer;
use crate::{Filter, Health, Reports};
use core::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};

/// Aggregates health state from multiple components.
#[derive(Debug)]
pub struct Aggregator {
    aggregator_tx: mpsc::UnboundedSender<AggregatorMessage>,
    health_rx: watch::Receiver<Health>,
}

/// Messages sent to the aggregator worker.
pub enum AggregatorMessage {
    ComponentCreated(ComponentMonitor),
    ComponentDropped,
    ComponentHealthChanged,
    GetReport(Filter, oneshot::Sender<Reports>),
}

// we only send state updates at most once per second (TODO: should come from config)
const MIN_DEBOUNCE_INTERVAL: Duration = Duration::from_secs(1);

impl Aggregator {
    /// Create a new health aggregator.
    #[must_use]
    pub fn new() -> Self {
        let (aggregator_tx, aggregator_rx) = mpsc::unbounded_channel();
        let (health_tx, health_rx) = watch::channel(Health::Nominal);

        drop(tokio::spawn(aggregator_worker(aggregator_rx, health_tx, MIN_DEBOUNCE_INTERVAL)));

        Self { aggregator_tx, health_rx }
    }

    /// Create a new component.
    pub fn component(&self, name: impl AsRef<str>) -> Component {
        Component::new(name, self.aggregator_tx.downgrade())
    }

    /// Track changes to the application's health state over time.
    ///
    /// This method's future will resolve when there has been a change to the overall health state of the application.
    /// Not every state change is actually reported. In particular, if the application's health state changes quickly,
    /// intermediate states may be skipped. In other words, quick health state changes are automatically debounced.
    ///
    /// # Errors
    ///
    /// The only reason why this future ever fails is when the aggregator has been dropped.
    pub async fn changed(&mut self) -> Result<(), ()> {
        self.health_rx.changed().await.map_err(|_e| ())
    }

    /// Get the overall health state of the application.
    ///
    /// The overall health is determined by the most severe health state reported by any component.
    #[must_use]
    pub fn state(&self) -> Health {
        *self.health_rx.borrow()
    }

    /// Get a health report for each component.
    ///
    /// The filter parameter can be used to control which publisher messages are included in the report.
    ///
    /// This returns `None` if the aggregator has been dropped.
    #[must_use]
    pub async fn reports(&self, filter: Filter) -> Option<Reports> {
        let (response_tx, response_rx) = oneshot::channel();
        let msg = AggregatorMessage::GetReport(filter, response_tx);
        if self.aggregator_tx.send(msg).is_ok() {
            return response_rx.await.ok();
        }

        None
    }
}

async fn aggregator_worker(
    mut aggregator_rx: mpsc::UnboundedReceiver<AggregatorMessage>,
    health_tx: watch::Sender<Health>,
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

                        let mut reports = Vec::with_capacity(monitors.len());
                        for monitor in &monitors {
                            if let Some(component_report) = monitor.report(filter).await {
                                reports.push(component_report);
                            }
                        }

                        // don't care if the receiver has gone away
                        let _ = response_tx.send(Reports::new(reports));
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

fn get_aggregate_health_state(monitors: &[ComponentMonitor]) -> Health {
    let mut state = Health::Nominal;
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
