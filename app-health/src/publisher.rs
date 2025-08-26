use crate::Health;

pub struct Publisher;

impl Publisher {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn publish(&self, health: Health) {
        todo!()
    }
}
