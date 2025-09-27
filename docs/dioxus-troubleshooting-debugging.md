# Dioxus Troubleshooting and Debugging Guide

## Common Compilation Errors

### Hook Order Violations
```rust
// ❌ ERROR: Hooks called in different order
#[component]
fn BadComponent(condition: bool) -> Element {
    if condition {
        let state = use_signal(|| 0); // Hook called conditionally
    }
    
    let other_state = use_signal(|| "hello");
    
    rsx! { "Content" }
}

// ✅ FIXED: Always call hooks in the same order
#[component]
fn GoodComponent(condition: bool) -> Element {
    let state = use_signal(|| if condition { 0 } else { -1 });
    let other_state = use_signal(|| "hello");
    
    rsx! { "Content" }
}
```

### Missing `move` Keywords
```rust
// ❌ ERROR: Captured variables don't live long enough
#[component]
fn BadEventHandler() -> Element {
    let count = use_signal(|| 0);
    
    rsx! {
        button {
            onclick: |_| count += 1, // Missing `move`
            "Click"
        }
    }
}

// ✅ FIXED: Use `move` to capture variables
#[component]
fn GoodEventHandler() -> Element {
    let count = use_signal(|| 0);
    
    rsx! {
        button {
            onclick: move |_| count += 1, // `move` captures count
            "Click"
        }
    }
}
```

### Incorrect Return Types
```rust
// ❌ ERROR: Expected Element, found ()
#[component]
fn BadComponent() -> Element {
    let count = use_signal(|| 0);
    
    if count() > 5 {
        return; // Returns () instead of Element
    }
    
    rsx! { "Count: {count}" }
}

// ✅ FIXED: Always return Element
#[component]
fn GoodComponent() -> Element {
    let count = use_signal(|| 0);
    
    if count() > 5 {
        return rsx! { "Count is too high!" };
    }
    
    rsx! { "Count: {count}" }
}
```

### Props Macro Issues
```rust
// ❌ ERROR: Missing #[component] macro
fn BadComponent(name: String) -> Element {
    rsx! { "Hello {name}" }
}

// ✅ FIXED: Add #[component] macro
#[component]
fn GoodComponent(name: String) -> Element {
    rsx! { "Hello {name}" }
}
```

## Runtime Errors and Panics

### Signal Borrow Checker Panics
```rust
// ❌ PANIC: Borrow held across await point
let bad_async = move |_| async move {
    let value = signal.read(); // Starts borrow
    async_operation().await; // Panic! Borrow held across await
    println!("{}", *value);
};

// ✅ FIXED: Clone value before async operations
let good_async = move |_| async move {
    let value = signal(); // Clones the value
    async_operation().await; // Safe!
    println!("{}", value);
};

// ❌ PANIC: Multiple mutable borrows
let bad_handler = move |_| {
    let mut write1 = signal.write(); // First mutable borrow
    let mut write2 = signal.write(); // Panic! Second mutable borrow
    *write1 += 1;
    *write2 += 1;
};

// ✅ FIXED: Use scoped writes or clone
let good_handler = move |_| {
    signal.with_mut(|val| *val += 2); // Single scoped write
};
```

### Context Not Found Errors
```rust
// ❌ PANIC: Context not provided
#[component]
fn ChildComponent() -> Element {
    let state = use_context::<AppState>(); // Panics if not provided
    rsx! { "State: {state:?}" }
}

// ✅ FIXED: Provide context in parent
#[component]
fn ParentComponent() -> Element {
    use_context_provider(|| AppState::new());
    
    rsx! {
        ChildComponent {}
    }
}

// ✅ ALTERNATIVE: Use optional context
#[component]
fn SafeChildComponent() -> Element {
    match try_use_context::<AppState>() {
        Some(state) => rsx! { "State: {state:?}" },
        None => rsx! { "No state provided" },
    }
}
```

### Resource State Errors
```rust
// ❌ ERROR: Accessing resource before it's ready
#[component]
fn BadResourceUsage() -> Element {
    let data = use_resource(|| async { fetch_data().await });
    
    // This will panic if resource isn't ready
    let value = data.read().as_ref().unwrap().as_ref().unwrap();
    
    rsx! { "Data: {value}" }
}

// ✅ FIXED: Properly handle resource states
#[component]
fn GoodResourceUsage() -> Element {
    let data = use_resource(|| async { fetch_data().await });
    
    match &*data.read_unchecked() {
        Some(Ok(value)) => rsx! { "Data: {value}" },
        Some(Err(e)) => rsx! { "Error: {e}" },
        None => rsx! { "Loading..." },
    }
}
```

## Debugging Techniques

### Adding Debug Output
```rust
#[component]
fn DebuggableComponent(props: MyProps) -> Element {
    let state = use_signal(|| 0);
    
    // Debug props
    tracing::debug!("Component rendered with props: {props:?}");
    
    // Debug state changes
    use_effect(move || {
        tracing::debug!("State changed to: {}", state());
    });
    
    rsx! {
        div {
            // Debug current values in UI
            div { style: "display: none;", "Debug: state={state}, props={props:?}" }
            "Count: {state}"
            button {
                onclick: move |_| {
                    tracing::debug!("Button clicked, incrementing state");
                    state += 1;
                },
                "+"
            }
        }
    }
}
```

### Component Inspector
```rust
#[component]
fn ComponentInspector(children: Element) -> Element {
    let render_count = use_signal(|| 0);
    
    use_effect(move || {
        render_count += 1;
        tracing::info!("Component rendered {} times", render_count());
    });
    
    rsx! {
        div {
            div { 
                style: "position: fixed; top: 0; right: 0; background: yellow; padding: 4px; font-size: 12px;",
                "Renders: {render_count}"
            }
            {children}
        }
    }
}
```

### Signal Debugging
```rust
// Custom signal wrapper with debugging
fn debug_signal<T: std::fmt::Debug + Clone + 'static>(
    initial: T,
    name: &'static str,
) -> Signal<T> {
    let signal = use_signal(move || initial.clone());
    
    use_effect(move || {
        tracing::debug!("Signal '{}' changed to: {:?}", name, signal());
    });
    
    signal
}

#[component]
fn ComponentWithDebugSignals() -> Element {
    let count = debug_signal(0, "count");
    let name = debug_signal("".to_string(), "name");
    
    rsx! {
        div {
            input {
                value: name(),
                oninput: move |e| name.set(e.value())
            }
            button {
                onclick: move |_| count += 1,
                "Count: {count}"
            }
        }
    }
}
```

## Performance Debugging

### Identifying Unnecessary Re-renders
```rust
#[component]
fn PerformanceMonitor(name: &'static str, children: Element) -> Element {
    let render_count = use_signal(|| 0);
    let last_render = use_signal(|| std::time::Instant::now());
    
    use_effect(move || {
        let now = std::time::Instant::now();
        let since_last = now.duration_since(last_render());
        render_count += 1;
        
        if since_last.as_millis() < 100 {
            tracing::warn!(
                "Component '{}' rendered {} times in {}ms - possible performance issue",
                name,
                render_count(),
                since_last.as_millis()
            );
        }
        
        last_render.set(now);
    });
    
    children
}

// Usage
rsx! {
    PerformanceMonitor { name: "MyComponent",
        MyComponent { /* props */ }
    }
}
```

### Memory Usage Monitoring
```rust
#[component]
fn MemoryMonitor() -> Element {
    let memory_usage = use_signal(|| 0);
    
    use_effect(|| {
        spawn(async move {
            loop {
                // This is a simplified example - actual memory monitoring
                // would require platform-specific APIs
                let usage = get_memory_usage(); // Hypothetical function
                memory_usage.set(usage);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    });
    
    rsx! {
        div {
            style: "position: fixed; bottom: 0; right: 0; background: #333; color: white; padding: 4px;",
            "Memory: {memory_usage} MB"
        }
    }
}
```

## Web-Specific Debugging

### Browser DevTools Integration
```rust
#[component]
fn WebDebugger() -> Element {
    let state = use_signal(|| 0);
    
    // Log to browser console
    use_effect(move || {
        web_sys::console::log_1(&format!("State changed: {}", state()).into());
    });
    
    rsx! {
        div {
            button {
                onclick: move |_| {
                    state += 1;
                    // Debug specific events
                    web_sys::console::group_1(&"Button Click Event".into());
                    web_sys::console::log_1(&format!("New state: {}", state()).into());
                    web_sys::console::group_end();
                },
                "Count: {state}"
            }
        }
    }
}
```

### Network Request Debugging
```rust
#[component]
fn NetworkDebugger() -> Element {
    let requests = use_signal(|| Vec::<String>::new());
    
    let make_request = move |_| async move {
        let start = std::time::Instant::now();
        
        requests.with_mut(|r| r.push("Request started".to_string()));
        
        match fetch_data().await {
            Ok(data) => {
                let duration = start.elapsed();
                requests.with_mut(|r| {
                    r.push(format!("Request completed in {:?}: {}", duration, data));
                });
            }
            Err(e) => {
                requests.with_mut(|r| {
                    r.push(format!("Request failed: {}", e));
                });
            }
        }
    };
    
    rsx! {
        div {
            button { onclick: make_request, "Make Request" }
            div {
                h3 { "Request Log:" }
                ul {
                    for req in requests() {
                        li { "{req}" }
                    }
                }
            }
        }
    }
}
```

## Desktop-Specific Debugging

### Native Logging
```rust
#[component]
fn DesktopDebugger() -> Element {
    use_effect(|| {
        // Set up file logging for desktop apps
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("app_debug.log")
            .unwrap();
        
        // Configure logging to file
        // (This is a simplified example)
    });
    
    rsx! {
        div { "Check app_debug.log for detailed logs" }
    }
}
```

## Common Anti-Patterns and Solutions

### Anti-Pattern: Creating Signals in Loops
```rust
// ❌ BAD: Creating signals in loops
#[component]
fn BadListComponent(items: Vec<String>) -> Element {
    rsx! {
        ul {
            for item in items {
                li {
                    // This creates a new signal for each item on every render!
                    {
                        let signal = use_signal(|| item.clone());
                        format!("{}", signal())
                    }
                }
            }
        }
    }
}

// ✅ GOOD: Stable component structure
#[component]
fn GoodListComponent(items: Vec<String>) -> Element {
    rsx! {
        ul {
            for (i, item) in items.iter().enumerate() {
                ListItem { key: "{i}", text: item.clone() }
            }
        }
    }
}

#[component]
fn ListItem(text: String) -> Element {
    let signal = use_signal(move || text);
    rsx! { li { "{signal}" } }
}
```

### Anti-Pattern: Overusing Global State
```rust
// ❌ BAD: Everything in global state
static GLOBAL_USER_NAME: GlobalSignal<String> = Signal::global(|| String::new());
static GLOBAL_USER_EMAIL: GlobalSignal<String> = Signal::global(|| String::new());
static GLOBAL_FORM_ERRORS: GlobalSignal<Vec<String>> = Signal::global(|| Vec::new());

// ✅ GOOD: Use local state when appropriate
#[component]
fn UserForm() -> Element {
    let user_name = use_signal(String::new);
    let user_email = use_signal(String::new);
    let form_errors = use_signal(|| Vec::<String>::new());
    
    // Only store truly global state globally
    let app_user = use_context::<GlobalUser>();
    
    rsx! { /* form content */ }
}
```

## Testing and Debugging Tools

### Custom Testing Utilities
```rust
#[cfg(test)]
mod test_utils {
    use super::*;
    
    pub fn render_component<F>(component: F) -> VirtualDom 
    where 
        F: Fn() -> Element + 'static 
    {
        VirtualDom::new(component)
    }
    
    pub fn find_element_by_id(dom: &VirtualDom, id: &str) -> Option<&VNode> {
        // Implementation to find elements in the virtual DOM
        // This is a simplified example
        None
    }
    
    pub fn simulate_click(dom: &mut VirtualDom, element_id: &str) {
        // Implementation to simulate click events
        // This is a simplified example
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[test]
    fn test_counter_component() {
        let mut dom = render_component(|| rsx! {
            Counter { initial_value: 5 }
        });
        
        let _ = dom.rebuild();
        
        // Test assertions here
        assert!(true); // Placeholder
    }
}
```

### Hot Reload Development
```rust
// Use subsecond for hot reloading during development
#[component]
fn DevelopmentWrapper(children: Element) -> Element {
    #[cfg(debug_assertions)]
    {
        subsecond::call(|| {
            rsx! {
                div {
                    div { 
                        style: "background: orange; color: black; padding: 2px; font-size: 10px;",
                        "🔥 Hot Reload Active"
                    }
                    {children}
                }
            }
        })
    }
    
    #[cfg(not(debug_assertions))]
    children
}
```

## Error Recovery Patterns

### Graceful Error Boundaries
```rust
#[component]
fn ErrorBoundary(fallback: Element, children: Element) -> Element {
    let error_state = use_signal(|| None::<String>);
    
    // In a real implementation, you'd need to catch panics
    // This is a simplified example
    
    if let Some(error) = error_state() {
        rsx! {
            div { class: "error-boundary",
                h2 { "Something went wrong" }
                details {
                    summary { "Error details" }
                    pre { "{error}" }
                }
                button {
                    onclick: move |_| error_state.set(None),
                    "Try Again"
                }
                {fallback}
            }
        }
    } else {
        children
    }
}
```

### Retry Mechanisms
```rust
#[component]
fn RetryWrapper<T: Clone + 'static>(
    max_retries: usize,
    operation: impl Fn() -> T + 'static,
    children: impl Fn(T) -> Element + 'static,
) -> Element {
    let result = use_signal(|| None::<T>);
    let error = use_signal(|| None::<String>);
    let retry_count = use_signal(|| 0);
    
    let attempt_operation = move |_| {
        if retry_count() < max_retries {
            retry_count += 1;
            
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| operation())) {
                Ok(value) => {
                    result.set(Some(value));
                    error.set(None);
                }
                Err(_) => {
                    error.set(Some(format!("Attempt {} failed", retry_count())));
                }
            }
        }
    };
    
    // Auto-retry on mount
    use_effect(move || {
        if result().is_none() && retry_count() == 0 {
            attempt_operation(());
        }
    });
    
    match (result(), error()) {
        (Some(value), _) => children(value),
        (None, Some(err)) if retry_count() >= max_retries => rsx! {
            div { class: "error",
                "Operation failed after {max_retries} attempts: {err}"
            }
        },
        (None, Some(err)) => rsx! {
            div {
                "Error: {err}"
                button { onclick: attempt_operation, "Retry ({retry_count}/{max_retries})" }
            }
        },
        (None, None) => rsx! { "Initializing..." },
    }
}
```