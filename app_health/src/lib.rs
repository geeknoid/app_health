//! Application Health Monitoring
//!
//! This crate provides mechanisms to track the overall health of an application. Individual components in an
//! application each report their own health to a central aggregator, which then determines the overall health of the
//! application based on the health of its components.
//!
//! You can probe the aggregator to synchronously get the current health status, including detailed per-component
//! state. Additionally, you can asynchronously wait to be notified whenever the overall health of the
//! application changes.
//!
//! There are two primary uses for the health information that the aggregator collects:
//!
//! 1. **Telemetry**. The application's health state can be reported to a telemetry system for monitoring and alerting.
//! 2. **Liveness/Readiness Probes**. The application's health state can be used to determine if the application is healthy enough to
//!    serve traffic (readiness) or if it should be restarted (liveness).
//!
//! # General Model
//!
//! Here's how the different types interact:
//!
//! - **[`Aggregator`]**: The central entity that collects health information from a set of components. It determines the overall health status of the application.
//!
//! - **[`Component`]**: Represents a specific feature or subsystem within the application. Each component can have multiple publishers determining its health.
//!
//! - **[`Publisher`]**: An entity that can publish health information regarding a specific component in the application. A single component
//!   can have multiple publishers. For example, one per thread.

//! - **[`Monitor`]**: An entity that lets you collect reports about the health of the application as well as being
//!   notified where there are significant changes in the application's health.
//!
#![cfg_attr(feature = "mermaid", doc = simple_mermaid::mermaid!("overall.mmd"))]
//!
//! # Reports
//!
//! You can query a monitor to get detailed information about the application's overall health, or the
//! health of individual components using these reporting functions:
//!
//! - **[`Monitor::report`](Monitor::report)**: Get a snapshot of the current overall health status, including details about individual components and their health states.
//!
//! - **[`ComponentMonitor::report`](ComponentMonitor::report)**: Get a snapshot of the health status of a specific component, including details about its publishers and their
//!   health states.
//!
//! Both functions accept a **[`Filter`]** parameter which lets you control the level of detail returned in the reports.
//!
//! # Example
//!
//! ```rust
//! use app_health::{Aggregator, Filter};
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     // Create a health aggregator
//!     let aggregator = Aggregator::new();
//!
//!     // Create components
//!     let db_component = aggregator.component("database");
//!     let cache_component = aggregator.component("cache");
//!
//!     // Create publishers for different components
//!     let mut db_publisher = db_component.publisher();
//!     let mut cache_publisher = cache_component.publisher();
//!
//!     // Simulate health state changes, these calls would normally occur inside the respective components in
//!     // response to observed conditions
//!     db_publisher.degraded("High latency detected");
//!     cache_publisher.critical("Cache server unreachable");
//!
//!     // Wait a moment to allow the aggregator to process updates
//!     sleep(Duration::from_millis(1200)).await;
//!
//!     // Query the overall health report
//!     let monitor = aggregator.monitor();
//!     let report = monitor.report(Filter::empty()).await;
//!     println!("Overall Health: {:?}", report);
//!
//!     // Query a specific component's health report
//!     let component_monitor = db_component.monitor();
//!     let db_report = component_monitor.report(Filter::empty()).await;
//!     println!("Database Health: {:?}", db_report);
//! }
//! ```

mod aggregator;
mod component;
mod component_monitor;
mod component_report;
mod component_state;
mod filter;
mod health_state;
mod messages;
mod monitor;
mod publisher;
mod report;
mod debouncer;

pub use aggregator::Aggregator;
pub use component::Component;
pub use component_monitor::ComponentMonitor;
pub use component_report::ComponentReport;
pub use filter::Filter;
pub use health_state::HealthState;
pub use messages::Messages;
pub use monitor::Monitor;
pub use publisher::{Publisher, PublisherMessage};
pub use report::Report;
