use crate::aggregator::AggregatorMessage;
use crate::{Filter, HealthState, Report};
use tokio::sync::{mpsc, oneshot, watch};

/// Monitors the health of an application.
#[derive(Debug, Clone)]
pub struct Monitor {
    aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>,
    health_rx: watch::Receiver<HealthState>,
}

impl Monitor {
    pub(crate) const fn new(aggregator_tx: mpsc::WeakUnboundedSender<AggregatorMessage>, health_rx: watch::Receiver<HealthState>) -> Self {
        Self { aggregator_tx, health_rx }
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
    pub fn state(&self) -> HealthState {
        *self.health_rx.borrow()
    }

    /// Get a health report for the application.
    ///
    /// This report includes the overall health state of the application, as well as detailed reports for each component.
    /// The filter parameter can be used to control which publisher messages are included in the report.
    ///
    /// This returns `None` if the aggregator has been dropped.
    #[must_use]
    pub async fn report(&self, filter: Filter) -> Option<Report> {
        let (response_tx, response_rx) = oneshot::channel();
        let msg = AggregatorMessage::GetReport(filter, response_tx);
        if let Some(channel) = self.aggregator_tx.upgrade()
            && channel.send(msg).is_ok()
        {
            return response_rx.await.ok();
        }

        None
    }
}
