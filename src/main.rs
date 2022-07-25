use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
    Json
};
use reqwest::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use league_a_lot::Client;

async fn get_match_times(
    Path(summoner_name): Path<String>,
    riot: Arc<Client>,
) -> impl IntoResponse {
    let times = riot.get_match_times(&summoner_name).await.unwrap();
    Json(times)
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Whoops.. something went wrong...",
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "league_a_lot=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let riot = Arc::new(Client::new());

    let app = Router::new()
        .route(
            "/matches/:summoner",
            get({
                let shared_riot = Arc::clone(&riot);
                move |path| get_match_times(path, Arc::clone(&shared_riot))
            }),
        )
        .fallback(get_service(ServeDir::new("./static")).handle_error(handle_error))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
