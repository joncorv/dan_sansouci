use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct PaintingCategory {
    id: Uuid,
    index: u32,
    name: String,
    thumbnail: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Painting {
    id: Uuid,
    index: u32,
    name: String,
    medium: String,
    src_hq: String,
    src_thumb: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Slideshow {
    id: Uuid,
    idx: Option<i32>,
    src_img: Option<String>,
}

#[axum::debug_handler]
pub async fn get_slideshow_images(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(Slideshow, "SELECT * FROM slideshow",)
        .fetch_all(&pool)
        .await
    {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_painting_categories() {}

async fn get_paintings_by_index() {}
