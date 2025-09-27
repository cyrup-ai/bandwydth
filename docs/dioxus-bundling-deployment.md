# Dioxus Bundling and Deployment Guide

## Overview
Dioxus supports building and bundling apps for web, desktop, and mobile platforms using the `dx` CLI tool.

## Platform Support
- **Web**: WASM-based web applications
- **Desktop**: Native apps using Tauri (Windows, macOS, Linux)
- **Mobile**: iOS and Android applications
- **Server**: Server-side rendering and fullstack apps

## Testing Platforms

### iOS Testing
Requirements:
- macOS development machine
- Xcode installed
- iOS SDK and Simulator
- Rust iOS toolchains: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`

```bash
dx serve --platform ios
```

### Android Testing  
Requirements:
- Android SDK and NDK
- Java Development Kit
- Android emulator or device
- Rust Android toolchains

```bash
dx serve --platform android
```

### Desktop Testing
```bash
dx serve --platform desktop
```

## Building for Production

### Web Build
```bash
dx build --release --platform web
```
Output: `dist/` directory with HTML, CSS, JS, and WASM files

### Desktop Bundle
```bash  
dx bundle --platform desktop --release
```
Creates native installers:
- **Windows**: `.msi` and `.exe` files
- **macOS**: `.dmg` and `.app` bundle  
- **Linux**: `.deb`, `.rpm`, and `.AppImage`

### Mobile Bundle
```bash
# iOS
dx bundle --platform ios --release

# Android  
dx bundle --platform android --release
```

## Customizing Builds

### Dioxus.toml Configuration
```toml
[application]
name = "MyApp"
default_platform = "web"

[web.app]
title = "My Dioxus App"
base_path = "/"

[desktop.app]  
name = "MyApp"
identifier = "com.example.myapp"

[mobile.app]
name = "MyApp"
identifier = "com.example.myapp"
```

### Bundle Customization
```toml
[bundle]
identifier = "com.yourcompany.yourapp"
publisher = "Your Company"
icon = ["icons/icon.png"]
resources = ["assets/*"]
copyright = "Copyright (c) 2024 Your Company"
category = "DeveloperTool"
short_description = "A short description"
long_description = "A longer description of your app"
```

## Deployment Strategies

### Web Deployment
Static hosting options:
- **Netlify**: Easy deployment with git integration
- **Vercel**: Optimized for web frameworks
- **GitHub Pages**: Free hosting for public repositories
- **AWS S3 + CloudFront**: Scalable CDN distribution

### Desktop Distribution
- **GitHub Releases**: Host installers on GitHub
- **App stores**: Windows Store, Mac App Store, Snap Store
- **Direct download**: Host files on your website
- **Auto-updaters**: Integrate with Tauri's updater

### Mobile Distribution
- **Apple App Store**: iOS distribution (requires developer account)
- **Google Play Store**: Android distribution  
- **Direct APK**: Sideload Android apps
- **TestFlight**: iOS beta testing

## Fullstack Deployment
For apps with server functions:

### Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN dx build --release --platform server

FROM debian:bookworm-slim
COPY --from=builder /app/dist /app
EXPOSE 8080
CMD ["/app/server"]
```

### Platform Recommendations
- **Fly.io**: Excellent Rust support with global edge
- **Railway**: Simple deployment with automatic scaling
- **AWS/GCP/Azure**: Full-featured cloud platforms
- **DigitalOcean**: Simple VPS hosting

## Performance Optimization
- Use `--release` flag for production builds
- Enable WASM optimizations in `Cargo.toml`
- Optimize images and assets
- Consider code splitting for large apps
- Use CDN for static assets

## JSON Output Mode
Automate builds with JSON output:
```bash
dx bundle --platform desktop --json-output | tail -1 | jq -r '.json | fromjson | .BundleOutput.bundles[]'
```

Tags: bundling, deployment, platforms, ios, android, desktop, web, docker, ci-cd, distribution