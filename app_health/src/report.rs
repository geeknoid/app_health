use crate::{ComponentReport, HealthState};

/// A health report for the application as a whole.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Report {
    pub(crate) state: HealthState,
    pub(crate) nominal: usize,
    pub(crate) degraded: usize,
    pub(crate) critical: usize,
    pub(crate) down: usize,
    pub(crate) unrecoverable: usize,
    pub(crate) component_reports: Vec<ComponentReport>,
}

impl Report {
    /// The overall health of the application.
    ///
    /// This is determined by the most severe health state reported by any active publisher.
    #[must_use]
    pub const fn state(&self) -> HealthState {
        self.state
    }

    /// The number of publishers in the [`Nominal`](crate::HealthState::Nominal) state.
    #[must_use]
    pub const fn nominal_count(&self) -> usize {
        self.nominal
    }

    /// The number of publishers in the [`Degraded`](crate::HealthState::Degraded) state.
    #[must_use]
    pub const fn degraded_count(&self) -> usize {
        self.degraded
    }

    /// The number of publishers in the [`Critical`](crate::HealthState::Critical) state.
    #[must_use]
    pub const fn critical_count(&self) -> usize {
        self.critical
    }

    /// The number of publishers in the [`Down`](crate::HealthState::Down) state.
    #[must_use]
    pub const fn down_count(&self) -> usize {
        self.down
    }

    /// The number of publishers in the [`Unrecoverable`](crate::HealthState::Unrecoverable) state.
    #[must_use]
    pub const fn unrecoverable_count(&self) -> usize {
        self.unrecoverable
    }

    /// A map of categories to their respective reports.
    #[must_use]
    pub fn component_reports(&self) -> &[ComponentReport] {
        &self.component_reports
    }

    pub(crate) fn add_component_report(&mut self, report: ComponentReport) {
        self.nominal += report.nominal_count();
        self.degraded += report.degraded_count();
        self.critical += report.critical_count();
        self.down += report.down_count();
        self.unrecoverable += report.unrecoverable_count();
        self.component_reports.push(report);
    }
}
