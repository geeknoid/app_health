use crate::component::ComponentMessage;
use crate::{Filter, Health, Report};
use tokio::sync::{mpsc, oneshot, watch};

/// Monitors the health of a component.
///
/// This is used as a form of weak reference to the component, held by the aggregator.
/// It can be used to query the component's health state and request health reports.
#[derive(Debug)]
pub struct ComponentMonitor {
    component_tx: mpsc::WeakUnboundedSender<ComponentMessage>,
    health_rx: watch::Receiver<Health>,
}

impl ComponentMonitor {
    pub const fn new(component_tx: mpsc::WeakUnboundedSender<ComponentMessage>, health_rx: watch::Receiver<Health>) -> Self {
        Self { component_tx, health_rx }
    }

    /// Get the overall health state of the component.
    ///
    /// The overall health is determined by the most severe health state reported by any publisher.
    #[must_use]
    pub fn state(&self) -> Health {
        *self.health_rx.borrow()
    }

    pub fn alive(&self) -> bool {
        self.component_tx.strong_count() > 0
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
        if let Some(channel) = self.component_tx.upgrade()
            && channel.send(msg).is_ok()
        {
            return response_rx.await.ok();
        }

        None
    }
}
