use crate::HealthState;
use crate::component::{ComponentMessage, PublisherHealth};
use core::mem::replace;
use std::borrow::Cow;
use tokio::sync::mpsc;

/// A message provided by a publisher to describe its health state.
pub type PublisherMessage = Cow<'static, str>;

/// Lets a component report its health status.
///
/// Each component has got a number of publishers which each produce health updates that
/// are aggregated at the component level to determine the overall health of that component.
/// A component has many publishers, typically one per thread doing work for that component.
///
/// Publishers are created via the [`publisher`](crate::Component::publisher) method, or through cloning.
#[derive(Debug)]
pub struct Publisher {
    health: PublisherHealth,
    component_tx: mpsc::WeakUnboundedSender<ComponentMessage>,
}

impl Publisher {
    /// Creates a new component.
    #[must_use]
    pub(crate) fn new(component_tx: mpsc::WeakUnboundedSender<ComponentMessage>) -> Self {
        // if initial registration fails, it means the component is somehow gone already
        // this implies that any attempt for this publisher to publish will fail, and that's OK
        if let Some(channel) = component_tx.upgrade() {
            let _ = channel.send(ComponentMessage::StartPublishing(PublisherHealth::Nominal));
        }

        Self {
            health: PublisherHealth::Nominal,
            component_tx,
        }
    }

    /// Get the publisher's current health state.
    #[must_use]
    pub fn state(&self) -> HealthState {
        HealthState::from(&self.health)
    }

    /// Report that the publisher is in the [`Nominal`](crate::HealthState::Nominal) (healthy) state.
    pub fn nominal(&mut self) {
        self.change_health(PublisherHealth::Nominal);
    }

    /// Report that the publisher is in the [`Degraded`](crate::HealthState::Degraded) state, with a descriptive message.
    pub fn degraded(&mut self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Degraded(message.into()));
    }

    /// Report that the publisher is in the [`Critical`](crate::HealthState::Critical) state, with a descriptive message.
    pub fn critical(&mut self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Critical(message.into()));
    }

    /// Report that the publisher is in the [`Down`](crate::HealthState::Down) state, with a descriptive message.
    pub fn down(&mut self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Down(message.into()));
    }

    /// Report that the publisher is in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state, with a descriptive message.
    pub fn unrecoverable(&mut self, message: impl Into<PublisherMessage>) {
        self.change_health(PublisherHealth::Unrecoverable(message.into()));
    }

    /// Send health changes to the background worker.
    fn change_health(&mut self, new_health: PublisherHealth) {
        if new_health != self.health {
            let old_health = replace(&mut self.health, new_health);

            // communicate the state change to the component, but we don't care if it fails since it means the component is dead already
            if let Some(channel) = self.component_tx.upgrade() {
                let _ = channel.send(ComponentMessage::ChangeHealth(old_health, self.health.clone()));
            }
        }
    }
}

impl Clone for Publisher {
    /// Create a new publisher that starts in the [`Nominal`](crate::HealthState::Nominal) state.
    fn clone(&self) -> Self {
        Self::new(self.component_tx.clone())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        // try to tell the component about our demise, but we don't care if it fails since it means the component is dead already
        if let Some(channel) = self.component_tx.upgrade() {
            let _ = channel.send(ComponentMessage::StopPublishing(replace(
                &mut self.health,
                PublisherHealth::Nominal,
            )));
        }
    }
}
