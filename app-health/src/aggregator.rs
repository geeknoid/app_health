use crate::mermaid;
use crate::{Publisher, Report, Filter, Tracker};

#[derive(Debug, Clone)]
pub struct Aggregator;

impl Aggregator {
    #[must_use]
    pub fn new() -> Self {
        Self { }
    }

    #[must_use]
    pub fn is_health_nominal(&self) -> bool {
        todo!()
    }

    #[must_use]
    pub fn is_health_irrecoverable(&self) -> bool {
        todo!()
    }

    #[must_use]
    pub fn report(&self, filter: Filter) -> Report {
        todo!()
    }

    pub fn mute(&mut self, publisher: impl AsRef<str>) {
        todo!()
    }

    pub fn unmute(&mut self, publisher: impl AsRef<str>) {
        todo!()
    }

    pub fn track(&self, trigger: impl FnMut(bool)) -> Tracker {
        todo!()
    }

    pub fn publisher(&self, name: impl AsRef<str>) -> Publisher {
        todo!()
    }
}
