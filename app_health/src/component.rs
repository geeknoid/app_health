use crate::aggregator::AggregatorMessage;
use crate::component_state::ComponentState;
use crate::{ComponentMonitor, ComponentReport, Filter, HealthState, Publisher, PublisherMessage};
use tokio::sync::{mpsc, oneshot, watch};

/// A component responsible for tracking the health of an individual feature in an application.
#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names, reason = "field names are clear and unambiguous")]
pub struct Component {
    component_tx: mpsc::UnboundedSender<ComponentMessage>,
    health_rx: watch::Receiver<HealthState>,
    aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>,
}

/// Represents the health state of a publisher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublisherHealth {
    Nominal,
    Degraded(PublisherMessage),
    Critical(PublisherMessage),
    Down(PublisherMessage),
    Unrecoverable(PublisherMessage),
}

/// Messages sent to the component worker.
pub enum ComponentMessage {
    StartPublishing(PublisherHealth),
    ChangeHealth(PublisherHealth, PublisherHealth),
    StopPublishing(PublisherHealth),
    GetReport(Filter, oneshot::Sender<ComponentReport>),
}

impl Component {
    pub(crate) fn new(name: impl AsRef<str>, aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>) -> Self {
        let (component_tx, component_rx) = mpsc::unbounded_channel::<ComponentMessage>();
        let (health_tx, health_rx) = watch::channel(HealthState::Nominal);
        let name = name.as_ref().into();

        drop(tokio::spawn(component_worker(name, component_rx, health_tx, aggregator_tx.clone())));

        let result = Self {
            component_tx,
            health_rx,
            aggregator_tx,
        };

        let monitor = result.monitor();
        if let Some(channel) = result.aggregator_tx.upgrade() {
            let _ = channel.send(AggregatorMessage::ComponentCreated(monitor));
        }

        result
    }

    /// Create a publisher for this component.
    ///
    /// A publisher is how health information is injected into a component. A component's health
    /// is determined by the aggregate health of all its active publishers.
    #[must_use]
    pub fn publisher(&self) -> Publisher {
        Publisher::new(self.component_tx.downgrade())
    }

    /// Track changes to the component's health state over time.
    #[must_use]
    pub fn monitor(&self) -> ComponentMonitor {
        ComponentMonitor::new(self.component_tx.downgrade(), self.health_rx.clone())
    }
}

async fn component_worker(
    name: Box<str>,
    mut component_rx: mpsc::UnboundedReceiver<ComponentMessage>,
    health_tx: watch::Sender<HealthState>,
    aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>,
) {
    let mut component_state = ComponentState::new(name);
    let mut health_state = HealthState::Nominal;

    loop {
        let Some(mut update) = component_rx.recv().await else { return };
        let mut changes = false;

        loop {
            match update {
                ComponentMessage::StartPublishing(health) => {
                    component_state.add_publisher_health(health);
                    changes = true;
                }

                ComponentMessage::ChangeHealth(old_health, new_health) => {
                    component_state.remove_publisher_health(old_health);
                    component_state.add_publisher_health(new_health);
                    changes = true;
                }

                ComponentMessage::StopPublishing(health) => {
                    component_state.remove_publisher_health(health);
                    changes = true;
                }

                ComponentMessage::GetReport(filter, response_tx) => {
                    let report = component_state.make_report(filter);
                    let _ = response_tx.send(report);
                }
            }

            update = match component_rx.try_recv() {
                Ok(msg) => msg,
                _ => break,
            };
        }

        if changes {
            let new_state = component_state.state();

            // We don't send updates if the previous state was nominal and the new state is also nominal.
            // Any other transition is reported, since the publisher messages may have changed
            if new_state != health_state || new_state != HealthState::Nominal {
                health_state = new_state;
                let _ = health_tx.send(new_state);

                // it's OK if the aggregator is no longer there...
                if let Some(channel) = aggregator_tx.upgrade() {
                    let _ = channel.send(AggregatorMessage::ComponentHealthChanged);
                }
            }
        }
    }
}

impl Drop for Component {
    fn drop(&mut self) {
        // tell the aggregator we're going away, but we don't care if
        // the aggregator has gone away
        if let Some(channel) = self.aggregator_tx.upgrade() {
            let _ = channel.send(AggregatorMessage::ComponentDropped);
        }
    }
}
