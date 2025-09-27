# Dioxus RSX Guide - Building UIs

## Overview
RSX is Dioxus's templating syntax for building user interfaces. It's similar to JSX but designed specifically for Rust with better tooling support.

## Text Nodes
```rust
rsx! {
    "Hello world"
}
```

Formatted text with expressions:
```rust
let user = use_signal(|| User { name: "Dioxus".to_string() });
rsx! {
    "Hello {user.read().name}"
}
```

## Elements
Basic HTML elements:
```rust
rsx! {
    input {}
    div {
        h1 { "Title" }
        p { "Content" }
    }
}
```

## Attributes
```rust
rsx! {
    input {
        class: "my-input",
        id: "user-input",
        r#type: "text",
        placeholder: "Enter text..."
    }
}
```

## Conditional Attributes
```rust
rsx! {
    input {
        class: if highlighted { "input-highlighted" },
        disabled: is_disabled
    }
}
```

## Event Listeners
```rust
rsx! {
    button {
        onclick: move |event| {
            println!("Button clicked!");
        },
        "Click me"
    }
}
```

## Children and Loops
```rust
let items = vec!["Hello", "Dioxus"];
rsx! {
    ul {
        for item in items.iter() {
            li { key: "{item}", "{item}" }
        }
    }
}
```

## Conditional Rendering
```rust
let logged_in = use_signal(|| false);
rsx! {
    div {
        if logged_in() {
            "You are logged in"
        } else {
            "You are not logged in"
        }
    }
}
```

## Why RSX vs HTML?
- Token coloring and code-folding without additional tooling
- Faster to type with auto-closed curly braces
- Works in non-HTML contexts
- Pure Rust syntax compatibility

Tags: rsx, ui, templating, syntax, elements, events, conditional-rendering