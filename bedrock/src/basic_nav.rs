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

use crate::datatypes::{Category, Painting, Slideshow};

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
    match sqlx::query_as!(Painting, "SELECT * FROM paintings WHERE category_id = $1", category_id)
        .fetch_all(&pool)
        .await
    {
        Ok(mut data) => {
            let url_prefix = "https://cdn.dansansouci.com/";
            for painting in data.iter_mut() {
                painting.img_md = format!("{}{}", url_prefix, painting.img_md);

                if let Some(img_lg_unwrapped) = &painting.img_lg {
                    painting.img_lg = Some(format!("{}{}", url_prefix, img_lg_unwrapped));
                }

                if let Some(img_xl_unwrapped) = &painting.img_xl {
                    painting.img_xl = Some(format!("{}{}", url_prefix, img_xl_unwrapped));
                }
            }

            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_categories(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(Category, "SELECT * FROM categories",).fetch_all(&pool).await {
        Ok(mut data) => {
            let url_prefix = "https://cdn.dansansouci.com/";
            for category in data.iter_mut() {
                category.img_md = format!("{}{}", url_prefix, category.img_md);

                if let Some(img_lg_unwrapped) = &category.img_lg {
                    category.img_lg = Some(format!("{}{}", url_prefix, img_lg_unwrapped));
                }

                if let Some(img_xl_unwrapped) = &category.img_xl {
                    category.img_xl = Some(format!("{}{}", url_prefix, img_xl_unwrapped));
                }

                if let Some(img_vt_unwrapped) = &category.img_vt {
                    category.img_vt = Some(format!("{}{}", url_prefix, img_vt_unwrapped));
                }
            }

            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_slideshow_images(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(Slideshow, "SELECT * FROM slideshow",).fetch_all(&pool).await {
        Ok(mut data) => {
            let url_prefix = "https://cdn.dansansouci.com/";
            for slideshow in data.iter_mut() {
                slideshow.img_md = format!("{}{}", url_prefix, slideshow.img_md);

                if let Some(img_lg_unwrapped) = &slideshow.img_lg {
                    slideshow.img_lg = Some(format!("{}{}", url_prefix, img_lg_unwrapped));
                }

                if let Some(img_xl_unwrapped) = &slideshow.img_xl {
                    slideshow.img_xl = Some(format!("{}{}", url_prefix, img_xl_unwrapped));
                }

                slideshow.img_vt = format!("{}{}", url_prefix, slideshow.img_vt);
            }

            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
