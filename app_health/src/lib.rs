//! This crate provides mechanisms to track the overall health of an application. Individual components in an
//! application each report their own health to a central aggregator, which then determines the overall health of the
//! application based on the health of its components.
//!
//! You can probe the aggregator to synchronously get the current health status, including detailed per-component
//! state. Additionally, you can register a callback to be notified asynchronously whenever the overall health status changes.
//!
//! There are two primary uses for the health information:
//!
//! 1. **Telemetry**. The health status can be reported to a telemetry system for monitoring and alerting.
//! 2. **Liveness/Readiness Probes**. The health status can be used to determine if the application is healthy enough to
//!    serve traffic (readiness) or if it should be restarted (liveness).
//!
//! # General Model
//!
//! Here's how the different types interact:
//!
//! - **Aggregator**: The central component that collects health reports from various publishers. It determines the overall health status of the application,
//! and makes the data available for probing by external components.
//!
//! - **Publisher**: Represents a component or subsystem in the application that reports its health status
//! to the aggregator. For example, there might be a Redis Publisher to track the health of the Redis connection.
//!
//! - **Tracker**: A callback function that gets invoked by the aggregator whenever the health of the application
//! changes.
//!
//! - **Report**: A snapshot of the current health status, including details about individual components
//! and their health states.
//!
//! - **Health**: An enumeration representing the possible health states (e.g., Nominal, Degraded, Critical, NonFunctional,
//! and Irrecoverable).
//!
//! - **Filter**: Used to filter the health report based on specific health states.
#![doc = mermaid!("overall.mmd")]

mod aggregator;
mod entries;
mod entry;
mod filter;
mod fragments;
mod health;
mod publisher;
mod report;
mod tracker;

pub use aggregator::Aggregator;
pub use entries::Entries;
pub use entry::Entry;
pub use filter::Filter;
pub use fragments::Fragments;
pub use health::Health;
pub use publisher::Publisher;
pub use report::Report;
use simple_mermaid::mermaid;
pub use tracker::Tracker;

pub(crate) use publisher::PublisherId;
