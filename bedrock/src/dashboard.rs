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
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use webp::Encoder;
use webp::WebPMemory;

use std::path::Path as FilePath;
use std::path::PathBuf;

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
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DummyResponse>), (StatusCode, String)> {
    let mut category = Category::default();
    let id = uuid::Uuid::new_v4();
    category.category_id = id.clone();
    let mut category_name = "".to_string();

    while let Some(mut field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => {
                {
                    category_name = field.text().await.unwrap_or_default();
                    // println!("we have a name: {}", category.name);
                }
            }
            "img" => {
                let bytes = field.bytes().await;
                println!("we have an image");

                match bytes {
                    Ok(bytes) => {
                        match image::load_from_memory(&bytes) {
                            /////////////////////////////////////////////////////////////
                            ///////////////// TRANSCODE IMAGES //////////////////////////
                            /////////////////////////////////////////////////////////////
                            Ok(img) => {
                                println!("hey this image works");
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
                                let md_transcode_result_destination = dummy_dir.path().join(&md_simple_filename).with_extension("webp");
                                let lg_transcode_result_destination = dummy_dir.path().join(&lg_simple_filename).with_extension("webp");
                                let xl_transcode_result_destination = dummy_dir.path().join(&xl_simple_filename).with_extension("webp");
                                let vt_transcode_result_destination = dummy_dir.path().join(&vt_simple_filename).with_extension("webp");

                                // S3 Bucket Target Path
                                let s3_prefix = "categories";
                                let md_cdn_destination_path = format!("{s3_prefix}/{md_simple_filename}.webp");
                                let lg_cdn_destination_path = format!("{s3_prefix}/{lg_simple_filename}.webp");
                                let xl_cdn_destination_path = format!("{s3_prefix}/{xl_simple_filename}.webp");
                                let vt_cdn_destination_path = format!("{s3_prefix}/{vt_simple_filename}.webp");

                                // Set category data
                                category.img_md = md_cdn_destination_path.clone();
                                category.img_vt = Some(vt_cdn_destination_path.clone());
                                if lg_is_needed {
                                    category.img_lg = Some(lg_cdn_destination_path.clone());
                                }
                                if xl_is_needed {
                                    category.img_xl = Some(xl_cdn_destination_path.clone());
                                }

                                println!("starting resize");
                                let md_img = resize_image(&img, &mut category, Md, md_is_needed, md_resize_is_needed, w, h).unwrap();
                                let lg_img = resize_image(&img, &mut category, Lg, lg_is_needed, lg_resize_is_needed, w, h).unwrap();
                                let xl_img = resize_image(&img, &mut category, Xl, xl_is_needed, xl_resize_is_needed, w, h).unwrap();
                                let vt_img = resize_image(&img, &mut category, Vt, vt_is_needed, vt_resize_is_needed, w, h).unwrap();

                                println!("starting transcode");
                                let (_a, _b, _c, _d) = tokio::join!(
                                    transcode_image(md_is_needed, &md_img, &md_transcode_result_destination),
                                    transcode_image(lg_is_needed, &lg_img, &lg_transcode_result_destination),
                                    transcode_image(xl_is_needed, &xl_img, &xl_transcode_result_destination),
                                    transcode_image(vt_is_needed, &vt_img, &vt_transcode_result_destination),
                                );

                                println!("wrote webp successfully");
                                // use tokio::task::spawn_blocking;

                                // let (a, b, c, d) = tokio::try_join!(
                                //     spawn_blocking(resize_image(
                                //         &img,
                                //         &mut category,
                                //         Md,
                                //         md_is_needed,
                                //         md_resize_is_needed,
                                //         w,
                                //         h
                                //     )),
                                //     spawn_blocking(resize_image(
                                //         &img,
                                //         &mut category,
                                //         Lg,
                                //         lg_is_needed,
                                //         lg_resize_is_needed,
                                //         w,
                                //         h
                                //     )),
                                //     spawn_blocking(resize_image(
                                //         &img,
                                //         &mut category,
                                //         Xl,
                                //         xl_is_needed,
                                //         xl_resize_is_needed,
                                //         w,
                                //         h
                                //     )),
                                //     spawn_blocking(resize_image(
                                //         &img,
                                //         &mut category,
                                //         Vt,
                                //         vt_is_needed,
                                //         vt_resize_is_needed,
                                //         w,
                                //         h
                                //     )),
                                // )
                                // .unwrap();

                                // tokio::task::spawn_blocking(move || {
                                //     let _ =
                                //         resize_image(&img, &mut category, Md, md_is_needed, md_resize_is_needed, w, h);
                                //     let _ =
                                //         resize_image(&img, &mut category, Lg, lg_is_needed, lg_resize_is_needed, w, h);
                                //     let _ =
                                //         resize_image(&img, &mut category, Xl, xl_is_needed, xl_resize_is_needed, w, h);
                                //     let _ =
                                //         resize_image(&img, &mut category, Vt, vt_is_needed, vt_resize_is_needed, w, h);
                                // })
                                // .await
                                // .unwrap();

                                // dbg!(&category);
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

    category.name = category_name;
    dbg!(category);

    let dummy_response = DummyResponse { text: "dummy_response".to_string() };
    Ok((StatusCode::OK, Json(dummy_response)))

    // match sqlx::query_as!(Painting, "SELECT * FROM paintings WHERE category_id = $1", category_id)
    //     .fetch_all(&pool)
    //     .await
    // {
    //     Ok(mut data) => {
    //         let url_prefix = "https://cdn.dansansouci.com/";
    //         for painting in data.iter_mut() {
    //             painting.img_md = format!("{}{}", url_prefix, painting.img_md);
    //
    //             if let Some(img_lg_unwrapped) = &painting.img_lg {
    //                 painting.img_lg = Some(format!("{}{}", url_prefix, img_lg_unwrapped));
    //             }
    //
    //             if let Some(img_xl_unwrapped) = &painting.img_xl {
    //                 painting.img_xl = Some(format!("{}{}", url_prefix, img_xl_unwrapped));
    //             }
    //         }
    //
    //         Ok((StatusCode::OK, Json(data)))
    //     }
    //     Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    // }
}

pub fn resize_image(
    img: &DynamicImage,
    category: &mut Category,
    size: Size,
    is_needed: bool,
    resize_is_needed: bool,
    w: u32,
    h: u32,
) -> anyhow::Result<Option<DynamicImage>, anyhow::Error> {
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
                    category.img_md_width = new_w;
                    category.img_md_height = new_h;
                }
                Lg => {
                    category.img_lg_width = Some(new_w);
                    category.img_lg_height = Some(new_h);
                }
                Xl => {
                    category.img_xl_width = Some(new_w);
                    category.img_xl_height = Some(new_h);
                }
                Vt => {
                    category.img_vt_width = Some(new_h / 2);
                    category.img_vt_height = Some(new_h);
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
                    category.img_md_width = w as i32;
                    category.img_md_height = h as i32;
                    img.clone()
                }
                Lg => {
                    category.img_lg_width = Some(w as i32);
                    category.img_lg_height = Some(h as i32);
                    img.clone()
                }
                Xl => {
                    category.img_xl_width = Some(w as i32);
                    category.img_xl_height = Some(h as i32);
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

pub async fn transcode_image(
    is_needed: bool,
    sized_img: &Option<DynamicImage>,
    transcode_result_destination: &PathBuf,
) -> Result<(), anyhow::Error> {
    if is_needed && let Some(img) = sized_img {
        let md_encoder: webp::Encoder = webp::Encoder::from_image(img).unwrap();
        let md_webp: WebPMemory = md_encoder.encode(80f32);
        // tokio::fs::write(transcode_result_destination, &*md_webp).await.unwrap();
        std::fs::write(transcode_result_destination, &*md_webp)?;
    }

    Ok(())
}
