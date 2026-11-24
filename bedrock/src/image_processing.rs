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
use crate::datatypes::ImageMetadata;
use crate::datatypes::ImageType;
use crate::datatypes::ImageType::{Category, Painting, Slideshow};
use crate::datatypes::Size;
use crate::datatypes::Size::{Lg, Md, Vt, Xl};
use crate::datatypes::{CategorySchema, PaintingSchema, SlideshowSchema};

pub async fn process_images(
    app_state: &AppState,
    img: &DynamicImage,
    image_metadata: Arc<Mutex<ImageMetadata>>,
) -> Result<(), (StatusCode, String)> {
    let (w, h) = (img.width(), img.height());

    // will we transcode each size?
    let (md_is_needed, lg_is_needed, xl_is_needed, vt_is_needed) = sizes_needed(Category, w, h);

    // will we resize each size?
    let (md_resize_is_needed, lg_resize_is_needed, xl_resize_is_needed, vt_resize_is_needed) = re_sizes_needed(Category, w, h);

    // run some tasks that require reading the Mutex
    {
        // Set category data
        let mut lock = image_metadata.lock().await;

        // S3 Bucket Target Path
        let s3_prefix = format!("{:?}", lock.image_type);
        let md_cdn_destination_path = format!("{s3_prefix}/md_{}.webp", lock.id);
        let lg_cdn_destination_path = format!("{s3_prefix}/lg_{}.webp", lock.id);
        let xl_cdn_destination_path = format!("{s3_prefix}/xl_{}.webp", lock.id);
        let vt_cdn_destination_path = format!("{s3_prefix}/vt_{}.webp", lock.id);

        lock.img_md = md_cdn_destination_path.clone();
        lock.img_vt = vt_cdn_destination_path.clone();
        if lg_is_needed {
            lock.img_lg = Some(lg_cdn_destination_path.clone());
        }
        if xl_is_needed {
            lock.img_xl = Some(xl_cdn_destination_path.clone());
        }
    }

    println!("starting resize");
    let (md_img_res, lg_img_res, xl_img_res, vt_img_res) = tokio::join!(
        resize_image(img, Arc::clone(&image_metadata), Md, md_is_needed, md_resize_is_needed, w, h),
        resize_image(img, Arc::clone(&image_metadata), Lg, lg_is_needed, lg_resize_is_needed, w, h),
        resize_image(img, Arc::clone(&image_metadata), Xl, xl_is_needed, xl_resize_is_needed, w, h),
        resize_image(img, Arc::clone(&image_metadata), Vt, vt_is_needed, vt_resize_is_needed, w, h),
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

    let (md_path, lg_path, xl_path, vt_path): (String, String, String, String) = {
        let lock = image_metadata.lock().await;

        (
            lock.img_md.clone(),
            lock.img_lg.clone().unwrap_or_default(),
            lock.img_xl.clone().unwrap_or_default(),
            lock.img_vt.clone(),
        )
    };

    let (_a, _b, _c, _d) = tokio::join!(
        upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &md_path, md_bytes, md_is_needed,),
        upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &lg_path, lg_bytes, lg_is_needed,),
        upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &xl_path, xl_bytes, xl_is_needed,),
        upload_object(&app_state.s3_client, &app_state.s3_bucket_name, &vt_path, vt_bytes, vt_is_needed,),
    );

    Ok(())
}

pub async fn resize_image(
    img: &DynamicImage,
    image_metadata: Arc<Mutex<ImageMetadata>>,
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
                    let mut lock = image_metadata.lock().await;
                    lock.img_md_width = new_w;
                    lock.img_md_height = new_h;
                }
                Lg => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_lg_width = Some(new_w);
                    lock.img_lg_height = Some(new_h);
                }
                Xl => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_xl_width = Some(new_w);
                    lock.img_xl_height = Some(new_h);
                }
                Vt => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_vt_width = new_h / 2;
                    lock.img_vt_height = new_h;
                }
            }

            match size {
                Md | Lg | Xl => DynamicImage::ImageRgba8(imageops::resize(img, new_w as u32, new_h as u32, imageops::FilterType::Triangle)),
                _ => {
                    let new_w = new_w as u32;
                    let new_h = new_h as u32;
                    let x_crop = (new_w / 2) - (new_h / 4);

                    img.resize(new_w, new_h, imageops::FilterType::Triangle)
                        .crop_imm(x_crop, 0, new_h / 2, new_h)
                }
            }
        } else {
            // this particular size is needed, but resize is NOT needed
            match size {
                Md => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_md_width = w as i32;
                    lock.img_md_height = h as i32;
                    img.clone()
                }
                Lg => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_lg_width = Some(w as i32);
                    lock.img_lg_height = Some(h as i32);
                    img.clone()
                }
                Xl => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_xl_width = Some(w as i32);
                    lock.img_xl_height = Some(h as i32);
                    img.clone()
                }
                _ => {
                    let mut lock = image_metadata.lock().await;
                    lock.img_vt_width = (h / 2) as i32;
                    lock.img_vt_height = h as i32;
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

pub fn sizes_needed(image_type: ImageType, w: u32, h: u32) -> (bool, bool, bool, bool) {
    match image_type {
        Category => {
            let md_is_needed = true;
            let lg_is_needed = w > 700;
            let xl_is_needed = w > 1600;
            let vt_is_needed = true;

            (md_is_needed, lg_is_needed, xl_is_needed, vt_is_needed)
        }
        Painting => {
            //
            (false, false, false, false)
        }
        Slideshow => {
            //
            (false, false, false, false)
        }
    }
}

pub fn re_sizes_needed(image_type: ImageType, w: u32, h: u32) -> (bool, bool, bool, bool) {
    match image_type {
        Category => {
            // the logic here is wrong
            let md_resize_is_needed = w > 700;
            let lg_resize_is_needed = w > 1600;
            let xl_resize_is_needed = w > 2500;
            let vt_resize_is_needed = h > 2400;

            (md_resize_is_needed, lg_resize_is_needed, xl_resize_is_needed, vt_resize_is_needed)
        }
        Painting => {
            //
            (false, false, false, false)
        }
        Slideshow => {
            //
            (false, false, false, false)
        }
    }
}
