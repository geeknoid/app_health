use crate::component::{ComponentId, ComponentName};
use crate::tracker::TrackerCallback;
use crate::worker::{ControlRequest, ReportRequest, run_background_worker};
use crate::{ComponentReport, Filter, OverallReport, Publisher, Tracker};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use tokio::sync::{Mutex, mpsc};

/// Aggregates health reports from multiple components and publishers.
///
/// An `Aggregator` instance is typically created at application startup and lives for the entire lifetime of the application.
/// Individual components each get publishers to report their state to the aggregator. The aggregator can be probed using the
/// [`overall_report`](Self::overall_report) and [`component_report`](Self::component_report) methods, or you can register a
/// callback using the [`track`](Self::track) method to get notified whenever the overall health state changes.
#[derive(Debug)]
pub struct Aggregator {
    components_by_name: Arc<Mutex<HashMap<ComponentName, ComponentId>>>,
    next_tracker_id: Arc<AtomicUsize>,
    control_tx: UnboundedSender<ControlRequest>,
    report_tx: Sender<ReportRequest>,
    _shutdown_tx: Sender<()>,
}

impl Aggregator {
    /// Create a new health aggregator.
    #[must_use]
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
        let (control_tx, control_rx) = mpsc::unbounded_channel::<ControlRequest>();
        let (report_tx, report_rx) = mpsc::channel::<ReportRequest>(8);
        let components_by_name = Arc::new(Mutex::new(HashMap::new()));

        // Spawn the background worker task
        drop(tokio::spawn(run_background_worker(
            Arc::clone(&components_by_name),
            control_rx,
            report_rx,
            shutdown_rx,
        )));

        Self {
            control_tx,
            report_tx,
            components_by_name,
            _shutdown_tx: shutdown_tx, // keep the worker alive
            next_tracker_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get a health report for the application.
    ///
    /// This report includes the overall health state of the application, as well as detailed reports for each component.
    /// The filter parameter can be used to control which publisher messages are included in the report.
    #[must_use]
    pub async fn overall_report(&self, filter: Filter) -> OverallReport {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let req = ReportRequest::Overall(filter, response_tx);
        _ = self.report_tx.send(req).await;
        response_rx.await.unwrap_or_default()
    }

    /// Get a health report a specific component within the application.
    ///
    /// The filter parameter can be used to control which publisher messages are included in the report.
    #[must_use]
    pub async fn component_report(
        &self,
        component_name: impl AsRef<str>,
        filter: Filter,
    ) -> Option<ComponentReport> {
        let id = *self
            .components_by_name
            .lock()
            .await
            .get(component_name.as_ref())?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let req = ReportRequest::Component(id, filter, response_tx);
        _ = self.report_tx.send(req).await;
        response_rx.await.ok()
    }

    /// Create a new tracker that gets invoked whenever the application's health state changes.
    ///
    /// The callback is invoked immediately upon registration with the current health state, and
    /// then gets invoked again only when the state changes. The callback at a maximum frequency
    /// (typically every second) to avoid overwhelming the system with notifications. If the health
    /// state changes multiple times within that interval, only the latest state is reported.
    ///
    /// The callback is invoked on a background task on an arbitrary thread. The callback should not
    /// block.
    ///
    /// To stop receiving notifications, drop the tracker instance.
    pub fn track(&mut self, callback: impl TrackerCallback + 'static) -> Tracker {
        let id = self.next_tracker_id.fetch_add(1, Ordering::Relaxed);
        let req = ControlRequest::RegisterTracker(id, Box::new(callback));

        // ignore the results, only possible failure would be if the worker is shutting down, which won't happen as long as the aggregator is alive
        let _ = self.control_tx.send(req);

        Tracker::new(id, self.control_tx.clone())
    }

    /// Create a new publisher for the specific component.
    ///
    /// A publisher makes it possible to report health for a specific component.
    /// Reports of all publishers for the same component are aggregated to determine the overall health of that component.
    #[expect(
        clippy::significant_drop_tightening,
        reason = "The mutex is held only briefly to get or create the component ID."
    )]
    #[expect(
        clippy::option_if_let_else,
        reason = "This would substantially reduce readability."
    )]
    #[must_use]
    pub async fn publisher(&self, component_name: impl Into<ComponentName>) -> Publisher {
        let component_name = component_name.into();

        let mut components_by_name = self.components_by_name.lock().await;
        let id = if let Some(id) = components_by_name.get(&component_name) {
            *id
        } else {
            let new_id = components_by_name.len();
            _ = components_by_name.insert(component_name.clone(), new_id);
            new_id
        };

        Publisher::new(id, self.control_tx.clone())
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}
