use app_health::{Aggregator, Health};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut agg = Aggregator::new();

    // Creating publishers should emit initial Nominal with no prior health
    let mut p1 = agg.publisher("p1");
    let mut p2 = agg.publisher("p2");

    // Only changes should be sent
    p1.publish(Health::Degraded("warming".into()));
    p1.publish(Health::Degraded("warming".into())); // duplicate; should not send
    p2.publish(Health::Critical("db-down".into()));

    // Give the worker time to process (it ticks every 1s)
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    // Clone p1; clone should announce as healthy for the same id
    let mut p1_clone = p1.clone();
    // Now publish a change through the clone
    p1_clone.publish(Health::Critical("spike".into()));

    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    // Drop publishers to send final messages (next: None)
    drop(p1_clone);
    drop(p1);
    drop(p2);

    // Give the worker time to process the drop notifications
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    agg.shutdown().await;
}
