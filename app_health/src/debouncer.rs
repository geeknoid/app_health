use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use tokio::time::{Instant, Sleep, sleep};

pub struct Debouncer {
    debounce_delay: Duration,
    last_fired: Instant,
    timer: Pin<Box<Sleep>>,
    timer_active: bool,
}

impl Debouncer {
    pub fn new(debounce_delay: Duration) -> Self {
        Self {
            debounce_delay,
            last_fired: Instant::now(),
            timer: Box::pin(sleep(Duration::from_secs(0))),
            timer_active: false,
        }
    }

    /// Report that an event we want debounced has occurred.
    ///
    /// Returns `true` if the event should be processed immediately,
    /// or `false` if it should be deferred until the debounce period.
    pub fn trigger(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fired);
        if elapsed >= self.debounce_delay {
            // Immediate update; cancel any pending timer.
            self.timer_active = false;
            self.last_fired = now;
            true
        } else {
            if !self.timer_active {
                // Schedule the debounce timer
                let delay = self.debounce_delay - elapsed;
                self.timer.as_mut().reset(now + delay);
                self.timer_active = true;
            }

            false
        }
    }

    /// Returns a future that resolves when the debounce period has elapsed and pending events should be processed.
    pub const fn ready(&mut self) -> DebounceReady<'_> {
        DebounceReady { debouncer: self }
    }
}

pub struct DebounceReady<'a> {
    debouncer: &'a mut Debouncer,
}

impl Future for DebounceReady<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Only poll the timer if it's active
        if !self.debouncer.timer_active {
            return Poll::Pending;
        }

        // Poll the timer
        match self.debouncer.timer.as_mut().poll(cx) {
            Poll::Ready(()) => {
                // Timer fired and was active, so we can complete
                self.debouncer.timer_active = false;
                self.debouncer.last_fired = Instant::now();
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
