use crate::{Health, PublisherId};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::sleep;

// TODO
//
// - We need to use a bounded channel for input_rx so that publishers don't flood the system.
//   However, this means that publishers may fail to send messages if the channel is full. When
//   that happens, we need to flag the aggregator to reconcile its state with all the publishers
//   such that we don't miss any state changes.
//
// - Need to support configuration to control the wake up interval of the background task, and
//   possibly the size of the bounded channel.

use crate::{Filter, Publisher, Report, Tracker};

// Messages sent from Publisher instances to the Aggregator.
pub struct HealthMessage {
    pub id: PublisherId,
    pub prev: Option<Health>,
    pub next: Option<Health>,
}

struct PublisherState {
    id: PublisherId,
    name: String,
    nominal_count: usize,
    degraded: HashMap<String, usize>,
    critical: HashMap<String, usize>,
    non_functional: HashMap<String, usize>,
    irrecoverable: HashMap<String, usize>,
}

pub struct Aggregator {
    shutdown_tx: broadcast::Sender<()>,
    input_tx: mpsc::UnboundedSender<HealthMessage>,
    task_handle: Option<JoinHandle<()>>,
    next_publisher_id: AtomicUsize,
    publishers: HashMap<PublisherId, Publisher>,
}

impl Aggregator {
    #[must_use]
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        let (input_tx, input_rx) = mpsc::unbounded_channel();

        Self {
            shutdown_tx,
            input_tx,
            task_handle: Some(tokio::spawn(Self::worker(shutdown_rx, input_rx))),
            next_publisher_id: AtomicUsize::new(1),
            publishers: HashMap::default(),
        }
    }

    pub async fn shutdown(mut self) {
        let _ = self.shutdown_tx.send(());

        if let Some(handle) = self.task_handle.take() {
            handle.await.unwrap();
        }
    }

    async fn worker(
        mut shutdown_rx: broadcast::Receiver<()>,
        mut input_rx: mpsc::UnboundedReceiver<HealthMessage>,
    ) {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {
                    while let Ok(message) = input_rx.try_recv() {
                        match (message.prev, message.next) {
                            (None, Some(next)) => {
                                // publisher starting
                            },

                            (Some(prev), Some(next)) => {
                                // published health changed
                            },

                            (prev, None) => {
                                // publisher going away
                            },
                        }
                    }
                }

                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
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

    pub fn publisher(&self, _name: impl AsRef<str>) -> Publisher {
        let id = self.next_publisher_id.fetch_add(1, Ordering::Relaxed);
        Publisher::new(id, self.input_tx.clone())
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}
