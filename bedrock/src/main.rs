use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
// use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://example_user:example_password@localhost:5432/example_db".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/add", get(add))
        .route("/sub", get(sub))
        .layer(cors)
        // .with_state(shared_state)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
struct PopulationAnnouncement<'a> {
    population: i32,
    announcement: &'a str,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct CellarPopulation {
    population: i64,
}

#[axum::debug_handler]
async fn root(State(pool): State<PgPool>) -> impl IntoResponse {
    let id: i64 = 0;
    match sqlx::query_as!(
        CellarPopulation,
        "SELECT population FROM cellar_data WHERE id = $1",
        &id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[axum::debug_handler]
async fn add(State(pool): State<PgPool>) -> impl IntoResponse {
    let id: i64 = 0;
    match sqlx::query_as!(
        CellarPopulation,
        "UPDATE cellar_data SET population = population + 1 WHERE id = $1 RETURNING population",
        &id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(data) => {
            let popreturn = PopulationAnnouncement {
                population: data.population as i32,
                announcement: "Little timmy was born today.",
            };

            Ok((StatusCode::OK, Json(popreturn)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn sub(State(pool): State<PgPool>) -> impl IntoResponse {
    let id: i64 = 0;
    match sqlx::query_as!(
        CellarPopulation,
        "UPDATE cellar_data SET population = population - 1 WHERE id = $1 RETURNING population",
        &id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(data) => {
            let popreturn = PopulationAnnouncement {
                population: data.population as i32,
                announcement: "Little Timmy died of dysentary.",
            };

            Ok((StatusCode::OK, Json(popreturn)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
