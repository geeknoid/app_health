use crate::health::{ALL_HEALTH_STATES, NUM_HEALTH_STATES};
use crate::signal::Signal;
use crate::{Health, Signals};
use core::fmt::Display;
use std::sync::Arc;

/// The health of a single application component.
///
/// A component's health is determined by combining the data from the component's active publishers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Report {
    pub(crate) name: Arc<str>,
    pub(crate) state: Health,
    pub(crate) counts: [usize; NUM_HEALTH_STATES],
    pub(crate) signals: [Vec<(Signal, usize)>; NUM_HEALTH_STATES],
}

impl Report {
    /// The name of the component.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The overall health of the component.
    ///
    /// A component's health is determined by the most severe health state reported by any active publisher.
    #[must_use]
    pub const fn state(&self) -> Health {
        self.state
    }

    /// The number of active publisher signals in the given health state.
    #[must_use]
    pub const fn signal_count(&self, state: Health) -> usize {
        self.counts[state as usize]
    }

    /// The active publisher signals in the given health state, along with their counts.
    ///
    /// The count indicates how many publishers are reporting the same signal.
    #[must_use]
    pub fn signals(&self, state: Health) -> Signals<'_> {
        Signals::new(&self.signals[state as usize])
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Component {}: {}", self.name(), self.state())?;
        for state in ALL_HEALTH_STATES {
            let signals = self.signals(state);
            if signals.len() > 0 {
                writeln!(f, "  {state:?}")?;
                for (signal, count) in signals {
                    writeln!(f, "    {count} x {signal}")?;
                }
            }
        }

        Ok(())
    }
}
