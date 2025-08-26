/// Represents the health/severity of an individual component or of the entire process.
///
/// Ordering reflects severity from best to worst:
/// `Nominal < Degraded < Critical < Down < Unrecoverable`.
///
/// Typical aggregation picks the maximum (most severe) state among inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum HealthState {
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
