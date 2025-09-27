# Dioxus Patterns and Best Practices

## Component Design Patterns

### Container vs Presentational Components

#### Container Components (Smart)
Handle state and business logic:
```rust
#[component]
fn UserProfileContainer(user_id: i32) -> Element {
    let user_data = use_resource(move || async move {
        fetch_user(user_id).await
    });
    
    match &*user_data.read_unchecked() {
        Some(Ok(user)) => rsx! {
            UserProfile { user: user.clone() }
        },
        Some(Err(_)) => rsx! { "Error loading user" },
        None => rsx! { "Loading..." }
    }
}
```

#### Presentational Components (Dumb)
Only handle display:
```rust
#[component]
fn UserProfile(user: User) -> Element {
    rsx! {
        div { class: "user-profile",
            img { src: user.avatar, alt: "User avatar" }
            h2 { "{user.name}" }
            p { "{user.email}" }
        }
    }
}
```

### Compound Components Pattern
Create related components that work together:
```rust
#[component]
fn Card(children: Element) -> Element {
    rsx! {
        div { class: "card", {children} }
    }
}

#[component]
fn CardHeader(children: Element) -> Element {
    rsx! {
        div { class: "card-header", {children} }
    }
}

#[component]
fn CardBody(children: Element) -> Element {
    rsx! {
        div { class: "card-body", {children} }
    }
}

// Usage
rsx! {
    Card {
        CardHeader { "Profile" }
        CardBody { "User information..." }
    }
}
```

### Higher-Order Components (HOCs)
Wrap components with additional functionality:
```rust
#[component]
fn WithLoading<T: 'static + Clone>(
    loading: bool,
    children: Element,
) -> Element {
    if loading {
        rsx! { div { class: "spinner", "Loading..." } }
    } else {
        children
    }
}

// Usage
rsx! {
    WithLoading { loading: is_loading,
        MyComponent { data: user_data }
    }
}
```

## State Management Patterns

### State Lifting Pattern
Move state up to the closest common ancestor:
```rust
#[component]
fn Parent() -> Element {
    let shared_state = use_signal(|| 0);
    
    rsx! {
        div {
            ChildA { count: shared_state }
            ChildB { count: shared_state }
        }
    }
}

#[component]
fn ChildA(count: Signal<i32>) -> Element {
    rsx! {
        button {
            onclick: move |_| count += 1,
            "Increment: {count}"
        }
    }
}
```

### State Co-location Pattern
Keep state as close to where it's used as possible:
```rust
#[component]
fn TodoList() -> Element {
    let todos = use_signal(Vec::<Todo>::new);
    
    rsx! {
        div {
            TodoInput { todos } // Pass signal for adding
            TodoItems { todos } // Pass signal for reading
        }
    }
}
```

### Derived State Pattern
Use memos for computed values:
```rust
#[component]
fn ShoppingCart() -> Element {
    let items = use_signal(Vec::<CartItem>::new);
    
    // Derived state
    let total_price = use_memo(move || {
        items.read().iter()
            .map(|item| item.price * item.quantity)
            .sum::<f64>()
    });
    
    let item_count = use_memo(move || {
        items.read().iter()
            .map(|item| item.quantity)
            .sum::<u32>()
    });
    
    rsx! {
        div {
            "Items: {item_count}, Total: ${total_price:.2}"
            // ... rest of cart
        }
    }
}
```

## Event Handling Patterns

### Event Handler Composition
```rust
fn create_click_handler(
    on_click: Option<EventHandler<MouseEvent>>,
    additional_action: impl Fn() + 'static,
) -> EventHandler<MouseEvent> {
    EventHandler::new(move |event| {
        additional_action();
        if let Some(handler) = &on_click {
            handler.call(event);
        }
    })
}

#[component]
fn Button(
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let click_handler = create_click_handler(onclick, || {
        println!("Button clicked!");
    });
    
    rsx! {
        button { onclick: click_handler, {children} }
    }
}
```

### Async Event Handlers with Error Handling
```rust
#[component]
fn AsyncButton() -> Element {
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<String>);
    
    let handle_click = move |_| async move {
        loading.set(true);
        error.set(None);
        
        match async_operation().await {
            Ok(result) => {
                // Handle success
            }
            Err(e) => {
                error.set(Some(e.to_string()));
            }
        }
        
        loading.set(false);
    };
    
    rsx! {
        div {
            button {
                disabled: loading(),
                onclick: handle_click,
                if loading() { "Loading..." } else { "Click me" }
            }
            if let Some(err) = error() {
                p { class: "error", "Error: {err}" }
            }
        }
    }
}
```

## Form Handling Patterns

### Controlled Components
```rust
#[component]
fn LoginForm() -> Element {
    let username = use_signal(String::new);
    let password = use_signal(String::new);
    
    let handle_submit = move |_| {
        // Form submission logic
        println!("Login: {} / {}", username(), password());
    };
    
    rsx! {
        form { onsubmit: handle_submit,
            input {
                r#type: "text",
                value: username(),
                oninput: move |e| username.set(e.value()),
                placeholder: "Username"
            }
            input {
                r#type: "password",
                value: password(),
                oninput: move |e| password.set(e.value()),
                placeholder: "Password"
            }
            button { r#type: "submit", "Login" }
        }
    }
}
```

### Form Validation Pattern
```rust
#[derive(Clone)]
struct FormData {
    email: String,
    age: u32,
}

#[derive(Clone)]
struct ValidationError {
    email: Option<String>,
    age: Option<String>,
}

#[component]
fn ValidatedForm() -> Element {
    let form_data = use_signal(|| FormData {
        email: String::new(),
        age: 0,
    });
    
    let errors = use_memo(move || {
        let data = form_data.read();
        ValidationError {
            email: if data.email.contains('@') {
                None
            } else {
                Some("Invalid email".to_string())
            },
            age: if data.age >= 18 {
                None
            } else {
                Some("Must be 18 or older".to_string())
            },
        }
    });
    
    let is_valid = use_memo(move || {
        let err = errors.read();
        err.email.is_none() && err.age.is_none()
    });
    
    rsx! {
        form {
            div {
                input {
                    r#type: "email",
                    value: form_data.read().email.clone(),
                    oninput: move |e| {
                        form_data.with_mut(|data| data.email = e.value());
                    }
                }
                if let Some(error) = &errors.read().email {
                    span { class: "error", "{error}" }
                }
            }
            
            div {
                input {
                    r#type: "number",
                    value: form_data.read().age.to_string(),
                    oninput: move |e| {
                        if let Ok(age) = e.value().parse::<u32>() {
                            form_data.with_mut(|data| data.age = age);
                        }
                    }
                }
                if let Some(error) = &errors.read().age {
                    span { class: "error", "{error}" }
                }
            }
            
            button {
                disabled: !is_valid(),
                "Submit"
            }
        }
    }
}
```

## Performance Optimization Patterns

### Memo for Expensive Computations
```rust
#[component]
fn DataVisualization(data: ReadOnlySignal<Vec<DataPoint>>) -> Element {
    // Expensive computation only runs when data changes
    let processed_data = use_memo(move || {
        expensive_data_processing(data())
    });
    
    rsx! {
        div { "Chart data: {processed_data:?}" }
    }
}
```

### Selective Re-rendering with Keys
```rust
#[component]
fn TodoList(todos: ReadOnlySignal<Vec<Todo>>) -> Element {
    rsx! {
        ul {
            for todo in todos() {
                TodoItem {
                    key: "{todo.id}", // Prevents unnecessary re-renders
                    todo: todo
                }
            }
        }
    }
}
```

### Lazy Loading Pattern
```rust
#[component]
fn LazyComponent(should_load: bool) -> Element {
    let content = use_resource(move || async move {
        if should_load {
            Some(load_expensive_data().await)
        } else {
            None
        }
    });
    
    match &*content.read_unchecked() {
        Some(Some(data)) => rsx! { ExpensiveComponent { data: data.clone() } },
        Some(None) => rsx! { "Click to load" },
        None => rsx! { "Loading..." }
    }
}
```

## Error Handling Patterns

### Error Boundary Pattern
```rust
#[component]
fn ErrorBoundary(children: Element) -> Element {
    let error = use_signal(|| None::<String>);
    
    if let Some(err) = error() {
        rsx! {
            div { class: "error-boundary",
                h2 { "Something went wrong!" }
                p { "{err}" }
                button {
                    onclick: move |_| error.set(None),
                    "Try again"
                }
            }
        }
    } else {
        // In a real implementation, you'd wrap children with error catching
        children
    }
}
```

### Result Handling Pattern
```rust
#[component]
fn DataLoader(id: i32) -> Element {
    let data = use_resource(move || async move {
        fetch_data(id).await
    });
    
    match &*data.read_unchecked() {
        Some(Ok(value)) => rsx! {
            DataDisplay { data: value.clone() }
        },
        Some(Err(e)) => rsx! {
            div { class: "error",
                "Failed to load data: {e}"
                button {
                    onclick: move |_| data.restart(),
                    "Retry"
                }
            }
        },
        None => rsx! {
            div { class: "loading", "Loading..." }
        }
    }
}
```

## Testing Patterns

### Component Testing Setup
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;
    
    #[test]
    fn test_counter_component() {
        let mut dom = VirtualDom::new(|| rsx! {
            Counter { initial_count: 5 }
        });
        
        let _ = dom.rebuild();
        
        // Test initial render
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("5"));
        
        // Simulate click event
        // ... test event handling
    }
}
```

## Architecture Patterns

### Feature-Based Organization
```
src/
├── components/
│   ├── ui/           # Reusable UI components
│   └── layout/       # Layout components
├── features/
│   ├── auth/
│   │   ├── components.rs
│   │   ├── state.rs
│   │   └── mod.rs
│   └── dashboard/
│       ├── components.rs
│       ├── state.rs
│       └── mod.rs
├── services/         # API calls, business logic
├── utils/           # Helper functions
└── main.rs
```

### Dependency Injection Pattern
```rust
#[derive(Clone)]
struct Services {
    api_client: ApiClient,
    auth_service: AuthService,
}

#[component]
fn App() -> Element {
    use_context_provider(|| Services {
        api_client: ApiClient::new(),
        auth_service: AuthService::new(),
    });
    
    rsx! {
        Router::<Route> {}
    }
}
```

## Key Best Practices Summary

1. **Keep components small and focused** - Single responsibility principle
2. **Use descriptive prop names** - Make component APIs clear
3. **Prefer composition over inheritance** - Use children and slots
4. **Co-locate related state** - Keep state close to where it's used
5. **Use memos for expensive computations** - Optimize performance
6. **Handle loading and error states** - Provide good UX
7. **Use keys for dynamic lists** - Prevent unnecessary re-renders
8. **Test components in isolation** - Write focused unit tests
9. **Follow consistent naming conventions** - Use clear, descriptive names
10. **Document complex components** - Add doc comments for APIs