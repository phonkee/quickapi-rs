# QuickAPI

QuickAPI is rust package for building restful APIs for sea ORM models and axum framework.

# Example

Let's suppose we are in example directory. Let's try to explain how to use QuickAPI.

First we need to provide sea orm connection to instantiate QuickAPI instance.
QuickAPI is generic over axum State, so we can use any state we want. In this example we will use ().

```rust
#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // connect sea-orm to the database
    let api = quickapi::new::<()>(
        sea_orm::Database::connect(
            "postgres://user:password@localhost:5432/database",
        )
            .await?,
    );

    // create an axum router
    let router: axum::Router<()> = axum::Router::new();
    
    // add user list endpoint and register it to the router
    let router = api
        .view()
        .list()
        .new::<entity::User>("/api/user/")?
        .register_router(router)?;
    
    Ok(())
}
```

# Author

Peter Vrba <phonkee@phonkee.eu>
