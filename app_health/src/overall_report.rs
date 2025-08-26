use crate::{ComponentReport, HealthState};

/// A health report for the application as a whole.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OverallReport {
    pub(crate) nominal_count: usize,
    pub(crate) degraded_count: usize,
    pub(crate) critical_count: usize,
    pub(crate) down_count: usize,
    pub(crate) unrecoverable_count: usize,
    pub(crate) component_reports: Vec<ComponentReport>,
}

impl OverallReport {
    /// The overall health of the application.
    ///
    /// This is determined by the most severe health state reported by any active publisher.
    #[must_use]
    pub const fn state(&self) -> HealthState {
        if self.unrecoverable_count > 0 {
            HealthState::Unrecoverable
        } else if self.down_count > 0 {
            HealthState::Down
        } else if self.critical_count > 0 {
            HealthState::Critical
        } else if self.degraded_count > 0 {
            HealthState::Degraded
        } else {
            HealthState::Nominal
        }
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

    /// A map of categories to their respective reports.
    #[must_use]
    pub fn component_reports(&self) -> &[ComponentReport] {
        &self.component_reports
    }

    pub(crate) fn add_component_report(&mut self, report: ComponentReport) {
        self.nominal_count += report.nominal_count();
        self.degraded_count += report.degraded_count();
        self.critical_count += report.critical_count();
        self.down_count += report.down_count();
        self.unrecoverable_count += report.unrecoverable_count();
        self.component_reports.push(report);
    }
}
