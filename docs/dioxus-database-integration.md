# Dioxus Database Integration Guide

## Overview
Dioxus works with any Rust database library. Popular options include SQLite (via rusqlite/sqlx), PostgreSQL, MySQL, and modern solutions like SurrealDB.

## Database Options

### SQLite
- **rusqlite**: Synchronous, simple API
- **sqlx**: Async, compile-time checked queries
- Best for: Single-user apps, prototypes, local storage

### PostgreSQL
- **sqlx**: Async with compile-time safety
- **tokio-postgres**: Lower-level async client
- Best for: Multi-user web applications

### MySQL
- **sqlx**: Cross-database compatibility
- **mysql_async**: MySQL-specific async client

### Modern Alternatives
- **SurrealDB**: Multi-model database (document, graph, time-series)
- **MongoDB**: Document database via **mongodb** crate
- **Redis**: In-memory cache via **redis** crate

## SQLite Integration Example

### Dependencies
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
```

### Database Setup
```rust
use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};

#[server]
pub async fn init_database() -> Result<(), ServerFnError> {
    let db_url = "sqlite://dogs.db";
    
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        Sqlite::create_database(db_url).await?;
    }
    
    let pool = SqlitePool::connect(db_url).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dogs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_url TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;
    
    Ok(())
}
```

### Server Functions with Database
```rust
#[server(endpoint = "save_dog")]
pub async fn save_dog(image: String) -> Result<(), ServerFnError> {
    let pool = SqlitePool::connect("sqlite://dogs.db").await?;
    
    sqlx::query("INSERT INTO dogs (image_url) VALUES (?)")
        .bind(&image)
        .execute(&pool)
        .await?;
    
    Ok(())
}

#[server(endpoint = "list_dogs")]
pub async fn list_dogs() -> Result<Vec<(i64, String)>, ServerFnError> {
    let pool = SqlitePool::connect("sqlite://dogs.db").await?;
    
    let dogs = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, image_url FROM dogs ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(dogs)
}
```

## Connection Pooling
```rust
use once_cell::sync::OnceCell;

static DB_POOL: OnceCell<SqlitePool> = OnceCell::new();

pub async fn get_db_pool() -> &'static SqlitePool {
    DB_POOL.get_or_init(|| async {
        SqlitePool::connect("sqlite://dogs.db")
            .await
            .expect("Failed to create pool")
    }).await
}
```

## Database Providers

### SQLite Hosting
- **Turso**: Multi-tenant SQLite with edge replication
- **LiteFS**: Distributed SQLite for Fly.io

### PostgreSQL Hosting  
- **Supabase**: PostgreSQL with dashboard and auth
- **Neon**: Serverless PostgreSQL
- **AWS RDS**: Managed PostgreSQL

### Cloud-Native
- **PlanetScale**: MySQL-compatible with branching
- **FaunaDB**: Serverless, globally distributed
- **Firebase**: Real-time NoSQL database

## Best Practices
- Use connection pooling for performance
- Handle database errors gracefully
- Use migrations for schema changes
- Consider read replicas for heavy read workloads
- Implement proper indexing strategies
- Use transactions for data consistency

Tags: database, sqlite, postgresql, mysql, sqlx, connection-pooling, migrations, server-functions