use crate::Attribute;
use crate::Health;
use crate::component::ComponentMessage;
use crate::signal::Signal;
use core::mem::replace;
use tokio::sync::mpsc;

/// A publisher represents a single source of health information for a component,
///
/// Each component has a number of publishers which each produce a signal and collectively this set of signals
/// determines the overall health of the component. A component can have any number of associated publishers,
/// but a typical use would be to have a single publisher per thread within the component.
///
/// Publishers are created via the [`publisher`](crate::Component::publisher) method, or through cloning.
/// Over time, you use functions such as [`nominal`](Self::nominal) and [`degraded`](Self::degraded) to
/// update the publisher's signal.
///
/// Updating a publisher's signal involves providing an optional set of attributes. Attributes are name/value pairs
/// that provide additional context about the health state. For example, if a publisher is reporting a degraded
/// state, it might include an attribute with a "reason" name and a value that describes the reason for the degradation,
/// such as "high latency" or "temporary network issue". Attributes can also include things like the specific endpoint that's
/// causing a problem, or any other metadata useful during diagnostics. The attributes are collected and made available in health
/// reports, aiding in troubleshooting and understanding the health of the component and application.
///
/// # Example
///
/// ```no_run
/// use app_health::{Aggregator, Component, Health, Publisher, Attribute};
/// use tokio::time::{sleep, Duration};
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     let mut aggregator = Aggregator::new();
///     let mut component = aggregator.component("redis_component");
///     let mut publisher = component.publisher();
///
///     tokio::spawn(async move {
///         loop {
///             // Simulate some work
///             sleep(Duration::from_secs(5)).await;
///
///             // Update the publisher's signal to indicate a problem has been found
///             publisher.publish(Health::Degraded, [("reason", "temporary issue")]);
///
///             // Simulate some work
///             sleep(Duration::from_secs(5)).await;
///
///            // Update the publisher's signal to indicate the problem has been resolved
///             publisher.publish(Health::Nominal, Vec::<Attribute>::new());
///         }
///     });
///
///     // Monitor the component's health state over time
///     loop {
///        sleep(Duration::from_secs(1)).await;
///        println!("Current health: {}", component.state());
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Publisher {
    signal: Signal,
    component_tx: mpsc::WeakUnboundedSender<ComponentMessage>,
}

impl Publisher {
    /// Creates a new component.
    #[must_use]
    pub(crate) fn new(component_tx: mpsc::WeakUnboundedSender<ComponentMessage>) -> Self {
        // if initial registration fails, it means the component is somehow gone already
        // this implies that any attempt for this publisher to publish will fail, and that's OK
        if let Some(channel) = component_tx.upgrade() {
            let _ = channel.send(ComponentMessage::StartPublishing(Signal::nominal()));
        }

        Self {
            signal: Signal::nominal(),
            component_tx,
        }
    }

    /// Get the publisher's current signal.
    #[must_use]
    pub const fn signal(&self) -> &Signal {
        &self.signal
    }

    /// Set the publisher's signal.
    pub fn publish(&mut self, state: Health, attributes: impl IntoIterator<Item = impl Into<Attribute>>) {
        self.change_signal(Signal::new(state, attributes));
    }

    /// Send signal changes to the background worker.
    fn change_signal(&mut self, new_signal: Signal) {
        if new_signal != self.signal {
            let old_signal = replace(&mut self.signal, new_signal);

            // communicate the signal change to the component, but we don't care if it fails since it means the component is dead already
            if let Some(channel) = self.component_tx.upgrade() {
                let _ = channel.send(ComponentMessage::ChangeHealth(old_signal, self.signal.clone()));
            }
        }
    }
}

impl Clone for Publisher {
    /// Create a new publisher that starts in the [`Nominal`](Health::Nominal) state.
    fn clone(&self) -> Self {
        Self::new(self.component_tx.clone())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        // try to tell the component about our demise, but we don't care if it fails since it means the component is dead already
        if let Some(channel) = self.component_tx.upgrade() {
            let _ = channel.send(ComponentMessage::StopPublishing(replace(&mut self.signal, Signal::nominal())));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_clone() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let weak_tx = tx.downgrade();

        let publisher1 = Publisher::new(weak_tx);
        let publisher2 = publisher1.clone();

        assert_eq!(publisher1.signal().state(), Health::Nominal);
        assert_eq!(publisher2.signal().state(), Health::Nominal);

        // Change the state of publisher1
        let mut publisher1 = publisher1;
        publisher1.publish(Health::Degraded, [("reason", "temporary issue"), ("code", "123")]);

        assert_eq!(publisher1.signal().state(), Health::Degraded);
        assert_eq!(publisher2.signal().state(), Health::Nominal);
    }
}
