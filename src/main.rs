use std::net::SocketAddr;

use axum::{
    extract::FromRef,
    http::Response,
    routing::{get_service, post},
    Router,
};
use axum_extra::extract::cookie::Key;
use nori::migrations;

use sea_orm::{ConnectOptions, Database, DbConn};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{error, info, log::LevelFilter};

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
    tracing_subscriber::fmt::init();

    /* Session Management -- Private Cookie Key */
    let key_bytes = hex::decode(std::env::var("KEY").expect("KEY must be set"))
        .expect("KEY must be a hex string");
    let key = Key::from(&key_bytes);

    /* Set sqlx logging to debug */
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut db_opts = ConnectOptions::from(&db_url);
    db_opts.sqlx_logging_level(LevelFilter::Debug);
    let db = Database::connect(db_opts).await?;

    /* Run migrations */
    if let Err(e) = migrations::setup::up(&db).await {
        error!("Error setting up schema: {}", e);
    }

    let state = AppState { db, key };

    /* Serving frontend generated files */
    let serve_static =
        get_service(ServeDir::new("frontend/build")).handle_error(|err| async move {
            Response::builder()
                .status(500)
                .header("Content-Type", "text/html")
                .body(format!("Internal Server Error: {}", err))
                .unwrap()
        });

    let app = Router::new()
        .nest_service("/", serve_static)
        .route("/api/register", post(auth::register))
        .route("/api/login", post(auth::login))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
