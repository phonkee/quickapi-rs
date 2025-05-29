use axum::routing::get;
use quickapi::ListView;

#[tokio::main]
async fn main() {
    let mut router: axum::Router<()> = axum::Router::new();

    router = router.route(
        "/api/user",
        get::<ListView<entity::User>, (), ()>(ListView::default()),
    );

    // prepare listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4148")
        .await
        .unwrap();

    // Serve the router with the state
    axum::serve(listener, router).await.unwrap();
}
