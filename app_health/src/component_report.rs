use crate::{HealthState, Messages, PublisherMessage};

/// The health of a single application component.
///
/// A component's health is determined by combining the data from the component's active publishers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentReport {
    pub(crate) name: String,
    pub(crate) state: HealthState,
    pub(crate) nominal_count: usize,
    pub(crate) degraded_count: usize,
    pub(crate) critical_count: usize,
    pub(crate) down_count: usize,
    pub(crate) unrecoverable_count: usize,
    pub(crate) degraded_messages: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) critical_messages: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) down_messages: Option<Vec<(PublisherMessage, usize)>>,
    pub(crate) unrecoverable_messages: Option<Vec<(PublisherMessage, usize)>>,
}

impl ComponentReport {
    /// The name of the component.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The overall health of the component.
    ///
    /// A component's health is determined by the most severe health state reported by any active publishers.
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
        self.degraded_messages.as_ref().map(|v| Messages::new(v))
    }

    /// Get the messages associated with publishers in the [`Critical`](crate::HealthState::Critical) state.
    ///
    /// This returns [`None`] if [`Filter::CRITICAL`](crate::Filter::CRITICAL) was not specified when requesting the report.
    #[must_use]
    pub fn critical(&self) -> Option<Messages<'_>> {
        self.critical_messages.as_ref().map(|v| Messages::new(v))
    }

    /// Get the messages associated with publishers in the [`Down`](crate::HealthState::Down) state.
    ///
    /// This returns [`None`] if [`Filter::DOWN`](crate::Filter::DOWN) was not specified when requesting the report.
    #[must_use]
    pub fn down(&self) -> Option<Messages<'_>> {
        self.down_messages.as_ref().map(|v| Messages::new(v))
    }

    /// Get the messages associated with publishers in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state.
    ///
    /// This returns [`None`] if [`Filter::UNRECOVERABLE`](crate::Filter::UNRECOVERABLE) was not specified when requesting the report.
    #[must_use]
    pub fn unrecoverable(&self) -> Option<Messages<'_>> {
        self.unrecoverable_messages.as_ref().map(|v| Messages::new(v))
    }
}
