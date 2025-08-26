use crate::HealthState;
use crate::worker::ControlRequest;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;

/// A unique identifier for a tracker callback.
pub type TrackerId = usize;

#[async_trait]
pub trait TrackerCallback: Send + Sync {
    async fn on_state_change(&self, state: HealthState);
}

/// A handle to a registered tracker callback.
///
/// When this handle is dropped, the tracker callback is unregistered and is no longer invoked.
#[derive(Debug)]
pub struct Tracker {
    id: TrackerId,
    tx: UnboundedSender<ControlRequest>,
}

impl Tracker {
    pub(crate) const fn new(id: TrackerId, tx: UnboundedSender<ControlRequest>) -> Self {
        Self { id, tx }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        let _ = self.tx.send(ControlRequest::UnregisterTracker(self.id));
    }
}
