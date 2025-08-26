use crate::HealthState;
use crate::tracker::{TrackerCallback, TrackerId};
use std::collections::HashMap;

pub struct TrackerManager {
    trackers: HashMap<TrackerId, Box<dyn TrackerCallback + Send + Sync>>,
}

impl TrackerManager {
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
        }
    }

    pub fn add_tracker(&mut self, id: TrackerId, callback: Box<dyn TrackerCallback>) {
        _ = self.trackers.insert(id, callback);
    }

    pub fn remove_tracker(&mut self, id: TrackerId) {
        _ = self.trackers.remove(&id);
    }

    pub async fn call_trackers(&self, state: HealthState) {
        for t in self.trackers.values() {
            t.on_state_change(state).await;
        }
    }
}
