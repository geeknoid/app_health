use core::fmt::Display;

/// Represents the health/severity of an individual component or of the entire process.
///
/// Ordering reflects severity from best to worst:
/// `Nominal < Degraded < Critical < Down < Unrecoverable`.
///
/// Typical aggregation picks the maximum (most severe) state among inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Health {
    /// Everything is functioning as expected.
    ///
    /// - No errors detected.
    /// - Performance, capacity, and dependencies are within normal ranges.
    /// - Suitable for green/OK dashboards.
    #[default]
    Nominal,

    /// Functionality is available but with minor impairment or risk.
    ///
    /// - Degraded performance, reduced capacity, retries, or non-critical warnings.
    /// - User impact is limited or intermittent; SLAs are still largely met.
    /// - Requires attention but not immediate incident response.
    Degraded,

    /// Severe impairment that materially affects functionality.
    ///
    /// - Critical paths are failing or significantly unreliable.
    /// - Users are impacted; SLAs are being violated.
    /// - Immediate investigation and mitigation are required.
    Critical,

    /// Service or component is effectively down/unavailable, but recoverable.
    ///
    /// - No meaningful functionality is available to users.
    /// - Recovery is expected via operational actions (e.g., restart, failover, roll-back).
    /// - No permanent data loss is implied by this state alone.
    Down,

    /// Unrecoverable failure requiring deeper intervention beyond routine recovery.
    ///
    /// - Indicates corruption, irreversible configuration issues, or permanent data loss.
    /// - Automated/self-service recovery is not expected to succeed.
    /// - Requires extensive remediation, reboot, or rebuild.
    Unrecoverable,
}

pub const NUM_HEALTH_STATES: usize = 5;

pub const ALL_HEALTH_STATES: [Health; NUM_HEALTH_STATES] = [
    Health::Nominal,
    Health::Degraded,
    Health::Critical,
    Health::Down,
    Health::Unrecoverable,
];

impl Display for Health {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Nominal => "Nominal",
            Self::Degraded => "Degraded",
            Self::Critical => "Critical",
            Self::Down => "Down",
            Self::Unrecoverable => "Unrecoverable",
        };

        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Health;

    #[test]
    fn display_strings() {
        assert_eq!(Health::Nominal.to_string(), "Nominal");
        assert_eq!(Health::Degraded.to_string(), "Degraded");
        assert_eq!(Health::Critical.to_string(), "Critical");
        assert_eq!(Health::Down.to_string(), "Down");
        assert_eq!(Health::Unrecoverable.to_string(), "Unrecoverable");
    }
}
