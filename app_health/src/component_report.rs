use crate::component::ComponentName;
use crate::{HealthState, Messages, PublisherMessage};

/// The health of a single application component.
///
/// This is determined by  combining the data from the component's active publishers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentReport {
    pub(crate) name: ComponentName,
    pub(crate) state: HealthState,
    pub(crate) nominal_count: usize,
    pub(crate) degraded_count: usize,
    pub(crate) critical_count: usize,
    pub(crate) down_count: usize,
    pub(crate) unrecoverable_count: usize,
    pub(crate) degraded: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) critical: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) down: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) unrecoverable: Option<Vec<(PublisherMessage, usize)>>,
}

impl ComponentReport {
    /// The name of the component.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// The overall health of the component.
    ///
    /// This is determined by the most severe health state reported by any active publishers.
    #[must_use]
    pub const fn state(&self) -> HealthState {
        self.state
    }

    /// The number of publishers in the [`Nominal`](crate::HealthState::Nominal) state.
    #[must_use]
    pub const fn nominal_count(&self) -> usize {
        self.nominal_count
    }

    /// The number of publishers in the [`Degraded`](crate::HealthState::Degraded) state.
    #[must_use]
    pub const fn degraded_count(&self) -> usize {
        self.degraded_count
    }

    /// The number of publishers in the [`Critical`](crate::HealthState::Critical) state.
    #[must_use]
    pub const fn critical_count(&self) -> usize {
        self.critical_count
    }

    /// The number of publishers in the [`Down`](crate::HealthState::Down) state.
    #[must_use]
    pub const fn down_count(&self) -> usize {
        self.down_count
    }

    /// The number of publishers in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state.
    #[must_use]
    pub const fn unrecoverable_count(&self) -> usize {
        self.unrecoverable_count
    }

    /// Get the messages associated with publishers in the [`Degraded`](crate::HealthState::Degraded) state.
    ///
    /// This returns [`None`] if [`Filter::DEGRADED`](crate::Filter::DEGRADED) was not specified when requesting the report.
    #[must_use]
    pub fn degraded(&self) -> Option<Messages<'_>> {
        Some(Messages::new(self.degraded.as_ref()?))
    }

    /// Get the messages associated with publishers in the [`Critical`](crate::HealthState::Critical) state.
    ///
    /// This returns [`None`] if [`Filter::CRITICAL`](crate::Filter::CRITICAL) was not specified when requesting the report.
    #[must_use]
    pub fn critical(&self) -> Option<Messages<'_>> {
        Some(Messages::new(self.critical.as_ref()?))
    }

    /// Get the messages associated with publishers in the [`Down`](crate::HealthState::Down) state.
    ///
    /// This returns [`None`] if [`Filter::DOWN`](crate::Filter::DOWN) was not specified when requesting the report.
    #[must_use]
    pub fn down(&self) -> Option<Messages<'_>> {
        Some(Messages::new(self.down.as_ref()?))
    }

    /// Get the messages associated with publishers in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state.
    ///
    /// This returns [`None`] if [`Filter::UNRECOVERABLE`](crate::Filter::UNRECOVERABLE) was not specified when requesting the report.
    #[must_use]
    pub fn unrecoverable(&self) -> Option<Messages<'_>> {
        Some(Messages::new(self.unrecoverable.as_ref()?))
    }
}
