# Dioxus Event Handling and Async Guide

## Event Handling Fundamentals

### Basic Event Handlers
Event handlers in Dioxus are closures that must be `'static`:
```rust
rsx! {
    button {
        onclick: move |event| {
            println!("Button clicked! Event: {event:?}");
        },
        "Click me"
    }
}
```

### Event Types
Common event types and their data:
```rust
rsx! {
    div {
        // Mouse events
        onclick: move |event: MouseEvent| {
            println!("Clicked at: ({}, {})", event.page_x(), event.page_y());
        },
        
        // Keyboard events
        onkeydown: move |event: KeyboardEvent| {
            println!("Key pressed: {}", event.key());
            if event.key() == "Enter" {
                // Handle enter key
            }
        },
        
        // Focus events
        onfocus: move |event: FocusEvent| {
            println!("Element focused");
        },
        
        // Input events
        oninput: move |event: FormEvent| {
            println!("Input value: {}", event.value());
        },
        
        // Form events
        onsubmit: move |event: FormEvent| {
            event.prevent_default();
            println!("Form submitted");
        },
    }
}
```

### Event Propagation
```rust
rsx! {
    div {
        onclick: move |_| println!("Outer div clicked"),
        
        button {
            onclick: move |event| {
                event.stop_propagation(); // Prevent bubbling
                println!("Button clicked - won't bubble to div");
            },
            "Click me"
        }
    }
}
```

## Async Event Handlers

### Basic Async Handlers
```rust
#[component]
fn AsyncButton() -> Element {
    let data = use_signal(|| String::new());
    
    rsx! {
        div {
            button {
                onclick: move |_| async move {
                    let result = fetch_data().await;
                    data.set(result);
                },
                "Fetch Data"
            }
            p { "{data}" }
        }
    }
}
```

### Async Handlers with Loading States
```rust
#[component]
fn AsyncButtonWithLoading() -> Element {
    let data = use_signal(|| String::new());
    let loading = use_signal(|| false);
    
    let handle_click = move |_| async move {
        loading.set(true);
        
        match fetch_data().await {
            Ok(result) => data.set(result),
            Err(e) => data.set(format!("Error: {}", e)),
        }
        
        loading.set(false);
    };
    
    rsx! {
        div {
            button {
                disabled: loading(),
                onclick: handle_click,
                if loading() { "Loading..." } else { "Fetch Data" }
            }
            p { "{data}" }
        }
    }
}
```

### Error Handling in Async Handlers
```rust
#[component]
fn AsyncWithErrorHandling() -> Element {
    let result = use_signal(|| None::<Result<String, String>>);
    
    let handle_action = move |_| async move {
        result.set(None); // Reset state
        
        match risky_async_operation().await {
            Ok(data) => result.set(Some(Ok(data))),
            Err(e) => result.set(Some(Err(e.to_string()))),
        }
    };
    
    rsx! {
        div {
            button { onclick: handle_action, "Perform Action" }
            
            match result() {
                Some(Ok(data)) => rsx! {
                    div { class: "success", "Success: {data}" }
                },
                Some(Err(error)) => rsx! {
                    div { class: "error", "Error: {error}" }
                },
                None => rsx! {
                    div { "Click the button to start" }
                }
            }
        }
    }
}
```

## Form Handling Patterns

### Controlled Inputs
```rust
#[component]
fn ControlledForm() -> Element {
    let name = use_signal(String::new);
    let email = use_signal(String::new);
    
    let handle_submit = move |event: FormEvent| {
        event.prevent_default();
        println!("Submitted: {} - {}", name(), email());
    };
    
    rsx! {
        form { onsubmit: handle_submit,
            input {
                r#type: "text",
                placeholder: "Name",
                value: name(),
                oninput: move |event| name.set(event.value())
            }
            input {
                r#type: "email",
                placeholder: "Email",
                value: email(),
                oninput: move |event| email.set(event.value())
            }
            button { r#type: "submit", "Submit" }
        }
    }
}
```

### Uncontrolled Inputs with Refs
```rust
use dioxus::html::input_data::keyboard_types::Key;

#[component]
fn UncontrolledForm() -> Element {
    let name_ref = use_signal(|| None::<MountedData>);
    let email_ref = use_signal(|| None::<MountedData>);
    
    let handle_submit = move |event: FormEvent| {
        event.prevent_default();
        
        if let (Some(name_el), Some(email_el)) = (name_ref(), email_ref()) {
            let name_value = name_el.get_value().unwrap_or_default();
            let email_value = email_el.get_value().unwrap_or_default();
            println!("Submitted: {} - {}", name_value, email_value);
        }
    };
    
    rsx! {
        form { onsubmit: handle_submit,
            input {
                r#type: "text",
                placeholder: "Name",
                onmounted: move |data| name_ref.set(Some(data))
            }
            input {
                r#type: "email",
                placeholder: "Email",
                onmounted: move |data| email_ref.set(Some(data))
            }
            button { r#type: "submit", "Submit" }
        }
    }
}
```

### Form Validation
```rust
#[derive(Clone, Default)]
struct FormErrors {
    name: Option<String>,
    email: Option<String>,
}

#[component]
fn ValidatedForm() -> Element {
    let name = use_signal(String::new);
    let email = use_signal(String::new);
    
    let errors = use_memo(move || {
        let mut errors = FormErrors::default();
        
        if name().trim().is_empty() {
            errors.name = Some("Name is required".to_string());
        }
        
        if !email().contains('@') {
            errors.email = Some("Invalid email format".to_string());
        }
        
        errors
    });
    
    let is_valid = use_memo(move || {
        let errs = errors();
        errs.name.is_none() && errs.email.is_none()
    });
    
    let handle_submit = move |event: FormEvent| {
        event.prevent_default();
        if is_valid() {
            println!("Valid form submitted: {} - {}", name(), email());
        }
    };
    
    rsx! {
        form { onsubmit: handle_submit,
            div {
                input {
                    r#type: "text",
                    placeholder: "Name",
                    value: name(),
                    oninput: move |event| name.set(event.value()),
                    class: if errors().name.is_some() { "error" } else { "" }
                }
                if let Some(error) = &errors().name {
                    span { class: "error-message", "{error}" }
                }
            }
            
            div {
                input {
                    r#type: "email",
                    placeholder: "Email",
                    value: email(),
                    oninput: move |event| email.set(event.value()),
                    class: if errors().email.is_some() { "error" } else { "" }
                }
                if let Some(error) = &errors().email {
                    span { class: "error-message", "{error}" }
                }
            }
            
            button {
                r#type: "submit",
                disabled: !is_valid(),
                "Submit"
            }
        }
    }
}
```

## Advanced Async Patterns

### Concurrent Async Operations
```rust
#[component]
fn ConcurrentOperations() -> Element {
    let results = use_signal(|| Vec::<String>::new());
    let loading = use_signal(|| false);
    
    let run_concurrent = move |_| async move {
        loading.set(true);
        results.set(Vec::new());
        
        // Run multiple async operations concurrently
        let futures = vec![
            fetch_data_from_api_1(),
            fetch_data_from_api_2(),
            fetch_data_from_api_3(),
        ];
        
        let results_vec = futures_util::future::join_all(futures).await;
        
        let processed_results: Vec<String> = results_vec
            .into_iter()
            .enumerate()
            .map(|(i, result)| match result {
                Ok(data) => format!("API {}: {}", i + 1, data),
                Err(e) => format!("API {} failed: {}", i + 1, e),
            })
            .collect();
        
        results.set(processed_results);
        loading.set(false);
    };
    
    rsx! {
        div {
            button {
                disabled: loading(),
                onclick: run_concurrent,
                if loading() { "Loading..." } else { "Run Concurrent Operations" }
            }
            
            ul {
                for result in results() {
                    li { "{result}" }
                }
            }
        }
    }
}
```

### Debounced Input
```rust
use std::time::Duration;

#[component]
fn DebouncedSearch() -> Element {
    let query = use_signal(String::new);
    let search_results = use_signal(|| Vec::<String>::new());
    let loading = use_signal(|| false);
    
    // Debounced search effect
    use_effect(move || {
        let current_query = query();
        
        if current_query.trim().is_empty() {
            search_results.set(Vec::new());
            return;
        }
        
        // Spawn a task with a delay
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            
            // Check if query is still the same (basic debouncing)
            if query() == current_query {
                loading.set(true);
                
                match search_api(&current_query).await {
                    Ok(results) => search_results.set(results),
                    Err(_) => search_results.set(vec!["Error searching".to_string()]),
                }
                
                loading.set(false);
            }
        });
    });
    
    rsx! {
        div {
            input {
                r#type: "text",
                placeholder: "Search...",
                value: query(),
                oninput: move |event| query.set(event.value())
            }
            
            if loading() {
                div { "Searching..." }
            }
            
            ul {
                for result in search_results() {
                    li { "{result}" }
                }
            }
        }
    }
}
```

### Infinite Scroll
```rust
#[component]
fn InfiniteScroll() -> Element {
    let items = use_signal(|| Vec::<String>::new());
    let loading = use_signal(|| false);
    let page = use_signal(|| 0);
    
    // Load initial data
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let initial_items = fetch_page(0).await.unwrap_or_default();
            items.set(initial_items);
            page.set(1);
            loading.set(false);
        });
    });
    
    let load_more = move |_| async move {
        if loading() { return; }
        
        loading.set(true);
        let current_page = page();
        
        match fetch_page(current_page).await {
            Ok(new_items) => {
                items.with_mut(|items| items.extend(new_items));
                page.set(current_page + 1);
            }
            Err(_) => {
                // Handle error
            }
        }
        
        loading.set(false);
    };
    
    rsx! {
        div {
            ul {
                for item in items() {
                    li { "{item}" }
                }
            }
            
            if loading() {
                div { "Loading more..." }
            } else {
                button { onclick: load_more, "Load More" }
            }
        }
    }
}
```

## WebSocket Integration
```rust
use futures_util::{SinkExt, StreamExt};

#[component]
fn WebSocketChat() -> Element {
    let messages = use_signal(|| Vec::<String>::new());
    let input_value = use_signal(String::new);
    let websocket = use_signal(|| None::<WebSocketConnection>);
    
    // Connect to WebSocket on mount
    use_effect(|| {
        spawn(async move {
            match connect_websocket("ws://localhost:8080/chat").await {
                Ok(ws) => {
                    websocket.set(Some(ws.clone()));
                    
                    // Listen for incoming messages
                    let mut receiver = ws.receiver;
                    while let Some(message) = receiver.next().await {
                        if let Ok(text) = message {
                            messages.with_mut(|msgs| msgs.push(text));
                        }
                    }
                }
                Err(e) => {
                    messages.with_mut(|msgs| msgs.push(format!("Connection error: {}", e)));
                }
            }
        });
    });
    
    let send_message = move |_| async move {
        let message = input_value();
        if !message.trim().is_empty() {
            if let Some(ws) = websocket() {
                if ws.send(message.clone()).await.is_ok() {
                    input_value.set(String::new());
                    messages.with_mut(|msgs| msgs.push(format!("You: {}", message)));
                }
            }
        }
    };
    
    rsx! {
        div { class: "chat-container",
            div { class: "messages",
                for message in messages() {
                    div { class: "message", "{message}" }
                }
            }
            
            div { class: "input-area",
                input {
                    r#type: "text",
                    value: input_value(),
                    oninput: move |event| input_value.set(event.value()),
                    onkeypress: move |event| {
                        if event.key() == "Enter" {
                            spawn(send_message(()));
                        }
                    }
                }
                button { onclick: send_message, "Send" }
            }
        }
    }
}
```

## Event Handler Performance Tips

### Avoiding Unnecessary Closures
```rust
// ❌ Bad: Creates new closure on every render
#[component]
fn BadExample(items: Vec<String>) -> Element {
    rsx! {
        ul {
            for (i, item) in items.iter().enumerate() {
                li {
                    onclick: move |_| {
                        // This creates a new closure for each item on every render
                        handle_click(i);
                    },
                    "{item}"
                }
            }
        }
    }
}

// ✅ Good: Use keys and stable identifiers
#[component] 
fn GoodExample(items: Vec<Item>) -> Element {
    rsx! {
        ul {
            for item in items {
                ListItem { 
                    key: "{item.id}",
                    item: item,
                    onclick: handle_click
                }
            }
        }
    }
}

#[component]
fn ListItem(item: Item, onclick: EventHandler<usize>) -> Element {
    rsx! {
        li {
            onclick: move |_| onclick.call(item.id),
            "{item.text}"
        }
    }
}
```

### Event Handler Cleanup
```rust
#[component]
fn WithCleanup() -> Element {
    use_effect(|| {
        let handle_keydown = move |event: KeyboardEvent| {
            // Handle global keydown
        };
        
        // Add global event listener
        document().add_event_listener("keydown", &handle_keydown);
        
        // Cleanup function
        move || {
            document().remove_event_listener("keydown", &handle_keydown);
        }
    });
    
    rsx! { div { "Component with global event listener" } }
}
```

## Common Async Pitfalls and Solutions

### Avoiding Race Conditions
```rust
#[component]
fn SearchWithCancellation() -> Element {
    let query = use_signal(String::new);
    let results = use_signal(|| Vec::<String>::new());
    let request_id = use_signal(|| 0u32);
    
    use_effect(move || {
        let current_query = query();
        let current_id = request_id() + 1;
        request_id.set(current_id);
        
        if current_query.trim().is_empty() {
            results.set(Vec::new());
            return;
        }
        
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            
            // Only update if this is still the latest request
            if request_id() == current_id {
                match search_api(&current_query).await {
                    Ok(search_results) => {
                        // Double-check we're still the latest request
                        if request_id() == current_id {
                            results.set(search_results);
                        }
                    }
                    Err(_) => {
                        if request_id() == current_id {
                            results.set(vec!["Error occurred".to_string()]);
                        }
                    }
                }
            }
        });
    });
    
    rsx! {
        div {
            input {
                value: query(),
                oninput: move |event| query.set(event.value())
            }
            ul {
                for result in results() {
                    li { "{result}" }
                }
            }
        }
    }
}
```

### Memory Management in Async Handlers
```rust
// ✅ Good: Clone values before async operations
let handle_click = move |_| async move {
    let user_id = user_id(); // Clone the value
    let user_name = user_name(); // Clone the value
    
    // Safe to use across await points
    let result = fetch_user_data(user_id).await;
    update_ui_with_user_data(user_name, result);
};

// ❌ Bad: Don't hold signal reads across await points
let bad_handler = move |_| async move {
    let user_ref = user_id.read(); // Starts borrow
    let result = fetch_user_data(*user_ref).await; // Panic! Borrow held across await
};
```