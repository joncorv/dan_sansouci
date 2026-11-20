#![allow(unused_imports)]
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use axum::Json;
use axum::Router;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
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

// #[derive(Debug, Deserialize, Serialize)]
// struct SlideshowSelection {
//     src_img: Option<String>,
// }

pub fn build_routes_categories() -> Router<AppState> {
    Router::new().route("/categories", get(get_categories).post(post_category))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryRequest {
    pub name: String,
    // pub img:
}

#[axum::debug_handler]
pub async fn get_categories(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("getting some categories here");
    match sqlx::query_as!(Category, "SELECT * FROM categories",).fetch_all(&app_state.pool).await {
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
pub async fn post_category(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<Category>>), (StatusCode, String)> {
    let category = Arc::new(Mutex::new(Category::default()));
    let id = uuid::Uuid::new_v4();

    {
        let mut lock = category.lock().await;
        lock.category_id = id;
    }

    let mut img: Option<DynamicImage> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => {
                let mut lock = category.lock().await;
                lock.name = field.text().await.unwrap_or_default();
                println!("we have a name: {}", lock.name);
            }
            "idx" => {
                // let found_index = field.text().await;

                match field.text().await {
                    Ok(idx_text) => {
                        match idx_text.parse::<i32>() {
                            Ok(idx) => {
                                // nice we have a fuckin cool and sweet i32
                                let mut lock = category.lock().await;
                                lock.idx = idx;
                                println!("we have an index: {}", idx);
                            }
                            Err(e) => {
                                return Err((StatusCode::BAD_REQUEST, format!("illegal idx value: {e}")));
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
                println!("we have an image");
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

    if let Some(img) = img {
        let (w, h) = (img.width(), img.height());

        // will we transcode each size?
        let md_is_needed = true;
        let lg_is_needed = w > 700;
        let xl_is_needed = w > 1600;
        let vt_is_needed = true;

        // will we resize each size?
        let md_resize_is_needed = w > 700;
        let lg_resize_is_needed = lg_is_needed && w > 1600;
        let xl_resize_is_needed = xl_is_needed && w > 2500;
        let vt_resize_is_needed = h > 2400;

        // Simple Filname for all sizes
        let md_simple_filename = format!("md_{}", id);
        let lg_simple_filename = format!("lg_{}", id);
        let xl_simple_filename = format!("xl_{}", id);
        let vt_simple_filename = format!("vt_{}", id);

        // S3 Bucket Target Path
        let s3_prefix = "DUMMY_categories";
        let md_cdn_destination_path = format!("{s3_prefix}/{md_simple_filename}.webp");
        let lg_cdn_destination_path = format!("{s3_prefix}/{lg_simple_filename}.webp");
        let xl_cdn_destination_path = format!("{s3_prefix}/{xl_simple_filename}.webp");
        let vt_cdn_destination_path = format!("{s3_prefix}/{vt_simple_filename}.webp");

        // Set category data
        {
            let mut lock = category.lock().await;
            lock.img_md = md_cdn_destination_path.clone();
            lock.img_vt = Some(vt_cdn_destination_path.clone());
            if lg_is_needed {
                lock.img_lg = Some(lg_cdn_destination_path.clone());
            }
            if xl_is_needed {
                lock.img_xl = Some(xl_cdn_destination_path.clone());
            }
        }

        // println!("starting resize");
        // let md_img = resize_image(&img, category, Md, md_is_needed, md_resize_is_needed, w, h).unwrap();
        // let lg_img = resize_image(&img, category, Lg, lg_is_needed, lg_resize_is_needed, w, h).unwrap();
        // let xl_img = resize_image(&img, category, Xl, xl_is_needed, xl_resize_is_needed, w, h).unwrap();
        // let vt_img = resize_image(&img, category, Vt, vt_is_needed, vt_resize_is_needed, w, h).unwrap();

        // let resize_result = tokio::task::spawn_blocking(move || {
        //     let (mut md_img, mut lg_img, mut xl_img, mut vt_img) = (Ok(None), Ok(None), Ok(None), Ok(None));
        //     rayon::scope(|s| {
        //         s.spawn(|_| md_img = resize_image(&img, category.clone(), Md, md_is_needed, md_resize_is_needed, w, h));
        //         s.spawn(|_| lg_img = resize_image(&img, category.clone(), Lg, lg_is_needed, lg_resize_is_needed, w, h));
        //         s.spawn(|_| xl_img = resize_image(&img, category.clone(), Xl, xl_is_needed, xl_resize_is_needed, w, h));
        //         s.spawn(|_| vt_img = resize_image(&img, category.clone(), Vt, vt_is_needed, vt_resize_is_needed, w, h));
        //     });
        //     (md_img, lg_img, xl_img, vt_img)
        // })
        // .await;

        println!("starting resize");
        let (md_img_res, lg_img_res, xl_img_res, vt_img_res) = tokio::join!(
            resize_image(&img, category.clone(), Md, md_is_needed, md_resize_is_needed, w, h),
            resize_image(&img, category.clone(), Lg, lg_is_needed, lg_resize_is_needed, w, h),
            resize_image(&img, category.clone(), Xl, xl_is_needed, xl_resize_is_needed, w, h),
            resize_image(&img, category.clone(), Vt, vt_is_needed, vt_resize_is_needed, w, h),
        );

        let (md_img, lg_img, xl_img, vt_img);
        match md_img_res {
            Ok(img) => {
                md_img = img;
            }
            Err(e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Image resize error: {e}")));
            }
        }
        match lg_img_res {
            Ok(img) => {
                lg_img = img;
            }
            Err(e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Image resize error: {e}")));
            }
        }
        match xl_img_res {
            Ok(img) => {
                xl_img = img;
            }
            Err(e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Image resize error: {e}")));
            }
        }
        match vt_img_res {
            Ok(img) => {
                vt_img = img;
            }
            Err(e) => {
                return Err((StatusCode::BAD_REQUEST, format!("Image resize error: {e}")));
            }
        }

        let encode_result = tokio::task::spawn_blocking(move || {
            let (mut md_bytes, mut lg_bytes, mut xl_bytes, mut vt_bytes) = (Ok(None), Ok(None), Ok(None), Ok(None));
            rayon::scope(|s| {
                s.spawn(|_| md_bytes = encode_image_to_bytes(md_is_needed, &md_img));
                s.spawn(|_| lg_bytes = encode_image_to_bytes(lg_is_needed, &lg_img));
                s.spawn(|_| xl_bytes = encode_image_to_bytes(xl_is_needed, &xl_img));
                s.spawn(|_| vt_bytes = encode_image_to_bytes(vt_is_needed, &vt_img));
            });
            (md_bytes, lg_bytes, xl_bytes, vt_bytes)
        })
        .await;

        // handle any encode errors here and break the whole thing
        let (md_bytes, lg_bytes, xl_bytes, vt_bytes);
        match encode_result {
            Ok(val) => {
                match val.0 {
                    Ok(md) => md_bytes = md,
                    Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image encode error: {e}"))),
                }
                match val.1 {
                    Ok(lg) => lg_bytes = lg,
                    Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image encode error: {e}"))),
                }
                match val.2 {
                    Ok(xl) => xl_bytes = xl,
                    Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image encode error: {e}"))),
                }
                match val.3 {
                    Ok(vt) => vt_bytes = vt,
                    Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image encode error: {e}"))),
                }
            }
            Err(e) => {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image encode error: {e}")));
            }
        }

        /////////////////////////////////////////////////////////////
        ///////////////// Upload to S3 //////////////////////////////
        /////////////////////////////////////////////////////////////

        let (_a, _b, _c, _d) = tokio::join!(
            upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &md_cdn_destination_path, md_bytes, md_is_needed,),
            upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &lg_cdn_destination_path, lg_bytes, lg_is_needed,),
            upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &xl_cdn_destination_path, xl_bytes, xl_is_needed,),
            upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &vt_cdn_destination_path, vt_bytes, vt_is_needed,),
        );
    } else {
        return Err((StatusCode::BAD_REQUEST, String::from("You must include an image with each new Category")));
    }

    {
        let category = category.lock().await;

        let query_result = sqlx::query_as!(
            Category,
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
            category.category_id,
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
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database Error: {e}")));
        } else {
            match sqlx::query_as!(Category, "SELECT * FROM categories",).fetch_all(&app_state.pool).await {
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

                    return Ok((StatusCode::OK, Json(data)));
                }
                Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }
    }
    // dbg!(category);

    // let dummy_response = DummyResponse { text: "dummy_response".to_string() };
    // Ok((StatusCode::OK, Json(dummy_response)))
}

pub async fn resize_image(
    img: &DynamicImage,
    category: Arc<Mutex<Category>>,
    size: Size,
    is_needed: bool,
    resize_is_needed: bool,
    w: u32,
    h: u32,
) -> anyhow::Result<Option<DynamicImage>, anyhow::Error> {
    println!("resize of size: {:#?}", size);
    let new_img = if is_needed {
        if resize_is_needed {
            let resize_factor = match size {
                Md => 700.0 / w as f64,
                Lg => 1600.0 / w as f64,
                Xl => 2500.0 / w as f64,
                Vt => 2400.0 / h as f64,
            };

            let (new_w, new_h) = ((w as f64 * resize_factor) as i32, (h as f64 * resize_factor) as i32);

            match size {
                Md => {
                    let mut lock = category.lock().await;
                    lock.img_md_width = new_w;
                    lock.img_md_height = new_h;
                }
                Lg => {
                    let mut lock = category.lock().await;
                    lock.img_lg_width = Some(new_w);
                    lock.img_lg_height = Some(new_h);
                }
                Xl => {
                    let mut lock = category.lock().await;
                    lock.img_xl_width = Some(new_w);
                    lock.img_xl_height = Some(new_h);
                }
                Vt => {
                    let mut lock = category.lock().await;
                    lock.img_vt_width = Some(new_h / 2);
                    lock.img_vt_height = Some(new_h);
                }
            }

            match size {
                Md | Lg | Xl => DynamicImage::ImageRgba8(imageops::resize(img, new_w as u32, new_h as u32, imageops::FilterType::Triangle)),
                _ => {
                    let new_w = new_w as u32;
                    let new_h = new_h as u32;
                    let x_crop = (new_w / 2) - (new_h / 4);

                    img.resize(new_w, new_h, imageops::FilterType::Triangle).crop_imm(x_crop, 0, new_h / 2, new_h)
                }
            }
        } else {
            // this particular size is needed, but resize is NOT needed
            match size {
                Md => {
                    let mut lock = category.lock().await;
                    lock.img_md_width = w as i32;
                    lock.img_md_height = h as i32;
                    img.clone()
                }
                Lg => {
                    let mut lock = category.lock().await;
                    lock.img_lg_width = Some(w as i32);
                    lock.img_lg_height = Some(h as i32);
                    img.clone()
                }
                Xl => {
                    let mut lock = category.lock().await;
                    lock.img_xl_width = Some(w as i32);
                    lock.img_xl_height = Some(h as i32);
                    img.clone()
                }
                _ => {
                    let mut lock = category.lock().await;
                    lock.img_vt_width = Some((h / 2) as i32);
                    lock.img_vt_height = Some(h as i32);
                    let x_crop = (w / 2) - (h / 4);
                    img.crop_imm(x_crop, 0, h / 2, h)
                }
            }
        }
    } else {
        return Ok(None);
    };
    Ok(Some(new_img))
}

pub fn encode_image_to_bytes(is_needed: bool, sized_img: &Option<DynamicImage>) -> Result<Option<Vec<u8>>, anyhow::Error> {
    if is_needed && let Some(img) = sized_img {
        println!("encoding now");
        let encoder = match webp::Encoder::from_image(img) {
            Ok(it) => it,
            Err(err) => return Err(anyhow::anyhow!("encoder error: {:?}", err)),
        };
        let webp_bytes = encoder.encode(80f32).to_vec();
        return Ok(Some(webp_bytes));
    }

    Ok(None)
}

async fn upload_object(
    client: &s3::Client,
    bucket: &str,
    key: &str,
    bytes: Option<Vec<u8>>,
    validation: bool,
) -> Result<(), anyhow::Error> {
    if validation && let Some(bytes) = bytes {
        let body = ByteStream::from(bytes);

        client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .content_type("image/webp")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await?;

        println!("Uploadedto {}/{}", bucket, key);
    }

    Ok(())
}

pub async fn post_new_category_to_database() {}
