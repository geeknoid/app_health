use crate::Health;
use crate::aggregator::HealthMessage;
use tokio::sync::mpsc::UnboundedSender;

pub type PublisherId = usize;

#[derive(Debug)]
pub struct Publisher {
    id: PublisherId,
    tx: UnboundedSender<HealthMessage>,
    last: Health,
}

impl Publisher {
    pub(crate) fn new(id: PublisherId, tx: UnboundedSender<HealthMessage>) -> Self {
        // publisher startup message
        let _ = tx.send(HealthMessage {
            id,
            prev: None,
            next: Some(Health::Nominal),
        });

        Self {
            id,
            tx,
            last: Health::Nominal,
        }
    }

    pub fn publish(&mut self, health: Health) {
        if health != self.last {
            let prev = core::mem::replace(&mut self.last, health);

            // publish health change message
            let _ = self.tx.send(HealthMessage {
                id: self.id,
                prev: Some(prev),
                next: Some(self.last.clone()),
            });
        }
    }
}

impl Clone for Publisher {
    fn clone(&self) -> Self {
        
        // note: clones all start in the healthy state
        Self::new(self.id, self.tx.clone())
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let prev = self.last.clone();

        // publisher shutdown message
        let _ = self.tx.send(HealthMessage {
            id: self.id,
            prev: Some(prev),
            next: None,
        });
    }
}
