#![allow(unused_imports)]
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use axum::Json;
use axum::Router;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use image::DynamicImage;
use image::ImageError;
use image::imageops;
use image::imageops::FilterType;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use webp::Encoder;
use webp::WebPMemory;

use std::path::Path as StdPath;

use crate::AppState;
use crate::datatypes::Size::{Lg, Md, Vt, Xl};
use crate::datatypes::{Category, Painting, Size, Slideshow};

pub fn build_routes_slideshow() -> Router<AppState> {
    Router::new()
        .route("/slideshow", get(get_slideshow).post(post_slideshow).patch(patch_slideshow))
        .route("/sort-slideshow", patch(sort_slideshow))
}

#[axum::debug_handler]
pub async fn patch_slideshow(State(_app_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, String::from("WIP -- user updates a single slideshow item"))
}

#[axum::debug_handler]
pub async fn post_slideshow(State(_app_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, String::from("WIP --- user creates a new slideshow item"))
}

#[axum::debug_handler]
pub async fn sort_slideshow(State(app_state): State<AppState>, Json(uuids): Json<Vec<Uuid>>) -> impl IntoResponse {
    if uuids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, String::from("Empty array")));
    }
    let indexes: Vec<i32> = (0..uuids.len()).map(|num| num as i32).collect();
    let sort_result = sqlx::query_as!(
        Slideshow,
        r#"
            UPDATE slideshow as old_slides
            SET
                idx = new_slides.idx
            FROM UNNEST($1::uuid[], $2::int[]) AS new_slides(id, idx)
            WHERE old_slides.id = new_slides.id
        "#,
        &uuids,
        &indexes
    )
    .execute(&app_state.pool)
    .await;

    if let Err(e) = sort_result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    match sqlx::query_as!(Category, "SELECT * FROM categories ORDER BY idx",)
        .fetch_all(&app_state.pool)
        .await
    {
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
pub async fn get_slideshow(State(app_state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as!(Slideshow, "SELECT * FROM slideshow ORDER BY idx",)
        .fetch_all(&app_state.pool)
        .await
    {
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
