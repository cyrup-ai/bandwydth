# Dioxus Hooks and State Management Reference

## Core Hooks

### `use_signal` - Local Reactive State
Create mutable reactive state within a component:
```rust
let mut count = use_signal(|| 0);

// Reading
let value = count(); // Clones the value
let value_ref = count.read(); // Borrows the value

// Writing
count.set(42);
count += 1; // Shorthand
count.with_mut(|c| *c += 1); // Scoped write

// Transforming
let doubled = count.map(|c| c * 2); // Returns ReadOnlySignal<i32>
```

### `use_memo` - Computed State
Create computed values that only recalculate when dependencies change:
```rust
let count = use_signal(|| 0);
let expensive_calc = use_memo(move || {
    // Only runs when count changes
    expensive_computation(count())
});

// Multi-dependency memo
let a = use_signal(|| 1);
let b = use_signal(|| 2);
let sum = use_memo(move || a() + b());
```

### `use_resource` - Async State
Handle async operations that depend on reactive state:
```rust
let user_id = use_signal(|| 1);
let user_data = use_resource(move || async move {
    fetch_user(user_id()).await // Reruns when user_id changes
});

// Using the resource
match &*user_data.read_unchecked() {
    Some(Ok(user)) => rsx! { "User: {user.name}" },
    Some(Err(e)) => rsx! { "Error: {e}" },
    None => rsx! { "Loading..." }
}

// Manual restart
user_data.restart();
```

### `use_effect` - Side Effects
Run side effects after rendering:
```rust
let count = use_signal(|| 0);

// Effect that runs when count changes
use_effect(move || {
    println!("Count changed to: {}", count());
});

// Effect with cleanup
use_effect(move || {
    let timer = set_interval(move || {
        count += 1;
    }, 1000);
    
    // Cleanup function
    move || {
        clear_interval(timer);
    }
});

// Effect that runs only once (empty dependency array equivalent)
use_effect(|| {
    println!("Component mounted");
    // No dependencies, so runs only once
});
```

## Advanced Hooks

### `use_context` - Consuming Context
Access context provided by parent components:
```rust
#[derive(Clone, Copy)]
struct AppState {
    theme: Signal<String>,
    user: Signal<Option<User>>,
}

#[component]
fn ChildComponent() -> Element {
    let app_state = use_context::<AppState>();
    
    rsx! {
        div {
            "Current theme: {app_state.theme}"
            "User: {app_state.user:?}"
        }
    }
}
```

### `use_context_provider` - Providing Context
Provide context to child components:
```rust
#[component]
fn App() -> Element {
    use_context_provider(|| AppState {
        theme: Signal::new("dark".to_string()),
        user: Signal::new(None),
    });
    
    rsx! {
        ChildComponent {}
    }
}
```

### `use_reactive` - Manual Dependency Tracking
Manually specify dependencies for memos and effects:
```rust
#[component]
fn MyComponent(count: i32, name: String) -> Element {
    // Reactive memo that tracks specific props
    let message = use_memo(use_reactive!(|count, name| {
        format!("Hello {name}, count: {count}")
    }));
    
    rsx! { "{message}" }
}
```

## Signal Types Deep Dive

### `Signal<T>` - Full Access
```rust
let mut signal = use_signal(|| "initial".to_string());

// All operations available
signal.set("new value".to_string());
signal.with_mut(|s| s.push_str(" more"));
let value = signal();
let readonly = signal.map(|s| s.len()); // Convert to ReadOnlySignal<usize>
```

### `ReadOnlySignal<T>` - Read-Only Access
```rust
#[component]
fn DisplayOnly(data: ReadOnlySignal<String>) -> Element {
    // Can only read, not write
    let length = use_memo(move || data().len());
    
    rsx! {
        div {
            "Data: {data}"
            "Length: {length}"
        }
    }
}
```

### `GlobalSignal<T>` - Global State
```rust
static GLOBAL_COUNT: GlobalSignal<i32> = Signal::global(|| 0);

// Use anywhere in your app
#[component]
fn AnyComponent() -> Element {
    rsx! {
        div {
            "Global count: {GLOBAL_COUNT}"
            button {
                onclick: move |_| *GLOBAL_COUNT.write() += 1,
                "Increment Global"
            }
        }
    }
}
```

### `Memo<T>` - Computed Signal
```rust
let count = use_signal(|| 0);
let doubled: Memo<i32> = use_memo(move || count() * 2);

// Memos are also signals
let quadrupled = use_memo(move || doubled() * 2);
```

## State Management Patterns

### Local Component State
```rust
#[component]
fn Counter() -> Element {
    let count = use_signal(|| 0);
    
    rsx! {
        div {
            "Count: {count}"
            button { onclick: move |_| count += 1, "+" }
            button { onclick: move |_| count -= 1, "-" }
        }
    }
}
```

### Shared State Between Components
```rust
#[component]
fn Parent() -> Element {
    let shared_count = use_signal(|| 0);
    
    rsx! {
        div {
            Counter { count: shared_count }
            Display { count: shared_count }
        }
    }
}

#[component]
fn Counter(count: Signal<i32>) -> Element {
    rsx! {
        button { onclick: move |_| count += 1, "+" }
    }
}

#[component]
fn Display(count: ReadOnlySignal<i32>) -> Element {
    rsx! { "Count: {count}" }
}
```

### Context-Based State Management
```rust
#[derive(Clone, Copy)]
struct AppState {
    user: Signal<Option<User>>,
    notifications: Signal<Vec<Notification>>,
    theme: Signal<Theme>,
}

impl AppState {
    fn login(&self, user: User) {
        self.user.set(Some(user));
    }
    
    fn logout(&self) {
        self.user.set(None);
    }
    
    fn add_notification(&self, notification: Notification) {
        self.notifications.with_mut(|n| n.push(notification));
    }
}

#[component]
fn App() -> Element {
    use_context_provider(|| AppState {
        user: Signal::new(None),
        notifications: Signal::new(Vec::new()),
        theme: Signal::new(Theme::Dark),
    });
    
    rsx! {
        Router::<Route> {}
    }
}
```

### Global State Management
```rust
// Define global state
#[derive(Clone, Copy)]
struct GlobalAppState {
    count: GlobalSignal<i32>,
    user: GlobalSignal<Option<User>>,
    settings: GlobalSignal<AppSettings>,
}

static APP_STATE: GlobalAppState = GlobalAppState {
    count: Signal::global(|| 0),
    user: Signal::global(|| None),
    settings: Signal::global(|| AppSettings::default()),
};

// Actions/mutations
impl GlobalAppState {
    fn increment_count(&self) {
        *self.count.write() += 1;
    }
    
    fn login(&self, user: User) {
        *self.user.write() = Some(user);
    }
    
    fn update_settings(&self, settings: AppSettings) {
        *self.settings.write() = settings;
    }
}

// Use in components
#[component]
fn AnyComponent() -> Element {
    rsx! {
        div {
            "Count: {APP_STATE.count}"
            "User: {APP_STATE.user:?}"
            button {
                onclick: move |_| APP_STATE.increment_count(),
                "Increment"
            }
        }
    }
}
```

## Advanced State Patterns

### State Machines
```rust
#[derive(Clone, PartialEq)]
enum LoadingState<T> {
    Idle,
    Loading,
    Success(T),
    Error(String),
}

#[component]
fn DataLoader() -> Element {
    let state = use_signal(|| LoadingState::<String>::Idle);
    
    let load_data = move |_| async move {
        state.set(LoadingState::Loading);
        
        match fetch_data().await {
            Ok(data) => state.set(LoadingState::Success(data)),
            Err(e) => state.set(LoadingState::Error(e.to_string())),
        }
    };
    
    match state() {
        LoadingState::Idle => rsx! {
            button { onclick: load_data, "Load Data" }
        },
        LoadingState::Loading => rsx! {
            div { "Loading..." }
        },
        LoadingState::Success(data) => rsx! {
            div { "Data: {data}" }
        },
        LoadingState::Error(error) => rsx! {
            div { 
                "Error: {error}"
                button { onclick: load_data, "Retry" }
            }
        },
    }
}
```

### Optimistic Updates
```rust
#[component]
fn OptimisticCounter() -> Element {
    let count = use_signal(|| 0);
    let pending_count = use_signal(|| 0);
    
    let increment = move |_| async move {
        // Optimistic update
        pending_count.set(count() + 1);
        
        match api_increment().await {
            Ok(new_count) => {
                count.set(new_count);
                pending_count.set(new_count);
            }
            Err(_) => {
                // Revert optimistic update
                pending_count.set(count());
            }
        }
    };
    
    rsx! {
        div {
            "Count: {pending_count}"
            button { onclick: increment, "+" }
        }
    }
}
```

### Undo/Redo Pattern
```rust
#[derive(Clone)]
struct History<T> {
    past: Vec<T>,
    present: T,
    future: Vec<T>,
}

impl<T: Clone> History<T> {
    fn new(initial: T) -> Self {
        Self {
            past: Vec::new(),
            present: initial,
            future: Vec::new(),
        }
    }
    
    fn push(&mut self, new_state: T) {
        self.past.push(self.present.clone());
        self.present = new_state;
        self.future.clear();
    }
    
    fn undo(&mut self) -> bool {
        if let Some(previous) = self.past.pop() {
            self.future.push(self.present.clone());
            self.present = previous;
            true
        } else {
            false
        }
    }
    
    fn redo(&mut self) -> bool {
        if let Some(next) = self.future.pop() {
            self.past.push(self.present.clone());
            self.present = next;
            true
        } else {
            false
        }
    }
}

#[component]
fn UndoableCounter() -> Element {
    let history = use_signal(|| History::new(0));
    
    let increment = move |_| {
        history.with_mut(|h| h.push(h.present + 1));
    };
    
    let undo = move |_| {
        history.with_mut(|h| h.undo());
    };
    
    let redo = move |_| {
        history.with_mut(|h| h.redo());
    };
    
    rsx! {
        div {
            "Count: {history.read().present}"
            button { onclick: increment, "+" }
            button { 
                disabled: history.read().past.is_empty(),
                onclick: undo, 
                "Undo" 
            }
            button { 
                disabled: history.read().future.is_empty(),
                onclick: redo, 
                "Redo" 
            }
        }
    }
}
```

## Performance Considerations

### Signal Performance Tips
1. **Avoid unnecessary clones**: Use `read()` for borrowing when possible
2. **Batch updates**: Use `with_mut()` for multiple mutations
3. **Use ReadOnlySignal for props**: Prevents accidental writes and enables optimizations
4. **Don't hold reads across await points**: Can cause runtime panics

### Memory Management
```rust
// ✅ Good: Signals are automatically cleaned up
#[component]
fn MyComponent() -> Element {
    let signal = use_signal(|| Vec::new());
    // Signal is automatically dropped when component unmounts
    rsx! { /* ... */ }
}

// ❌ Bad: Don't create signals outside components
static BAD_SIGNAL: Signal<i32> = Signal::new(0); // This won't work!

// ✅ Good: Use GlobalSignal for global state
static GOOD_GLOBAL: GlobalSignal<i32> = Signal::global(|| 0);
```

### Common Pitfalls
```rust
// ❌ Don't hold reads across await points
let bad_async = move |_| async move {
    let value = signal.read(); // Starts borrow
    fetch_data().await; // Await point - panic!
    println!("{}", *value); // Use after await - panic!
};

// ✅ Clone the value before async operations
let good_async = move |_| async move {
    let value = signal(); // Clones the value
    fetch_data().await; // Safe!
    println!("{}", value); // Safe!
};
```