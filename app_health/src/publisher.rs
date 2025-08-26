use crate::HealthState;
use crate::component::ComponentId;
use crate::worker::{ControlRequest, PublisherHealth};
use parking_lot::Mutex;
use std::borrow::Cow;
use std::mem::replace;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// A message provided by a publisher to describe its health state.
pub type PublisherMessage = Cow<'static, str>;

/// Lets a component report its health status.
///
/// Each component has got a number of publishers which each produce health updates that
/// are aggregated at the component level to determine the overall health of that component.
/// A component has many publishers, typically one per thread doing work for that component.
///
/// Publishers are created via the [`publisher`](crate::Aggregator::publisher) method, or through cloning.
#[derive(Debug)]
pub struct Publisher {
    id: ComponentId,
    tx: UnboundedSender<ControlRequest>,
    health: Arc<Mutex<PublisherHealth>>,
}

impl Publisher {
    pub(crate) fn new(id: ComponentId, tx: UnboundedSender<ControlRequest>) -> Self {
        let _ = tx.send(ControlRequest::StartPublishing(
            id,
            PublisherHealth::Nominal,
        ));

        Self {
            id,
            tx,
            health: Arc::new(Mutex::new(PublisherHealth::Nominal)),
        }
    }

    /// Get the current health state of the publisher.
    #[expect(
        clippy::missing_panics_doc,
        reason = "Mutex is never poisoned in normal operation"
    )]
    #[must_use]
    pub fn state(&self) -> HealthState {
        match *self.health.lock() {
            PublisherHealth::Nominal => HealthState::Nominal,
            PublisherHealth::Degraded(_) => HealthState::Degraded,
            PublisherHealth::Critical(_) => HealthState::Critical,
            PublisherHealth::Down(_) => HealthState::Down,
            PublisherHealth::Unrecoverable(_) => HealthState::Unrecoverable,
        }
    }

    /// Report that the publisher is in the [`Nominal`](crate::HealthState::Nominal) (healthy) state.
    pub fn nominal(&self) {
        self.change_health(PublisherHealth::Nominal);
    }

    /// Report that the publisher is in the [`Degraded`](crate::HealthState::Degraded) state, with a descriptive message.
    pub fn degraded(&self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Degraded(message.into()));
    }

    /// Report that the publisher is in the [`Critical`](crate::HealthState::Critical) state, with a descriptive message.
    pub fn critical(&self, message: impl Into<PublisherMessage>) {
        Self::change_health(self, PublisherHealth::Critical(message.into()));
    }

    /// Report that the publisher is in the [`Down`](crate::HealthState::Down) state, with a descriptive message.
    pub fn down(&self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Down(message.into()));
    }

    /// Report that the publisher is in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state, with a descriptive message.
    pub fn unrecoverable(&self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Unrecoverable(message.into()));
    }

    /// Send health changes to the background worker.
    fn change_health(&self, new_health: PublisherHealth) {
        let mut health = self.health.lock();
        if *health != new_health {
            let clone = new_health.clone();
            let old_health = replace(&mut *health, new_health);
            drop(health);

            let _ = self
                .tx
                .send(ControlRequest::ChangeHealth(self.id, old_health, clone));
        }
    }
}

impl Clone for Publisher {
    fn clone(&self) -> Self {
        // all clones start in the Nominal state
        Self::new(self.id, self.tx.clone())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let mut health = self.health.lock();
        let old_health = replace(&mut *health, PublisherHealth::Nominal);
        drop(health);

        let _ = self
            .tx
            .send(ControlRequest::StopPublishing(self.id, old_health));
    }
}
