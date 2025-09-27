# Dioxus Assets and Styling Guide

## Overview
Dioxus uses HTML and CSS for styling across all platforms. This provides powerful, familiar styling capabilities for web, desktop, and mobile apps.

## CSS Integration
Use the `asset!()` macro to include CSS files:

```rust
static CSS: Asset = asset!("/assets/main.css");

#[component]  
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
        // Your app content
    }
}
```

## Project Structure
```
├── Cargo.toml
├── assets
│   └── main.css
└── src
    └── main.rs
```

## Hot Reloading
CSS files are automatically reloaded during development with `dx serve`. Changes appear instantly without rebuilding.

## Including Images
Use the `asset!()` macro for images too:

```rust
static LOGO: Asset = asset!("/assets/logo.png");

rsx! {
    img { src: LOGO }
}
```

## Build Optimizations
- Assets are automatically bundled with your app
- Images are optimized for size and format
- CSS is minified in release builds
- Unused CSS can be tree-shaken

## Example CSS Structure
```css
/* Layout */
#app {
    display: flex;
    flex-direction: column;
    height: 100vh;
}

/* Components */
#title {
    text-align: center;
    padding: 20px;
}

#buttons {
    display: flex;
    justify-content: space-evenly;
    padding: 20px;
}

/* Interactive states */
button:hover {
    background-color: #f0f0f0;
}

.favorite-dog:hover button {
    display: block;
}
```

## Cross-Platform Considerations
- CSS works consistently across web, desktop, and mobile
- Use responsive design principles
- Consider touch-friendly sizing for mobile
- Test layouts on different screen sizes

Tags: css, styling, assets, images, hot-reload, responsive, cross-platform