#![allow(dead_code, unused_imports)]
use aws_sdk_s3 as s3;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

// mod basic_nav;
mod cloud_storage;
mod dashboard;
mod datatypes;
// mod dummy_data;
mod logging;

#[derive(Debug, Clone)]
pub struct AppState {
    pool: Pool<Postgres>,
    s3_client: s3::Client,
    s3_bucket_name: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // dummy_data::parse_args_create_dummy_data().await.unwrap();

    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let tcp_addr_string = std::env::var("TCP_ADDR").unwrap();

    logging::create_tracing();

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    let s3_client = cloud_storage::new().await.unwrap();
    let s3_bucket_name = std::env::var("S3_BUCKET_NAME").expect("env var not found: s3 bucket name");

    let app_state = AppState { pool, s3_client, s3_bucket_name };

    let app = Router::new()
        // .merge(basic_nav::build_routes_basic_nav())
        .merge(dashboard::build_routes_dashboard())
        .layer(cors)
        .layer(logging::create_trace_layer())
        .layer(DefaultBodyLimit::max(10_000_000))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&tcp_addr_string).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
