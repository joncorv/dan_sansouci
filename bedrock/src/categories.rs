// use aws_sdk_s3 as s3;
// use aws_sdk_s3::primitives::ByteStream;
use axum::Json;
use axum::Router;
use axum::extract::Multipart;
use axum::extract::Path;
// use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use image::DynamicImage;
// use image::ImageError;
// use image::imageops;
// use image::imageops::FilterType;
// use rayon::prelude::*;
// use serde::{Deserialize, Serialize};
// use sqlx::Pool;
// use sqlx::Postgres;
// use sqlx::postgres::PgPool;
// use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
// use webp::Encoder;
// use webp::WebPMemory;

use crate::AppState;
use crate::datatypes::ImageMetadata;
// use crate::datatypes::ImageType;
// use crate::datatypes::ImageType::{Category, Painting, Slideshow};
// use crate::datatypes::Size::{Lg, Md, Vt, Xl};
use crate::datatypes::CategorySchema;
use crate::image_processing::process_images;
// use std::path::Path as StdPath;

// use crate::image_processing::re_sizes_needed;
// use crate::image_processing::sizes_needed;

pub fn build_routes_categories() -> Router<AppState> {
    Router::new()
        .route("/categories", get(get_categories))
        .route("/category", post(post_category).patch(patch_category))
        .route("/category/{category_id}", delete(delete_category))
        .route("/sort-categories", patch(sort_categories))
}

#[axum::debug_handler]
pub async fn delete_category(
    State(app_state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<CategorySchema>>), (StatusCode, String)> {
    let query_result = sqlx::query_as!(
        CategorySchema,
        r#"
            DELETE FROM categories
            WHERE category_id = $1
        "#,
        category_id,
    )
    .execute(&app_state.pool)
    .await;

    if let Err(e) = query_result {
        Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database Error: {e}")))
    } else {
        let all_items = get_all_categories_from_database(&app_state).await?;
        Ok(all_items)
    }
}

#[axum::debug_handler]
pub async fn sort_categories(State(app_state): State<AppState>, Json(uuids): Json<Vec<Uuid>>) -> impl IntoResponse {
    if uuids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, String::from("Empty array")));
    }
    let indexes: Vec<i32> = (0..uuids.len()).map(|num| num as i32).collect();
    let sort_result = sqlx::query_as!(
        Category,
        r#"
            UPDATE categories as old_cats
            SET
                idx = new_cats.idx
            FROM UNNEST($1::uuid[], $2::int[]) AS new_cats(category_id, idx)
            WHERE old_cats.category_id = new_cats.category_id
        "#,
        &uuids,
        &indexes
    )
    .execute(&app_state.pool)
    .await;

    if let Err(e) = sort_result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let all_items = get_all_categories_from_database(&app_state).await?;
    Ok(all_items)
}

#[axum::debug_handler]
pub async fn patch_category(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<CategorySchema>>), (StatusCode, String)> {
    let image_metadata = Arc::new(Mutex::new(ImageMetadata::default()));
    let mut img: Option<DynamicImage> = None;
    let mut name: Option<String> = None;
    let mut id: Option<Uuid> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => match field.text().await {
                Ok(text) => {
                    name = Some(text.to_string());
                }
                Err(e) => {
                    eprintln!("illegal idx value. taking as blank and allowing to continue: {e}");
                }
            },
            "idx" => {
                // for now we are skipping idx. sorting is a separate function.
            }

            "id" => {
                match field.text().await {
                    Ok(idx_text) => {
                        match idx_text.parse::<Uuid>() {
                            Ok(idx) => {
                                // nice we have a fuckin cool and sweet Uuid
                                id = Some(idx)
                            }
                            Err(e) => {
                                return Err((StatusCode::BAD_REQUEST, format!("illegal id: {e}")));
                            }
                        }
                    }
                    Err(e) => {
                        return Err((StatusCode::BAD_REQUEST, format!("Multipart form error: {e}")));
                    }
                }
            }

            "img" => {
                let bytes = field.bytes().await;
                println!("we have an image attached to the form");
                match bytes {
                    Ok(bytes) => {
                        match image::load_from_memory(&bytes) {
                            Ok(img_good) => {
                                println!("hey this image works");

                                img = Some(img_good);
                            }
                            Err(e) => {
                                //
                                return Err((StatusCode::BAD_REQUEST, format!("image processing error: {e}")));
                            }
                        }
                    }
                    Err(e) => {
                        return Err((StatusCode::BAD_REQUEST, format!("Image file error: {e}")));
                    }
                }
            }
            _ => {
                return Err((StatusCode::BAD_REQUEST, "Client using illegal fields".into()));
            }
        }
    }

    // if there is no id
    if id.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("Valid ID is required to drink in the USA")));
    }
    // if there is no image or name
    else if img.is_none() && name.is_none() {
        return Err((StatusCode::BAD_REQUEST, "No name or image attached to request".into()));
    }
    // else if there is only a name
    else if img.is_none()
        && let Some(name) = &name
        && let Some(id) = &id
    {
        let name_update_result = sqlx::query_as!(
            Category,
            r#"
                UPDATE categories
                SET name = $2
                WHERE category_id = $1
            "#,
            id,
            name,
        )
        .execute(&app_state.pool)
        .await;

        if let Err(e) = name_update_result {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }

        let all_items = get_all_categories_from_database(&app_state).await?;
        return Ok(all_items);
    }

    /////////////////////////////////////////////////////////////
    ///////////////// PROCESS IMAGES ////////////////////////////
    /////////////////////////////////////////////////////////////
    if let Some(img) = img
        && let Some(_id) = &id
    {
        process_images(&app_state, &img, Arc::clone(&image_metadata)).await?;
    }

    if let Some(name) = &name
        && let Some(id) = &id
    {
        let image_metadata = image_metadata.lock().await;
        dbg!(&image_metadata);

        let query_result = sqlx::query_as!(
            CategorySchema,
            r#"
                UPDATE categories 
                SET
                    name = $2,
                    img_md = $3,
                    img_md_width = $4,
                    img_md_height = $5,
                    img_lg = $6,
                    img_lg_width = $7,
                    img_lg_height = $8,
                    img_xl = $9,
                    img_xl_width = $10,
                    img_xl_height = $11,
                    img_vt = $12,
                    img_vt_width = $13,
                    img_vt_height = $14

                WHERE category_id = $1
            "#,
            id,
            name,
            image_metadata.img_md,
            image_metadata.img_md_width,
            image_metadata.img_md_height,
            image_metadata.img_lg,
            image_metadata.img_lg_width,
            image_metadata.img_lg_height,
            image_metadata.img_xl,
            image_metadata.img_xl_width,
            image_metadata.img_xl_height,
            image_metadata.img_vt,
            image_metadata.img_vt_width,
            image_metadata.img_vt_height
        )
        .execute(&app_state.pool)
        .await;

        if let Err(e) = query_result {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database Error: {e}")));
        } else {
            let all_items = get_all_categories_from_database(&app_state).await?;
            return Ok(all_items);
        }
    }
    Err((StatusCode::INTERNAL_SERVER_ERROR, "logic error".to_string()))
}

#[axum::debug_handler]
pub async fn get_categories(State(app_state): State<AppState>) -> Result<(StatusCode, Json<Vec<CategorySchema>>), (StatusCode, String)> {
    println!("getting some categories here");
    let all_items = get_all_categories_from_database(&app_state).await?;
    Ok(all_items)
}

#[axum::debug_handler]
pub async fn post_category(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<CategorySchema>>), (StatusCode, String)> {
    let image_metadata = Arc::new(Mutex::new(ImageMetadata::default()));
    let id = uuid::Uuid::new_v4();

    {
        let mut lock = image_metadata.lock().await;
        lock.id = id;
    }

    let mut img: Option<DynamicImage> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => {
                let mut lock = image_metadata.lock().await;
                lock.name = field.text().await.unwrap_or_default();
                println!("we have a name: {}", lock.name);
            }
            "idx" => match field.text().await {
                Ok(idx_text) => match idx_text.parse::<i32>() {
                    Ok(idx) => {
                        let mut lock = image_metadata.lock().await;
                        lock.idx = idx;
                        println!("we have an index: {}", idx);
                    }
                    Err(e) => {
                        return Err((StatusCode::BAD_REQUEST, format!("illegal idx value: {e}")));
                    }
                },
                Err(e) => {
                    return Err((StatusCode::BAD_REQUEST, format!("Multipart form error: {e}")));
                }
            },
            "img" => {
                let bytes = field.bytes().await;
                println!("we have an image attached to the form");
                match bytes {
                    Ok(bytes) => {
                        match image::load_from_memory(&bytes) {
                            Ok(img_good) => {
                                println!("hey this image works");

                                img = Some(img_good);
                            }
                            Err(e) => {
                                //
                                return Err((StatusCode::BAD_REQUEST, format!("image processing error: {e}")));
                            }
                        }
                    }
                    Err(e) => {
                        return Err((StatusCode::BAD_REQUEST, format!("Image file error: {e}")));
                    }
                }
            }
            _ => {
                return Err((StatusCode::BAD_REQUEST, "Client using illegal fields".into()));
            }
        }
    }

    /////////////////////////////////////////////////////////////
    ///////////////// PROCESS IMAGES ////////////////////////////
    /////////////////////////////////////////////////////////////
    if let Some(img) = img {
        crate::image_processing::process_images(&app_state, &img, Arc::clone(&image_metadata)).await?
    }

    {
        let category = image_metadata.lock().await;

        let query_result = sqlx::query_as!(
            CategorySchema,
            r#"
            INSERT INTO categories (
                category_id,
                idx,
                name,
                img_md,
                img_md_width,
                img_md_height,
                img_lg,
                img_lg_width,
                img_lg_height,
                img_xl,
                img_xl_width,
                img_xl_height,
                img_vt,
                img_vt_width,
                img_vt_height
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (category_id)
            DO UPDATE set
                category_id = EXCLUDED.category_id,
                idx = EXCLUDED.idx,
                name = EXCLUDED.name,
                img_md = EXCLUDED.img_md,
                img_md_width = EXCLUDED.img_md_width,
                img_md_height = EXCLUDED.img_md_height,
                img_lg = EXCLUDED.img_lg,
                img_lg_width = EXCLUDED.img_lg_width,
                img_lg_height = EXCLUDED.img_lg_height,
                img_xl = EXCLUDED.img_xl,
                img_xl_width = EXCLUDED.img_xl_width,
                img_xl_height = EXCLUDED.img_xl_height,
                img_vt = EXCLUDED.img_vt,
                img_vt_width = EXCLUDED.img_vt_width,
                img_vt_height = EXCLUDED.img_vt_height
            RETURNING
                category_id,
                idx,
                name,
                img_md,
                img_md_width,
                img_md_height,
                img_lg,
                img_lg_width,
                img_lg_height,
                img_xl,
                img_xl_width,
                img_xl_height,
                img_vt,
                img_vt_width,
                img_vt_height
            "#,
            category.id,
            category.idx,
            category.name,
            category.img_md,
            category.img_md_width,
            category.img_md_height,
            category.img_lg,
            category.img_lg_width,
            category.img_lg_height,
            category.img_xl,
            category.img_xl_width,
            category.img_xl_height,
            category.img_vt,
            category.img_vt_width,
            category.img_vt_height
        )
        .fetch_one(&app_state.pool)
        .await;

        if let Err(e) = query_result {
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database Error: {e}")))
        } else {
            let all_items = get_all_categories_from_database(&app_state).await?;
            Ok(all_items)
        }
    }
}

pub async fn get_all_categories_from_database(
    app_state: &AppState,
) -> Result<(StatusCode, Json<Vec<CategorySchema>>), (StatusCode, String)> {
    match sqlx::query_as!(CategorySchema, "SELECT * FROM categories ORDER BY idx",)
        .fetch_all(&app_state.pool)
        .await
    {
        Ok(mut data) => {
            let url_prefix = "https://cdn.dansansouci.com/";
            for category in data.iter_mut() {
                category.img_md = format!("{}{}", url_prefix, category.img_md);
                category.img_vt = format!("{}{}", url_prefix, category.img_vt);

                if let Some(img_lg_unwrapped) = &category.img_lg {
                    category.img_lg = Some(format!("{}{}", url_prefix, img_lg_unwrapped));
                }

                if let Some(img_xl_unwrapped) = &category.img_xl {
                    category.img_xl = Some(format!("{}{}", url_prefix, img_xl_unwrapped));
                }
            }

            Ok((StatusCode::OK, Json(data)))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn post_new_category_to_database() {}
