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
use crate::datatypes::{CategorySchema, PaintingSchema, Size, SlideshowSchema};

pub fn build_routes_paintings() -> Router<AppState> {
    Router::new().route("/paintings/{category_id}", get(get_paintings_by_category_id))
}

#[axum::debug_handler]
pub async fn get_paintings_by_category_id(State(app_state): State<AppState>, Path(category_id): Path<Uuid>) -> impl IntoResponse {
    println!("getting some paintings here");
    match sqlx::query_as!(
        PaintingSchema,
        r#"
        SELECT * FROM paintings 
        WHERE category_id = $1 
        ORDER BY idx
        "#,
        category_id
    )
    .fetch_all(&app_state.pool)
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
