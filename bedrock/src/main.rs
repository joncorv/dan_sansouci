use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

mod basic_nav;
mod dummy_data;
mod logging;

#[tokio::main]
async fn main() {
    dummy_data::parse_args_create_dummy_data();

    dotenvy::dotenv().ok();
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let tcp_addr_string = std::env::var("TCP_ADDR").unwrap();

    logging::create_tracing();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let app = Router::new()
        .merge(basic_nav::build_routes_basic_nav())
        .layer(cors)
        .layer(logging::create_trace_layer())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&tcp_addr_string)
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
