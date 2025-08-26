//! Application Health Monitoring
//!
//! This crate provides mechanisms to track the overall health of an application. Individual components in an
//! application each report their own health to a central aggregator, which then determines the overall health of the
//! application based on the health of its components.
//!
//! You can probe the aggregator to synchronously get the current health status, including detailed per-component
//! state. Additionally, you can register a callback to be notified asynchronously whenever the overall health of the\
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
//! - **[`Aggregator`]**: The central component that collects health reports from various publishers. It determines the overall health status of the application
//!   and makes the data available for probing by external components.
//!
//! - **[`Publisher`]**: Represents a component or subsystem in the application that reports its health status
//!   to the aggregator. For example, there might be a Redis Publisher to track the health of a connection to Redis.
//!
//! - **[`Tracker`]**: A callback function that gets invoked by the aggregator whenever the application's health state changes.
//!
#![cfg_attr(feature = "mermaid", doc = simple_mermaid::mermaid!("overall.mmd"))]
//!
//! # Reports
//!
//! You can query an aggregator to get detailed information about the application's overall health, or the
//! health of individual components using these reporting functions:
//!
//! - **[`overall_report`](Aggregator::overall_report)**: Get a snapshot of the current overall health status, including details about individual components and their health states.
//!
//! - **[`component_report`](Aggregator::component_report)**: Get a snapshot of the health status of a specific component, including details about its publishers and their
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
//!     // Create publishers for different components
//!     let db_publisher = aggregator.publisher("database").await;
//!     let cache_publisher = aggregator.publisher("cache").await;
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
//!     let overall_report = aggregator.overall_report(Filter::empty()).await;
//!     println!("Overall Health: {:?}", overall_report);
//!
//!     // Query a specific component's health report
//!     if let Some(db_report) = aggregator.component_report("database", Filter::empty()).await
//!     {
//!         println!("Database Health: {:?}", db_report);
//!     }
//! }
//! ```

mod aggregator;
mod component;
mod component_report;
mod filter;
mod health_state;
mod messages;
mod overall_report;
mod publisher;
mod tracker;
mod tracker_manager;
mod worker;

pub use aggregator::Aggregator;
pub use component::ComponentName;
pub use component_report::ComponentReport;
pub use filter::Filter;
pub use health_state::HealthState;
pub use messages::Messages;
pub use overall_report::OverallReport;
pub use publisher::{Publisher, PublisherMessage};
pub use tracker::Tracker;
