//! A simple smoke test for the `app_health` crate

use app_health::Aggregator;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let agg = Aggregator::new();

    // Creating publishers should emit initial Nominal with no prior health
    let p1 = agg.publisher("p1").await;
    let p2 = agg.publisher("p2").await;

    // Only changes should be sent
    p1.degraded("warming");
    p1.degraded("warming"); // duplicate; should not send
    p2.critical("db-down");

    // Give the worker time to process (it ticks every 1s)
    tokio::time::sleep(core::time::Duration::from_millis(1200)).await;

    // Clone p1; clone should announce as healthy for the same id
    let p1_clone = p1.clone();
    // Now publish a change through the clone
    p1_clone.critical("spike");

    tokio::time::sleep(core::time::Duration::from_millis(1200)).await;

    // Drop publishers to send final messages (next: None)
    drop(p1_clone);
    drop(p1);
    drop(p2);

    // Give the worker time to process the drop notifications
    tokio::time::sleep(core::time::Duration::from_millis(1200)).await;
}
