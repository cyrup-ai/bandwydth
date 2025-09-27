# Dioxus Data Fetching Guide

## Overview
Dioxus doesn't provide built-in data fetching utilities. Use standard Rust HTTP clients like reqwest with async patterns and the `use_resource` hook.

## Adding Dependencies
```bash
cargo add reqwest --features json
cargo add serde --features derive
```

## Defining Response Types
Create structs that match your API responses:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct DogApi {
    message: String,
    status: String,
}
```

## Basic Async Fetching
```rust
async fn fetch_dog_image() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://dog.ceo/api/breeds/image/random")
        .await?
        .json::<DogApi>()
        .await?;
    
    Ok(response.message)
}
```

## Using use_resource for State Management
Resources manage async state and provide loading/error states:

```rust
#[component]
fn DogView() -> Element {
    let mut img_src = use_resource(|| async move {
        reqwest::get("https://dog.ceo/api/breeds/image/random")
            .await
            .unwrap()
            .json::<DogApi>()
            .await
            .unwrap()
            .message
    });

    rsx! {
        div {
            match img_src.value() {
                Some(url) => rsx! { img { src: "{url}" } },
                None => rsx! { "Loading..." }
            }
        }
        button {
            onclick: move |_| img_src.restart(),
            "Fetch New Image"
        }
    }
}
```

## Resource Methods
- `img_src.restart()` - Restart the async operation
- `img_src.value()` - Get current value (Option)
- `img_src.cloned()` - Get cloned value
- `img_src.suspend()?` - Wait for completion (in suspense boundary)

## Error Handling
```rust
let mut resource = use_resource(|| async move {
    match reqwest::get("https://api.example.com/data").await {
        Ok(response) => response.json::<MyData>().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string())
    }
});

rsx! {
    match resource.value() {
        Some(Ok(data)) => rsx! { "Data: {data:?}" },
        Some(Err(error)) => rsx! { "Error: {error}" },
        None => rsx! { "Loading..." }
    }
}
```

## Best Practices
- Use `use_resource` for async operations that affect UI
- Handle loading and error states explicitly  
- Avoid race conditions by using resources instead of manual async
- Consider using `dioxus-query` crate for advanced data fetching
- Resources integrate with Suspense and SSR

Tags: data-fetching, async, reqwest, resources, http, api, loading-states, error-handling