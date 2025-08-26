use crate::component::PublisherHealth;
use crate::{ComponentReport, Filter, HealthState, PublisherMessage};
use core::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// A component's state, tracking the aggregate health of its publishers.
#[derive(Debug)]
pub struct ComponentState {
    name: Box<str>,
    state: Cell<Option<HealthState>>,
    nominal_count: usize,
    degraded_count: usize,
    critical_count: usize,
    down_count: usize,
    unrecoverable_count: usize,
    degraded_messages: HashMap<PublisherMessage, usize>,
    critical_messages: HashMap<PublisherMessage, usize>,
    down_messages: HashMap<PublisherMessage, usize>,
    unrecoverable_messages: HashMap<PublisherMessage, usize>,
}

impl ComponentState {
    pub fn new(name: Box<str>) -> Self {
        Self {
            name,
            state: Cell::new(None),
            nominal_count: 0,
            degraded_count: 0,
            critical_count: 0,
            down_count: 0,
            unrecoverable_count: 0,

            degraded_messages: HashMap::new(),
            critical_messages: HashMap::new(),
            down_messages: HashMap::new(),
            unrecoverable_messages: HashMap::new(),
        }
    }

    /// Incorporate a publisher's health into the component's aggregate state.
    pub fn add_publisher_health(&mut self, health: PublisherHealth) {
        self.state.set(None);

        match health {
            PublisherHealth::Nominal => {
                self.nominal_count += 1;
            }

            PublisherHealth::Degraded(message) => {
                self.degraded_count += 1;
                *self.degraded_messages.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Critical(message) => {
                self.critical_count += 1;
                *self.critical_messages.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Down(message) => {
                self.down_count += 1;
                *self.down_messages.entry(message).or_insert(0) += 1;
            }

            PublisherHealth::Unrecoverable(message) => {
                self.unrecoverable_count += 1;
                *self.unrecoverable_messages.entry(message).or_insert(0) += 1;
            }
        }
    }

    /// Remove a publisher's health from the component's aggregate state.
    pub fn remove_publisher_health(&mut self, health: PublisherHealth) {
        self.state.set(None);

        match health {
            PublisherHealth::Nominal => {
                self.nominal_count -= 1;
            }

            PublisherHealth::Degraded(message) => {
                self.degraded_count -= 1;
                dec_or_remove(&mut self.degraded_messages, message);
            }

            PublisherHealth::Critical(message) => {
                self.critical_count -= 1;
                dec_or_remove(&mut self.critical_messages, message);
            }

            PublisherHealth::Down(message) => {
                self.down_count -= 1;
                dec_or_remove(&mut self.down_messages, message);
            }

            PublisherHealth::Unrecoverable(message) => {
                self.unrecoverable_count -= 1;
                dec_or_remove(&mut self.unrecoverable_messages, message);
            }
        }
    }

    /// Get the component's aggregate health state, computing and caching it if needed.
    pub fn state(&self) -> HealthState {
        if let Some(state) = self.state.get() {
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

        self.state.set(Some(state));
        state
    }

    pub fn make_report(&self, filter: Filter) -> ComponentReport {
        let state = self.state();

        ComponentReport {
            name: self.name.to_string(),
            state,
            nominal_count: self.nominal_count,
            degraded_count: self.degraded_count,
            critical_count: self.critical_count,
            down_count: self.down_count,
            unrecoverable_count: self.unrecoverable_count,
            degraded_messages: filter
                .contains(Filter::DEGRADED)
                .then(|| self.degraded_messages.iter().map(|(msg, count)| (msg.clone(), *count)).collect()),
            critical_messages: filter
                .contains(Filter::CRITICAL)
                .then(|| self.critical_messages.iter().map(|(msg, count)| (msg.clone(), *count)).collect()),
            down_messages: filter
                .contains(Filter::DOWN)
                .then(|| self.down_messages.iter().map(|(msg, count)| (msg.clone(), *count)).collect()),
            unrecoverable_messages: filter.contains(Filter::UNRECOVERABLE).then(|| {
                self.unrecoverable_messages
                    .iter()
                    .map(|(msg, count)| (msg.clone(), *count))
                    .collect()
            }),
        }
    }
}

/// Decrement a count for an individual key in the map, removing the key when the count reaches zero.
fn dec_or_remove(map: &mut HashMap<PublisherMessage, usize>, key: PublisherMessage) {
    if let Entry::Occupied(mut e) = map.entry(key) {
        let v = e.get_mut();
        if *v == 1 {
            let _ = e.remove();
        } else {
            *v -= 1;
        }
    }
}
