#![allow(unused_imports)]
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

pub fn build_routes_dashboard() -> Router<AppState> {
    Router::new().route("/create-category", post(create_category))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryRequest {
    pub name: String,
    // pub img:
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DummyResponse {
    pub text: String,
}

#[axum::debug_handler]
pub async fn create_category(
    State(_app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DummyResponse>), (StatusCode, String)> {
    let category = Arc::new(Mutex::new(Category::default()));
    let id = uuid::Uuid::new_v4();

    let mut img: Option<DynamicImage> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => {
                let mut lock = category.lock().await;
                lock.name = field.text().await.unwrap_or_default();
                println!("we have a name: {}", lock.name);
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
        let dummy_dir = tempdir::TempDir::new("category").unwrap();
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

        // Transcode Result Destination for all sizes
        let md_transcode_result_destination = Arc::new(dummy_dir.path().join(&md_simple_filename).with_extension("webp"));
        let lg_transcode_result_destination = Arc::new(dummy_dir.path().join(&lg_simple_filename).with_extension("webp"));
        let xl_transcode_result_destination = Arc::new(dummy_dir.path().join(&xl_simple_filename).with_extension("webp"));
        let vt_transcode_result_destination = Arc::new(dummy_dir.path().join(&vt_simple_filename).with_extension("webp"));

        // S3 Bucket Target Path
        let s3_prefix = "categories";
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

        let transcode_result = tokio::task::spawn_blocking(move || {
            let (mut tc_md, mut tc_lg, mut tc_xl, mut tc_vt) = (Ok(()), Ok(()), Ok(()), Ok(()));
            rayon::scope(|s| {
                s.spawn(|_| tc_md = transcode_image(md_is_needed, &md_img, md_transcode_result_destination.clone()));
                s.spawn(|_| tc_lg = transcode_image(lg_is_needed, &lg_img, lg_transcode_result_destination.clone()));
                s.spawn(|_| tc_xl = transcode_image(xl_is_needed, &xl_img, xl_transcode_result_destination.clone()));
                s.spawn(|_| tc_vt = transcode_image(vt_is_needed, &vt_img, vt_transcode_result_destination.clone()));
            });
            (tc_md, tc_lg, tc_xl, tc_vt)
        })
        .await;

        // handle any transcode errors here and break the whole thing
        match transcode_result {
            Ok(val) => {
                if let Err(e) = val.0 {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image transcode error: {e}")));
                }
                if let Err(e) = val.1 {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image transcode error: {e}")));
                }
                if let Err(e) = val.2 {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image transcode error: {e}")));
                }
                if let Err(e) = val.3 {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image transcode error: {e}")));
                }
            }
            Err(e) => {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Image transcode error: {e}")));
            }
        }
    }

    let dummy_response = DummyResponse { text: "dummy_response".to_string() };
    Ok((StatusCode::OK, Json(dummy_response)))
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

pub fn transcode_image(
    is_needed: bool,
    sized_img: &Option<DynamicImage>,
    transcode_result_destination: Arc<PathBuf>,
) -> Result<(), anyhow::Error> {
    if is_needed && let Some(img) = sized_img {
        println!("transcoding now");
        let md_encoder = match webp::Encoder::from_image(img) {
            Ok(it) => it,
            Err(err) => return Err(anyhow::anyhow!("encoder error: {:?}", err)),
        };
        let md_webp: WebPMemory = md_encoder.encode(80f32);
        std::fs::write(transcode_result_destination.as_path(), &*md_webp)?;
    }

    Ok(())
}
