use crate::aggregator::AggregatorMessage;
use crate::component_monitor::ComponentMonitor;
use crate::component_state::ComponentState;
use crate::debouncer::Debouncer;
use crate::signal::Signal;
use crate::{Filter, Health, Publisher, Report};
use core::time::Duration;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch};

/// A component responsible for tracking the health of an individual feature in an application.
#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names, reason = "field names are clear and unambiguous")]
pub struct Component {
    component_tx: mpsc::UnboundedSender<ComponentMessage>,
    health_rx: watch::Receiver<Health>,
    aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>,
}

/// Messages sent to the component worker.
pub enum ComponentMessage {
    StartPublishing(Signal),
    ChangeHealth(Signal, Signal),
    StopPublishing(Signal),
    GetReport(Filter, oneshot::Sender<Report>),
}

impl Component {
    pub(crate) fn new(name: impl AsRef<str>, aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>) -> Self {
        let (component_tx, component_rx) = mpsc::unbounded_channel::<ComponentMessage>();
        let (health_tx, health_rx) = watch::channel(Health::Nominal);
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
    pub(crate) fn monitor(&self) -> ComponentMonitor {
        ComponentMonitor::new(self.component_tx.downgrade(), self.health_rx.clone())
    }

    /// Track changes to the component's health state over time.
    ///
    /// This method's future will resolve when there has been a change to the overall health state of the component.
    /// Not every state change is actually reported. In particular, if the component's health state changes quickly,
    /// intermediate states may be skipped. In other words, quick health state changes are automatically debounced.
    ///
    /// # Errors
    ///
    /// The only reason why this future ever fails is when the associated component has been dropped.
    pub async fn changed(&mut self) -> Result<(), ()> {
        self.health_rx.changed().await.map_err(|_e| ())
    }

    /// Get the overall health state of the component.
    ///
    /// The overall health is determined by the most severe health state reported by any publisher.
    #[must_use]
    pub fn state(&self) -> Health {
        *self.health_rx.borrow()
    }

    /// Get a health report for the component.
    ///
    /// The filter parameter can be used to control which publisher signals are included in the report.
    ///
    /// This returns `None` if the associated component has been dropped.
    #[must_use]
    pub async fn report(&self, filter: Filter) -> Option<Report> {
        let (response_tx, response_rx) = oneshot::channel();
        let msg = ComponentMessage::GetReport(filter, response_tx);
        if self.component_tx.send(msg).is_ok() {
            return response_rx.await.ok();
        }

        None
    }
}

async fn component_worker(
    name: Arc<str>,
    mut component_rx: mpsc::UnboundedReceiver<ComponentMessage>,
    health_tx: watch::Sender<Health>,
    aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>,
) {
    let mut component_state = ComponentState::new(name);
    let mut health_state = Health::Nominal;
    let mut debouncer = Debouncer::new(Duration::from_millis(100));

    loop {
        let mut send_update = false;

        tokio::select! {
            msg = component_rx.recv() => {
                match msg {
                    Some(ComponentMessage::StartPublishing(health)) => {
                        component_state.add_publisher_signal(health);
                        send_update = debouncer.trigger();
                    }

                    Some(ComponentMessage::ChangeHealth(old_health, new_health)) => {
                        component_state.remove_publisher_signal(old_health);
                        component_state.add_publisher_signal(new_health);
                        send_update = debouncer.trigger();
                    }

                    Some(ComponentMessage::StopPublishing(health)) => {
                        component_state.remove_publisher_signal(health);
                        send_update = debouncer.trigger();
                    }

                    Some(ComponentMessage::GetReport(filter, response_tx)) => {
                        let report = component_state.make_report(filter);
                        let _ = response_tx.send(report);
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
            let new_state = component_state.state();

            // We don't send updates if the previous state was nominal and the new state is also nominal.
            // Any other transition is reported, since the publisher messages may have changed
            if new_state != health_state || new_state != Health::Nominal {
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
