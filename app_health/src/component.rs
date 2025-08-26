use crate::worker::PublisherHealth;
use crate::{ComponentReport, Filter, HealthState, PublisherMessage};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// A component's name.
pub type ComponentName = Cow<'static, str>;

/// A component's unique identifier.
pub type ComponentId = usize;

/// A component's state, tracking the aggregate health of its publishers.
#[derive(Debug)]
pub struct Component {
    name: ComponentName,
    state: Option<HealthState>,
    nominal_count: usize,
    degraded_count: usize,
    critical_count: usize,
    down_count: usize,
    unrecoverable_count: usize,
    degraded: HashMap<PublisherMessage, usize>,
    critical: HashMap<PublisherMessage, usize>,
    down: HashMap<PublisherMessage, usize>,
    unrecoverable: HashMap<PublisherMessage, usize>,
}

impl Component {
    pub fn new(name: ComponentName) -> Self {
        Self {
            name,
            state: None,
            nominal_count: 0,
            degraded_count: 0,
            critical_count: 0,
            down_count: 0,
            unrecoverable_count: 0,

            degraded: HashMap::new(),
            critical: HashMap::new(),
            down: HashMap::new(),
            unrecoverable: HashMap::new(),
        }
    }

    /// Incorporate a publisher's health into the component's aggregate state.
    pub fn add_publisher_health(&mut self, health: PublisherHealth) {
        self.state = None;

        match health {
            PublisherHealth::Nominal => {
                self.nominal_count += 1;
            }

            PublisherHealth::Degraded(message) => {
                self.degraded_count += 1;
                *self.degraded.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Critical(message) => {
                self.critical_count += 1;
                *self.critical.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Down(message) => {
                self.down_count += 1;
                *self.down.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Unrecoverable(message) => {
                self.unrecoverable_count += 1;
                *self.unrecoverable.entry(message).or_insert(0) += 1;
            }
        }
    }

    /// Remove a publisher's health from the component's aggregate state.
    pub fn remove_publisher_health(&mut self, health: PublisherHealth) {
        self.state = None;

        match health {
            PublisherHealth::Nominal => {
                self.nominal_count -= 1;
            }

            PublisherHealth::Degraded(message) => {
                self.degraded_count -= 1;
                dec_or_remove(&mut self.degraded, message);
            }

            PublisherHealth::Critical(message) => {
                self.critical_count -= 1;
                dec_or_remove(&mut self.critical, message);
            }

            PublisherHealth::Down(message) => {
                self.down_count -= 1;
                dec_or_remove(&mut self.down, message);
            }

            PublisherHealth::Unrecoverable(message) => {
                self.unrecoverable_count -= 1;
                dec_or_remove(&mut self.unrecoverable, message);
            }
        }
    }

    pub const fn state(&mut self) -> HealthState {
        if let Some(state) = self.state {
            return state;
        }

        let state = if self.unrecoverable_count > 0 {
            HealthState::Unrecoverable
        } else if self.down_count > 0 {
            HealthState::Down
        } else if self.critical_count > 0 {
            HealthState::Critical
        } else if self.degraded_count > 0 {
            HealthState::Degraded
        } else {
            HealthState::Nominal
        };

        self.state = Some(state);
        state
    }

    pub fn make_report(&mut self, filter: Filter) -> ComponentReport {
        let state = self.state();

        ComponentReport {
            name: self.name.clone(),
            state,
            nominal_count: self.nominal_count,
            degraded_count: self.degraded_count,
            critical_count: self.critical_count,
            down_count: self.down_count,
            unrecoverable_count: self.unrecoverable_count,
            degraded: filter.contains(Filter::DEGRADED).then(|| {
                self.degraded
                    .iter()
                    .map(|(msg, count)| (msg.clone(), *count))
                    .collect()
            }),
            critical: filter.contains(Filter::CRITICAL).then(|| {
                self.critical
                    .iter()
                    .map(|(msg, count)| (msg.clone(), *count))
                    .collect()
            }),
            down: filter.contains(Filter::DOWN).then(|| {
                self.down
                    .iter()
                    .map(|(msg, count)| (msg.clone(), *count))
                    .collect()
            }),
            unrecoverable: filter.contains(Filter::UNRECOVERABLE).then(|| {
                self.unrecoverable
                    .iter()
                    .map(|(msg, count)| (msg.clone(), *count))
                    .collect()
            }),
        }
    }
}

/// Decrement a count for an individual key in the map, removing the key when the count reaches zero.
fn dec_or_remove(map: &mut HashMap<PublisherMessage, usize>, key: PublisherMessage) {
    match map.entry(key) {
        Entry::Occupied(mut e) => {
            let c = e.get_mut();
            if *c > 1 {
                *c -= 1;
            } else {
                let _ = e.remove();
            }
        }

        Entry::Vacant(_) => {}
    }
}
