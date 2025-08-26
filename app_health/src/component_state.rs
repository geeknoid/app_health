use crate::health::{ALL_HEALTH_STATES, NUM_HEALTH_STATES};
use crate::signal::Signal;
use crate::{Filter, Health, Report};
use core::array::from_fn;
use core::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

/// A component's state, tracking the aggregate health of its publishers.
#[derive(Debug)]
pub struct ComponentState {
    name: Arc<str>,
    state: Cell<Option<Health>>,
    counts: [usize; NUM_HEALTH_STATES],
    signals: [HashMap<Signal, usize>; NUM_HEALTH_STATES],
}

impl ComponentState {
    pub fn new(name: Arc<str>) -> Self {
        Self {
            name,
            state: Cell::new(None),
            counts: [0; NUM_HEALTH_STATES],
            signals: from_fn(|_| HashMap::default()),
        }
    }

    /// Incorporate a publisher's health into the component's aggregate state.
    pub fn add_publisher_signal(&mut self, signal: Signal) {
        // induce the state to be recomputed on query
        self.state.set(None);

        let index = signal.state() as usize;
        self.counts[index] += 1;
        *self.signals[index].entry(signal).or_insert(0) += 1;
    }

    /// Remove a publisher's health from the component's aggregate state.
    pub fn remove_publisher_signal(&mut self, signal: Signal) {
        // induce the state to be recomputed on query
        self.state.set(None);

        let index = signal.state() as usize;
        self.counts[index] -= 1;
        dec_or_remove(&mut self.signals[index], signal);
    }

    /// Get the component's aggregate health state, computing and caching it if needed.
    pub fn state(&self) -> Health {
        if let Some(state) = self.state.get() {
            return state;
        }

        let state = ALL_HEALTH_STATES
            .iter()
            .rev()
            .find_map(|hs| (self.counts[*hs as usize] > 0).then_some(*hs))
            .unwrap_or_default();

        self.state.set(Some(state));
        state
    }

    pub fn make_report(&self, filter: Filter) -> Report {
        let state = self.state();

        Report {
            name: Arc::clone(&self.name),
            state,
            counts: self.counts,
            signals: from_fn(|i| {
                let health_state_bit = 1 << i;

                if filter.bits() & health_state_bit != 0 {
                    self.signals[i].iter().map(|(msg, count)| (msg.clone(), *count)).collect()
                } else {
                    Vec::new()
                }
            }),
        }
    }
}

/// Decrement a count for an individual key in the map, removing the key when the count reaches zero.
fn dec_or_remove(map: &mut HashMap<Signal, usize>, key: Signal) {
    if let Entry::Occupied(mut e) = map.entry(key) {
        let v = e.get_mut();
        if *v == 1 {
            let _ = e.remove();
        } else {
            *v -= 1;
        }
    }
}
