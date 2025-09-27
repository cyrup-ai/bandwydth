# Dioxus Core Concepts Reference

## Components

### Basic Component Structure
```rust
#[component]
fn MyComponent(props: Props) -> Element {
    // 1. Hooks at the top
    let state = use_signal(|| 0);
    
    // 2. Effects after hooks
    use_effect(move || {
        // Side effects
    });
    
    // 3. JSX at the end
    rsx! {
        div { "Hello World" }
    }
}
```

### Component Props
```rust
#[component]
fn Button(
    // Required props
    text: String,
    
    // Optional with default
    #[props(default = "blue")]
    color: String,
    
    // Type conversion
    #[props(into)]
    size: u32,
    
    // Reactive props
    count: ReadOnlySignal<i32>,
    
    // Optional props
    disabled: Option<bool>,
    
    // Children
    children: Element,
) -> Element {
    rsx! {
        button {
            style: "color: {color}; font-size: {size}px",
            disabled: disabled.unwrap_or(false),
            {children}
            "{text} - Count: {count}"
        }
    }
}
```

### Props Attributes
- `#[props(default)]` - Default value for optional props
- `#[props(into)]` - Automatic type conversion via `Into` trait
- `#[props(extends = GlobalAttributes)]` - Inherit HTML attributes
- `Option<T>` - Automatically optional props
- `ReadOnlySignal<T>` - Reactive props that auto-convert

## RSX Syntax

### Basic Elements
```rust
rsx! {
    div {
        class: "container",
        id: "main",
        p { "Hello World" }
    }
}
```

### Text Interpolation
```rust
let name = "Alice";
let count = 42;
rsx! {
    p { "Hello {name}, count: {count}" }
}
```

### Conditional Rendering
```rust
let show_message = true;
rsx! {
    div {
        if show_message {
            p { "Message is shown" }
        }
        
        // Ternary-style
        if count > 0 {
            "Positive"
        } else {
            "Zero or negative"
        }
    }
}
```

### Lists and Iteration
```rust
let items = vec!["apple", "banana", "cherry"];
rsx! {
    ul {
        for item in items {
            li { key: "{item}", "{item}" }
        }
    }
}
```

### Raw Expressions
```rust
let complex_value = calculate_something();
rsx! {
    div {
        // Any Rust expression
        {complex_value}
        
        // Function calls
        {format!("Result: {}", some_function())}
    }
}
```

## State Management with Signals

### Signal Types
- `Signal<T>` - Mutable reactive state
- `ReadOnlySignal<T>` - Immutable view of reactive state
- `GlobalSignal<T>` - Global reactive state
- `Memo<T>` - Computed reactive state
- `Resource<T>` - Async reactive state

### Signal Operations
```rust
let mut count = use_signal(|| 0);

// Reading
let value = count(); // Clones the value
let value_ref = count.read(); // Borrows the value

// Writing
count.set(42);
count += 1; // Shorthand for count.set(count() + 1)
count.with_mut(|c| *c += 1); // Scoped write

// Transforming
let doubled = count.map(|c| c * 2); // ReadOnlySignal<i32>
```

### Signal Best Practices
1. Never hold reads across await points
2. Clone values out before async operations
3. Use scoped writes to avoid borrow conflicts
4. Prefer `ReadOnlySignal` for props that don't need writing

## Reactivity System

### Reactive Contexts
These contexts track dependencies automatically:
- Components
- `use_memo`
- `use_resource`
- `use_effect`

### Tracked Values
Values that reactive contexts can subscribe to:
- `Signal<T>`
- `Memo<T>`
- `Resource<T>`
- `ReadOnlySignal<T>`

### Making Props Reactive
```rust
// ❌ Non-reactive prop
#[component]
fn Counter(count: i32) -> Element {
    let doubled = use_memo(move || count * 2); // Won't update!
    rsx! { "{doubled}" }
}

// ✅ Reactive prop
#[component]
fn Counter(count: ReadOnlySignal<i32>) -> Element {
    let doubled = use_memo(move || count() * 2); // Updates automatically!
    rsx! { "{doubled}" }
}

// ✅ Manual dependency tracking
#[component]
fn Counter(count: i32) -> Element {
    let doubled = use_memo(use_reactive!(|count| count * 2));
    rsx! { "{doubled}" }
}
```

## Component Lifecycle

### Mounting
```rust
#[component]
fn MyComponent() -> Element {
    // This runs once when component mounts
    let state = use_signal(|| {
        println!("Component mounted!");
        0
    });
    
    rsx! { "Component content" }
}
```

### Effects (After Render)
```rust
use_effect(move || {
    // Runs after each render when dependencies change
    println!("Component rendered");
});
```

### Cleanup
```rust
use_effect(move || {
    let timer = set_interval(move || {
        // Timer logic
    }, 1000);
    
    // Cleanup function
    move || {
        clear_interval(timer);
    }
});
```

## Context System

### Providing Context
```rust
#[derive(Clone, Copy)]
struct AppState {
    theme: Signal<String>,
    user: Signal<Option<User>>,
}

#[component]
fn App() -> Element {
    use_context_provider(|| AppState {
        theme: Signal::new("dark".to_string()),
        user: Signal::new(None),
    });
    
    rsx! {
        Router::<Route> {}
    }
}
```

### Consuming Context
```rust
#[component]
fn ThemeButton() -> Element {
    let app_state = use_context::<AppState>();
    
    rsx! {
        button {
            onclick: move |_| {
                let current = app_state.theme();
                let new_theme = if current == "dark" { "light" } else { "dark" };
                app_state.theme.set(new_theme.to_string());
            },
            "Toggle Theme"
        }
    }
}
```

## Global State

### Static Global Signals
```rust
static GLOBAL_COUNT: GlobalSignal<i32> = Signal::global(|| 0);

// Use anywhere in your app
#[component]
fn Counter() -> Element {
    rsx! {
        div {
            "Global count: {GLOBAL_COUNT}"
            button {
                onclick: move |_| *GLOBAL_COUNT.write() += 1,
                "+"
            }
        }
    }
}
```

### Global State Manager
```rust
#[derive(Clone, Copy)]
struct GlobalState {
    count: GlobalSignal<i32>,
    user: GlobalSignal<Option<User>>,
}

static GLOBAL_STATE: GlobalState = GlobalState {
    count: Signal::global(|| 0),
    user: Signal::global(|| None),
};
```

## Key Principles

1. **Explicit State**: State is always explicit and typed
2. **Automatic Reactivity**: Changes propagate automatically
3. **Compile-Time Safety**: Rust's type system prevents runtime errors
4. **Zero-Cost Abstractions**: No runtime overhead for unused features
5. **Cross-Platform**: Same code works on web, desktop, and mobile