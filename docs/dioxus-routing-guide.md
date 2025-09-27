# Dioxus Routing Guide

## Overview
Dioxus Router provides type-safe routing for multi-page applications. It works across web, desktop, and mobile platforms with the same API.

## Basic Setup
Add the router dependency:
```bash
cargo add dioxus-router
```

## Defining Routes
Create an enum representing your app's routes:

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(NavBar)]
        #[route("/")]
        Home {},
        #[route("/favorites")]
        Favorites {},
}
```

## Route Components
Implement components for each route:

```rust
#[component]
fn Home() -> Element {
    rsx! {
        div { id: "home",
            h1 { "Welcome to HotDog!" }
            p { "Find your favorite dog images" }
        }
    }
}

#[component] 
fn Favorites() -> Element {
    rsx! {
        div { id: "favorites",
            h1 { "Your Favorite Dogs" }
            // Favorite dogs list here
        }
    }
}
```

## Layout Components
Layouts wrap multiple routes with shared UI:

```rust
#[component]
fn NavBar() -> Element {
    rsx! {
        div { id: "app",
            nav { id: "navbar",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Favorites {}, "Favorites" }
            }
            main {
                Outlet::<Route> {}
            }
        }
    }
}
```

## Setting up the Router
Use the router in your app:

```rust
#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
```

## Navigation
### Declarative Links
```rust
rsx! {
    Link { to: Route::Favorites {}, "View Favorites" }
}
```

### Programmatic Navigation
```rust
let navigator = use_navigator();

rsx! {
    button {
        onclick: move |_| navigator.push(Route::Favorites {}),
        "Go to Favorites"
    }
}
```

## Route Parameters
Define routes with parameters:

```rust
#[derive(Routable, Clone)]
enum Route {
    #[route("/user/:id")]
    User { id: usize },
    #[route("/post/:id/:slug")]  
    Post { id: usize, slug: String },
}

#[component]
fn User(id: usize) -> Element {
    rsx! {
        h1 { "User {id}" }
    }
}
```

## Query Parameters
Access query parameters:

```rust
#[component]
fn Search() -> Element {
    let query = use_route::<Route>()
        .query::<String>("q")
        .unwrap_or_default();
    
    rsx! {
        h1 { "Search results for: {query}" }
    }
}
```

## Nested Routing
```rust
#[derive(Routable, Clone)]
enum Route {
    #[layout(AppLayout)]
        #[layout(DashboardLayout)]
            #[route("/dashboard")]
            Dashboard {},
            #[route("/dashboard/settings")]
            Settings {},
        #[end_layout]
    #[end_layout]
}
```

## Route Guards
Protect routes with guards:

```rust
#[component]
fn ProtectedRoute() -> Element {
    let is_authenticated = use_context::<AuthState>().is_logged_in;
    
    if !is_authenticated {
        return rsx! { Redirect { to: Route::Login {} } };
    }
    
    rsx! {
        div { "Protected content" }
    }
}
```

## Best Practices
- Use layouts for shared UI elements
- Keep route components focused and small
- Use type-safe route parameters
- Implement loading states for async route data
- Consider route-based code splitting
- Test navigation flows thoroughly

Tags: routing, navigation, links, layouts, parameters, guards, multi-page, spa