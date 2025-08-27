use crate::Entries;

pub struct Report;

impl Report {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {}
    }

    #[must_use]
    pub fn is_health_nominal(&self) -> bool {
        todo!()
    }

    #[must_use]
    pub fn entries(&self) -> Entries {
        todo!()
    }
}
