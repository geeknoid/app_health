use bitflags::bitflags;

bitflags! {
    /// Controls the information collected in health reports.
    #[derive(Clone, Copy)]
    pub struct Filter: u32 {

        /// Include publisher messages with `Degraded` health state.
        const DEGRADED = 0x01;

        /// Include publisher messages with `Critical` health state.
        const CRITICAL = 0x02;

        /// Include publisher messages with `Down` health state.
        const DOWN = 0x04;

        /// Include publisher messages with `Unrecoverable` health state.
        const UNRECOVERABLE = 0x08;

        /// Include all publisher messages.
        const ALL = Self::DEGRADED.bits() | Self::CRITICAL.bits() | Self::DOWN.bits() | Self::UNRECOVERABLE.bits();
    }
}
