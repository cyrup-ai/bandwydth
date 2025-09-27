# Dioxus Fullstack Backend Guide

## Overview
Dioxus is a fullstack framework supporting Server Functions for seamless frontend-backend integration with type-safe RPC-style communication.

## Enabling Fullstack
Update `Cargo.toml`:

```toml
[dependencies]
dioxus = { version = "0.6.0", features = ["fullstack"] }

[features]
default = [] # Remove default web target
web = ["dioxus/web"]
desktop = ["dioxus/desktop"] 
mobile = ["dioxus/mobile"]
server = ["dioxus/server"] # Add server target
```

## Server Functions
Define functions that run on the server but are called from the client:

```rust
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[server(endpoint = "save_dog")]
pub async fn save_dog(image: String) -> Result<(), ServerFnError> {
    println!("Saving dog image: {}", image);
    
    // Server-only code
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("dogs.txt")
        .unwrap();
    
    file.write_fmt(format_args!("{image}\n"));
    Ok(())
}
```

## Client/Server Split
Code inside server functions only runs on the server:

```rust
#[server]
pub async fn get_server_time() -> Result<String, ServerFnError> {
    // This runs on the server
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Ok(format!("Server time: {}", time))
}
```

## Calling Server Functions
Call server functions from client code like regular async functions:

```rust
#[component]
fn App() -> Element {
    let mut status = use_signal(|| "Ready".to_string());
    
    rsx! {
        button {
            onclick: move |_| async move {
                match save_dog("https://example.com/dog.jpg".to_string()).await {
                    Ok(_) => status.set("Saved!".to_string()),
                    Err(e) => status.set(format!("Error: {}", e))
                }
            },
            "Save Dog"
        }
        p { "Status: {status}" }
    }
}
```

## Managing Dependencies  
Use feature gates for server-only dependencies:

```toml
[dependencies]
# Client and server
serde = { version = "1.0", features = ["derive"] }

# Server only
sqlx = { version = "0.7", optional = true }

[features]
server = ["dioxus/server", "sqlx"]
```

```rust
#[server]
pub async fn save_to_database(data: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        // Use sqlx here - only compiles on server
        use sqlx::SqlitePool;
        // Database operations...
    }
    Ok(())
}
```

## Running the Fullstack App
- Development: `dx serve` (runs both client and server)
- Server only: `dx serve --platform server`  
- Client only: `dx serve --platform web`

## Best Practices
- Keep server functions focused and single-purpose
- Use proper error handling with `ServerFnError`
- Leverage feature gates for server-only dependencies
- Server functions work with SSR and streaming
- Consider using database connection pools

Tags: fullstack, server-functions, rpc, client-server, dependencies, async, error-handling