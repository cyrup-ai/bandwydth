use lib_bandwydth::concurrent::{OpportunisticDebouncedTicker, Tickable};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct TestTickable {
    ticks: Arc<Mutex<u32>>,
    active: bool,
    interval: Duration,
}

impl Tickable for TestTickable {
    fn tick(&mut self) {
        *self.ticks.lock().expect("lock poisoned") += 1;
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn frame_interval(&self) -> Duration {
        self.interval
    }
}

#[test]
fn test_opportunistic_ticking() {
    println!("Starting test_opportunistic_ticking");
    let ticks = Arc::new(Mutex::new(0));
    let tickable = TestTickable {
        ticks: ticks.clone(),
        active: true,
        interval: Duration::from_millis(100),
    };

    let mut ticker = OpportunisticDebouncedTicker::new(tickable);

    // First event should tick immediately
    assert!(ticker.on_any_event());
    assert_eq!(*ticks.lock().expect("lock poisoned"), 1);

    // Immediate event shouldn't tick (too early)
    assert!(!ticker.on_any_event());
    assert_eq!(*ticks.lock().expect("lock poisoned"), 1);

    // Event at 80% of interval should tick (80ms for 100ms interval)
    std::thread::sleep(Duration::from_millis(80));
    assert!(ticker.on_any_event());
    assert_eq!(*ticks.lock().expect("lock poisoned"), 2);
}

#[test]
fn test_metrics_tracking() {
    println!("Starting test_metrics_tracking");
    let tickable = TestTickable {
        ticks: Arc::new(Mutex::new(0)),
        active: true,
        interval: Duration::from_millis(50),
    };

    let mut ticker = OpportunisticDebouncedTicker::new(tickable);

    // Opportunistic tick
    ticker.on_any_event();
    assert_eq!(ticker.metrics().opportunistic_ticks, 1);
    assert_eq!(ticker.metrics().fallback_ticks, 0);

    // Schedule and drain
    ticker.schedule_fallback();
    std::thread::sleep(Duration::from_millis(45)); // 90% of interval
    ticker.on_any_event();

    // Should have cancelled the fallback
    assert_eq!(ticker.metrics().cancelled_fallbacks, 1);
}
