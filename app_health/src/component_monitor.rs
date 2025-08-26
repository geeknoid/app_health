use crate::component::ComponentMessage;
use crate::{ComponentReport, Filter, HealthState};
use tokio::sync::{mpsc, oneshot, watch};

/// Monitors to health of a component.
#[derive(Debug, Clone)]
pub struct ComponentMonitor {
    component_tx: mpsc::WeakUnboundedSender<ComponentMessage>,
    health_rx: watch::Receiver<HealthState>,
}

impl ComponentMonitor {
    pub(crate) const fn new(component_tx: mpsc::WeakUnboundedSender<ComponentMessage>, health_rx: watch::Receiver<HealthState>) -> Self {
        Self { component_tx, health_rx }
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
    pub fn state(&self) -> HealthState {
        *self.health_rx.borrow()
    }

    pub(crate) fn alive(&self) -> bool {
        self.component_tx.strong_count() > 0
    }

    /// Get a health report for the component.
    ///
    /// The filter parameter can be used to control which publisher messages are included in the report.
    ///
    /// This returns `None` if the associated component has been dropped.
    #[must_use]
    pub async fn report(&self, filter: Filter) -> Option<ComponentReport> {
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
