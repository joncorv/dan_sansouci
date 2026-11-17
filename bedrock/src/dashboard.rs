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
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx::Postgres;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::AppState;
use crate::datatypes::{Category, Painting, Slideshow};

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
) -> Result<(StatusCode, Json<DummyResponse>), StatusCode> {
    // let mut category_name: Option<String> = None;
    let mut img: Result<DynamicImage, ImageError>;

    let mut category = Category::default();

    while let Some(mut field) = multipart.next_field().await.unwrap_or_default() {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "name" => {
                // This is the Form Input of "Name"
                // use this for the new Category Name
                category.name = field_name.clone();
                println!("we have a name: {}", field.text().await.unwrap_or_default().to_string());
            }
            "img" => {
                // This is the multipart image key
                // use this for the new Category Name
                println!("we have an image");

                if let Ok(bytes) = field.bytes().await {
                    match image::load_from_memory(&bytes) {
                        Ok(_img) => {
                            // good image, good boy
                            println!("hey this image works");
                        }
                        Err(e) => {
                            // need to send an error to the client
                            eprintln!("{e:?}");
                        }
                    }
                }
            }
            _ => {
                // if there are any other fields, send a 400
                println!("we have something else");
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        // let name = field.name().unwrap().to_string();
        // let data = field.bytes().await.unwrap();

        // println!("Length of `{}` is {} bytes", name, data.len());
    }

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
