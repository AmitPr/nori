use std::net::SocketAddr;

use axum::{
    extract::FromRef,
    http::Response,
    routing::{get, post, get_service},
    Router,
};
use axum_extra::extract::cookie::Key;
use nori::migrations;

use sea_orm::{Database, DbConn};
use tower_http::services::ServeDir;

mod auth;

#[derive(Clone)]
pub struct AppState {
    db: DbConn,
    key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let key_bytes = hex::decode(std::env::var("KEY").expect("KEY must be set"))
        .expect("KEY must be a hex string");
    let key = Key::from(&key_bytes);

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&db_url).await?;

    if let Err(e) = migrations::setup::up(&db).await {
        println!("Error setting up schema: {}", e);
    }

    let state = AppState { db, key };

    let serve_static = get_service(ServeDir::new("frontend/dist")).handle_error(
        |err| async move {
            Response::builder()
                .status(500)
                .header("Content-Type", "text/html")
                .body(format!("Internal Server Error: {}", err))
                .unwrap()
        },
    );

    let app = Router::new()
        .nest_service("/static", serve_static)
        .route(
            "/",
            get(|| async {
                Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body(include_str!("../frontend/login.html").to_owned())
                    .unwrap()
            }),
        )
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
