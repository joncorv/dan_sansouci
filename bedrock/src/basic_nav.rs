use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct Category {
    category_id: i32,
    idx: i32,
    name: String,
    src_img: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Painting {
    id: Uuid,
    idx: i32,
    name: String,
    medium: String,
    src_thumb: String,
    src_thumb_height: i32,
    src_thumb_width: i32,
    src_img: String,
    src_img_height: i32,
    src_img_width: i32,
    category_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Slideshow {
    id: Uuid,
    idx: Option<i32>,
    src_img: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SlideshowSelection {
    src_img: Option<String>,
}

pub fn build_routes_basic_nav() -> Router<Pool<Postgres>> {
    Router::new()
        .route("/slideshow", get(get_slideshow_images))
        .route("/categories", get(get_categories))
        .route("/paintings/{category_id}", get(get_paintins_in_category))
}

#[axum::debug_handler]
pub async fn get_paintins_in_category(State(pool): State<PgPool>, Path(category_id): Path<i32>) -> impl IntoResponse {
    match sqlx::query_as!(Painting, "SELECT * FROM paintings WHERE category_id = $1", category_id).fetch_all(&pool).await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_categories(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(Category, "SELECT * FROM categories",).fetch_all(&pool).await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_slideshow_images(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(Slideshow, "SELECT * FROM slideshow",).fetch_all(&pool).await {
        Ok(data) => Ok((StatusCode::OK, Json(data))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
