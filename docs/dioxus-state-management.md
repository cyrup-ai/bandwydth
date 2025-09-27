# Dioxus State Management Guide

## Overview
Dioxus provides several state management patterns: local state with signals, global state with context, and global signals for simple shared state.

## Local State with Signals

### use_signal
The primary way to manage local component state:

```rust
let mut count = use_signal(|| 0);

rsx! {
    button {
        onclick: move |_| count += 1,
        "Count: {count}"
    }
}
```

### Reading and Writing Signals
```rust
// Reading
let value = count();
let value = count.read();

// Writing  
count.set(42);
*count.write() = 42;
count += 1; // Shorthand for count.set(count() + 1)
```

## Event Handlers
Attach actions to user interactions:

```rust
rsx! {
    button {
        onclick: move |event| {
            println!("Button clicked at: {:?}", event.coordinates());
        },
        "Click me"
    }
}
```

## State with use_hook
For complex state that doesn't fit signals:

```rust
let state = use_hook(|| MyComplexState::new());
```

## Global State with Context

### Providing Context
```rust
#[derive(Clone)]
struct MusicPlayer {
    song: Signal<String>,
}

#[component]
fn App() -> Element {
    use_context_provider(|| MusicPlayer {
        song: Signal::new("Dust in the Wind".to_string()),
    });
    
    rsx! {
        Player {}
    }
}
```

### Consuming Context
```rust
#[component]
fn Player() -> Element {
    let player = use_context::<MusicPlayer>();
    
    rsx! {
        h3 { "Now playing: {player.song}" }
        button {
            onclick: move |_| player.song.set("Vienna".to_string()),
            "Change Song"
        }
    }
}
```

## Global Signals
For simple global state without context setup:

```rust
static SONG: GlobalSignal<String> = Signal::global(|| "Drift Away".to_string());

#[component]
fn Player() -> Element {
    rsx! {
        h3 { "Now playing {SONG}" }
        button {
            onclick: move |_| *SONG.write() = "Vienna".to_string(),
            "Shuffle"
        }
    }
}
```

## Best Practices
- Use signals for reactive state that triggers re-renders
- Use context for state shared across many components
- Use global signals for simple app-wide state
- Keep state as local as possible
- GlobalSignals are per-app, not per-program

Tags: state, signals, context, global-state, reactivity, hooks, event-handlers