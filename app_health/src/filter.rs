use crate::Health;
use bitflags::bitflags;

bitflags! {
    /// Controls the information collected in health reports.
    #[derive(Clone, Copy)]
    pub struct Filter: u32 {

        /// Include publisher signals with `Nominal` health state.
        const NOMINAL = 1 << Health::Nominal as u32;

        /// Include publisher signals with `Degraded` health state.
        const DEGRADED = 1 << Health::Degraded as u32;

        /// Include publisher signals with `Critical` health state.
        const CRITICAL = 1 << Health::Critical as u32;

        /// Include publisher signals with `Down` health state.
        const DOWN = 1 << Health::Down as u32;

        /// Include publisher signals with `Unrecoverable` health state.
        const UNRECOVERABLE = 1 << Health::Unrecoverable as u32;

        /// Include all publisher signals.
        const ALL = Self::NOMINAL.bits() | Self::DEGRADED.bits() | Self::CRITICAL.bits() | Self::DOWN.bits() | Self::UNRECOVERABLE.bits();
    }
}
