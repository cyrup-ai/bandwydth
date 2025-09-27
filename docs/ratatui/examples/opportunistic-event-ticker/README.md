# Zero-Shot Spinner: A True No-Tick Event-Driven Animation Demo

This example demonstrates a sophisticated approach to spinner animation in a pure event-driven architecture, without dedicated timer threads or polling loops.

## Philosophy

Traditional terminal spinners rely on background timers that tick at fixed intervals, wasting CPU cycles and fighting against event-driven architectures. This implementation shows how to create smooth spinner animations that are driven entirely by the application's natural event flow.

## Key Innovation: Adaptive Frame Scheduling

Instead of fixed-interval ticks, our spinners use a sliding window approach:

```
[--early window--|----target time----]
     80%             100%
```

- **Target interval**: e.g., 80ms for ~12.5 FPS
- **Early window**: 20% (16ms) - imperceptible to human eye
- **Render decision**: ANY event can trigger a frame advance if we're within the window

## How It Works

### 1. Event Surfing
The spinner "surfs" on the application's natural event stream:
- Key presses
- Mouse movements
- Window resizes
- Network activity
- File system events

During active use, these events provide plenty of render opportunities.

### 2. Self-Sustaining Animation
Each render schedules exactly ONE future event as a safety net:
```rust
render() → advance frame → schedule next in 80ms → (cancel if rendered early)
```

### 3. Perceptual Optimization
The design leverages human psychology:
- **During interaction**: Users focus on their actions, not the spinner
- **During idle**: Scheduled events ensure smooth animation
- **Result**: Perfect perceived smoothness with minimal resources

## Benefits

1. **Zero CPU waste**: No spinning when not visible
2. **Natural throttling**: Debouncing prevents double-advances
3. **Event harmony**: Works WITH the event system, not against it
4. **Deterministic**: Easier testing and debugging
5. **Efficient**: Piggybacks on events that would process anyway

## Architecture

The implementation builds on Ratatui's event-driven model with minimal additions, leveraging our existing concurrent infrastructure from the `concurrent` module:

```
Events → Action → App::handle_action() → Check spinner timing → Maybe advance
                                      ↓
                                   Render → Schedule next frame (via debounced stream)
```

The spinner scheduling uses the same zero-allocation `Debounced` stream and lock-free primitives that power the rest of the application.

### Core Components

- **SpinnerState**: Tracks frame index and next frame timing
- **SpinnerController**: Manages multiple spinners
- **Event System**: Standard crossbeam channels
- **Debounced Stream**: Uses `concurrent::debounced` from our zero-allocation concurrent utilities
- **Cancellation**: Leverages `concurrent::CancellationToken` for clean shutdown
- **Executor**: Built on `concurrent::ExecutorHandle` for task management

### Timing State

Each spinner maintains:
```rust
struct SpinnerTiming {
    next_frame_time: Instant,    // When frame should ideally advance
    frame_interval: Duration,    // Target interval (e.g., 80ms)
    current_frame: usize,        // Current animation frame
}
```

## Running the Demo

```bash
cargo run
```

### Controls
- `h`/`F1` - Show help
- `s` - Select spinner
- `Space` - Pause/resume
- `r` - Reset animation
- `+`/`-` - Adjust speed
- `q` - Quit

## Implementation Notes

### Cross-Cutting Event Subscription
The key insight is intercepting ALL events at the application level. In `App::handle_action()`, before processing specific events:

```rust
// Every event is an opportunity to advance spinners
if self.spinner_controller.should_advance_any() {
    self.spinner_controller.advance_ready_spinners();
}
```

### Sliding Window Logic
```rust
fn should_advance(&self, now: Instant) -> bool {
    let early_threshold = self.next_frame_time - (self.frame_interval * 20 / 100);
    now >= early_threshold
}
```

### Self-Scheduling
After each render, using our concurrent utilities:
```rust
// Using the debounced stream to schedule the next frame
if let Some(next_time) = self.spinner_controller.earliest_next_frame() {
    // The debounced stream ensures proper timing and prevents double-ticks
    let delay = next_time.saturating_duration_since(Instant::now());
    self.spinner_tx.send_delayed(Action::SpinnerTick, delay);
}
```

## Why This Matters

This approach demonstrates that smooth animations don't require fighting the event-driven paradigm. By understanding frame timing, human perception, and event flow, we can create efficient components that enhance rather than burden our applications.

The spinner becomes a lens for understanding larger architectural principles:
- Work with the system, not against it
- Leverage human perception in design
- Question assumptions (do we really need a ticker?)
- Efficiency through elegance, not brute force

## Further Reading

- [Ratatui Event Handling](https://ratatui.rs/concepts/event-handling/)
- [Human Perception of Frame Rates](https://en.wikipedia.org/wiki/Frame_rate#Human_vision)
- [Debouncing in Reactive Systems](https://rxmarbles.com/#debounce)
