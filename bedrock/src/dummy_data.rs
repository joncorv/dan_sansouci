use anyhow::Error;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use image::DynamicImage;
use image::GenericImageView;
use image::imageops;
use image::imageops::FilterType;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::fmt::Debug;
use std::path::Path;
use std::time::Duration;
// use tower_http::classify::GrpcCode::Ok;
use uuid::Uuid;
use webp::Encoder;
use webp::WebPMemory;

use crate::datatypes::Category;
use crate::datatypes::Painting;
use crate::datatypes::Size;
use crate::datatypes::Slideshow;

#[derive(Serialize, Deserialize, Debug)]
struct PaintingDescription {
    path: String,
    idx: i32,
    name: String,
    medium: String,
    category_id: i32,
}

pub async fn parse_args_create_dummy_data() -> Result<(), Error> {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dummy" => {
                println!("dummy mode created");

                create_dummy_slideshow().await?;
                create_dummy_categories().await?;
                create_dummy_paintings().await?;

                println!("dummy mode finished");
            }
            _ => eprintln!("Unknown argument: {arg}"),
        }
    }

    Ok(())
}

async fn create_dummy_paintings() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// CONNECT TO DATABASE ///////////////////////
    /////////////////////////////////////////////////////////////

    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    /////////////////////////////////////////////////////////////
    ///////////////// CREATE S3 Client //////////////////////////
    /////////////////////////////////////////////////////////////

    let bucket_name = std::env::var("S3_BUCKET_NAME")?;
    let account_id = std::env::var("S3_ACCOUNT_ID")?;
    let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
    let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &access_key_id,
            &access_key_secret,
            None, // session token is not used with R2
            None,
            "R2",
        ))
        .region("auto") // Required by SDK but not used by R2
        .load()
        .await;
    let client = s3::Client::new(&config);

    /////////////////////////////////////////////////////////////
    ///////////////// TRANSCODE IMAGES //////////////////////////
    /////////////////////////////////////////////////////////////
    let dummy_dir = std::env::var("DUMMY_DIR")?;

    // parse dummy_data json
    let file = tokio::fs::read_to_string("dummy_data/painting_descriptions.json").await.unwrap();
    let data: Vec<PaintingDescription> = serde_json::from_str(&file)?;

    for data in data {
        println!("transcoding: {:#?}", data.name);
        let id = uuid::Uuid::new_v4();

        let image_path_to_be_transcoded = format!("{dummy_dir}{}", data.path);
        let img = image::ImageReader::open(&image_path_to_be_transcoded)?.decode()?;
        let (w, h) = img.dimensions();

        // will we transcode each size?
        let md_is_needed = true;
        let lg_is_needed = w > 700;
        let xl_is_needed = w > 1600;
        // let vt_is_needed = true;

        // will we resize each size?
        let md_resize_is_needed = w > 700;
        let lg_resize_is_needed = lg_is_needed && w > 1600;
        let xl_resize_is_needed = xl_is_needed && w > 2500;
        // let vt_resize_is_needed = h > 2400;

        // Simple Filname for all sizes
        let md_simple_filename = format!("md_{id}");
        let lg_simple_filename = format!("lg_{id}");
        let xl_simple_filename = format!("xl_{id}");
        // let vt_simple_filename = format!("vt_{id}");

        // Transcode Result Destination for all sizes
        let md_transcode_result_destination = Path::new(".temp").join(&md_simple_filename).with_extension("webp");
        let lg_transcode_result_destination = Path::new(".temp").join(&lg_simple_filename).with_extension("webp");
        let xl_transcode_result_destination = Path::new(".temp").join(&xl_simple_filename).with_extension("webp");
        // let vt_transcode_result_destination = Path::new(".temp").join(&vt_simple_filename).with_extension("webp");

        // S3 Bucket Target Path
        let s3_prefix = "paintings";
        let md_cdn_destination_path = format!("{s3_prefix}/{md_simple_filename}.webp");
        let lg_cdn_destination_path = format!("{s3_prefix}/{lg_simple_filename}.webp");
        let xl_cdn_destination_path = format!("{s3_prefix}/{xl_simple_filename}.webp");
        // let vt_cdn_destination_path = format!("{s3_prefix}/{vt_simple_filename}.webp");

        // Calculate all size factors if required
        // let (md_final_resolution, lg_final_resolution, xl_final_resolution, _vt_final_resolution);

        let (md_final_resolution, lg_final_resolution, xl_final_resolution);

        let md_img = if md_is_needed {
            if md_resize_is_needed {
                let resize_factor = 700.0 / w as f64;

                md_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    md_final_resolution.0,
                    md_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                md_final_resolution = (w, h);
                img.clone()
            }
        } else {
            md_final_resolution = (w, h);
            img.clone()
        };

        let lg_img = if lg_is_needed {
            if lg_resize_is_needed {
                let resize_factor = 1600.0 / w as f64;

                lg_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);

                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    lg_final_resolution.0,
                    lg_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                lg_final_resolution = (w, h);
                img.clone()
            }
        } else {
            lg_final_resolution = (w, h);
            img.clone()
        };

        let xl_img = if xl_is_needed {
            if xl_resize_is_needed {
                let resize_factor = 2500.0 / w as f64;
                xl_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    xl_final_resolution.0,
                    xl_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                xl_final_resolution = (w, h);
                img.clone()
            }
        } else {
            xl_final_resolution = (w, h);
            img.clone()
        };

        // remember that the new aspect ratio is 1:2
        // let vt_img = if vt_is_needed {
        //     let new_img = &img;
        //
        //     if vt_resize_is_needed {
        //         let resize_factor = 2400.0 / h as f64;
        //
        //         let resized_w = (w as f64 * resize_factor) as u32;
        //         let resized_h = (h as f64 * resize_factor) as u32;
        //         let x_crop = (resized_w / 2) - (resized_h / 4);
        //
        //         vt_final_resolution = (resized_h / 2, resized_h);
        //
        //         new_img.resize(resized_w, resized_h, imageops::FilterType::Triangle).crop_imm(
        //             x_crop,
        //             0,
        //             resized_h / 2,
        //             resized_h,
        //         )
        //     } else {
        //         vt_final_resolution = (h / 2, h);
        //         let x_crop = (w / 2) - (h / 4);
        //         new_img.crop_imm(x_crop, 0, h / 2, h)
        //     }
        // } else {
        //     vt_final_resolution = (h / 2, h);
        //     img.clone()
        // };

        // medium encode and save
        if md_is_needed {
            let md_encoder: webp::Encoder = webp::Encoder::from_image(&md_img).unwrap();
            let md_webp: WebPMemory = md_encoder.encode(80f32);
            tokio::fs::write(&md_transcode_result_destination, &*md_webp).await?;
        }

        if lg_is_needed {
            let lg_encoder: webp::Encoder = webp::Encoder::from_image(&lg_img).unwrap();
            let lg_webp: WebPMemory = lg_encoder.encode(80f32);
            tokio::fs::write(&lg_transcode_result_destination, &*lg_webp).await?;
        }

        if xl_is_needed {
            let xl_encoder: webp::Encoder = webp::Encoder::from_image(&xl_img).unwrap();
            let xl_webp: WebPMemory = xl_encoder.encode(80f32);
            tokio::fs::write(&xl_transcode_result_destination, &*xl_webp).await?;
        }

        // if vt_is_needed {
        //     let vt_encoder: webp::Encoder = webp::Encoder::from_image(&vt_img).unwrap();
        //     let vt_webp: WebPMemory = vt_encoder.encode(80f32);
        //     tokio::fs::write(&vt_transcode_result_destination, &*vt_webp).await?;
        // }

        println!("wrote webp successfully");
        /////////////////////////////////////////////////////////////
        ///////////////// Upload to S3 //////////////////////////////
        /////////////////////////////////////////////////////////////

        let md_transcode_result_destination = md_transcode_result_destination.to_string_lossy().to_string();
        let lg_transcode_result_destination = lg_transcode_result_destination.to_string_lossy().to_string();
        let xl_transcode_result_destination = xl_transcode_result_destination.to_string_lossy().to_string();
        // let vt_transcode_result_destination = vt_transcode_result_destination.to_string_lossy().to_string();

        let (_a, _b, _c) = tokio::join!(
            // medium upload
            upload_object(
                &client,
                &bucket_name,
                &md_cdn_destination_path,
                &md_transcode_result_destination,
                md_is_needed,
            ),
            // lg upload
            upload_object(
                &client,
                &bucket_name,
                &lg_cdn_destination_path,
                &lg_transcode_result_destination,
                lg_is_needed,
            ),
            // xl upload
            upload_object(
                &client,
                &bucket_name,
                &xl_cdn_destination_path,
                &xl_transcode_result_destination,
                xl_is_needed,
            ),
            // vt upload
            // upload_object(
            //     &client,
            //     &bucket_name,
            //     &vt_cdn_destination_path,
            //     &vt_transcode_result_destination,
            //     vt_is_needed,
            // ),
        );

        /////////////////////////////////////////////////////////////
        ///////////////// CREATE DB DATA ////////////////////////////
        /////////////////////////////////////////////////////////////

        let painting = Painting {
            id,
            category_id: data.category_id,
            idx: data.idx,
            name: data.name,
            medium: data.medium,
            dimensions: "5' x 5'".to_string(),

            img_md: md_cdn_destination_path,
            img_md_width: md_final_resolution.0 as i32,
            img_md_height: md_final_resolution.1 as i32,

            img_lg: if lg_is_needed { Some(lg_cdn_destination_path) } else { None },
            img_lg_width: if lg_is_needed { Some(lg_final_resolution.0 as i32) } else { None },
            img_lg_height: if lg_is_needed { Some(lg_final_resolution.1 as i32) } else { None },

            img_xl: if xl_is_needed { Some(xl_cdn_destination_path) } else { None },
            img_xl_width: if xl_is_needed { Some(xl_final_resolution.0 as i32) } else { None },
            img_xl_height: if xl_is_needed { Some(xl_final_resolution.1 as i32) } else { None },
            // img_vt: if vt_is_needed { Some(vt_cdn_destination_path) } else { None },
            // img_vt_width: if vt_is_needed { Some(vt_final_resolution.0 as i32) } else { None },
            // img_vt_height: if vt_is_needed { Some(vt_final_resolution.1 as i32) } else { None },
        };

        sqlx::query_as!(
            Painting,
            r#"
            INSERT INTO paintings (
                id,
                category_id,
                idx,
                name,
                medium,
                dimensions,
                img_md,
                img_md_width,
                img_md_height,
                img_lg,
                img_lg_width,
                img_lg_height,
                img_xl,
                img_xl_width,
                img_xl_height
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
            ON CONFLICT (id)
            DO UPDATE set
                id = EXCLUDED.id,
                category_id = EXCLUDED.category_id,
                idx = EXCLUDED.idx,
                name = EXCLUDED.name,
                medium = EXCLUDED.medium,
                dimensions = EXCLUDED.dimensions,
                img_md = EXCLUDED.img_md,
                img_md_width = EXCLUDED.img_md_width,
                img_md_height = EXCLUDED.img_md_height,
                img_lg = EXCLUDED.img_lg,
                img_lg_width = EXCLUDED.img_lg_width,
                img_lg_height = EXCLUDED.img_lg_height,
                img_xl = EXCLUDED.img_xl,
                img_xl_width = EXCLUDED.img_xl_width,
                img_xl_height = EXCLUDED.img_xl_height
            RETURNING
                id,
                category_id,
                idx,
                name,
                medium,
                dimensions,
                img_md,
                img_md_width,
                img_md_height,
                img_lg,
                img_lg_width,
                img_lg_height,
                img_xl,
                img_xl_width,
                img_xl_height
            "#,
            painting.id,
            painting.category_id,
            painting.idx,
            painting.name,
            painting.medium,
            painting.dimensions,
            painting.img_md,
            painting.img_md_width,
            painting.img_md_height,
            painting.img_lg,
            painting.img_lg_width,
            painting.img_lg_height,
            painting.img_xl,
            painting.img_xl_width,
            painting.img_xl_height,
        )
        .fetch_one(&pool)
        .await?;

        println!("sent {} to the database", painting.name);
    }

    Ok(())
}

async fn create_dummy_slideshow() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// CONNECT TO DATABASE ///////////////////////
    /////////////////////////////////////////////////////////////

    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    /////////////////////////////////////////////////////////////
    ///////////////// CREATE S3 Client //////////////////////////
    /////////////////////////////////////////////////////////////

    let bucket_name = std::env::var("S3_BUCKET_NAME")?;
    let account_id = std::env::var("S3_ACCOUNT_ID")?;
    let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
    let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &access_key_id,
            &access_key_secret,
            None, // session token is not used with R2
            None,
            "R2",
        ))
        .region("auto") // Required by SDK but not used by R2
        .load()
        .await;
    let client = s3::Client::new(&config);

    /////////////////////////////////////////////////////////////
    ///////////////// TRANSCODE IMAGES //////////////////////////
    /////////////////////////////////////////////////////////////
    let dummy_dir = std::env::var("DUMMY_DIR")?;
    let simple_filenames = ["slideshow_001", "slideshow_002", "slideshow_003"];

    for (index, filename) in simple_filenames.into_iter().enumerate() {
        println!("transcoding: {:#?}", filename);

        let image_path_to_be_transcoded = format!("{dummy_dir}/slideshow/{filename}.jpg");
        let img = image::ImageReader::open(&image_path_to_be_transcoded)?.decode()?;
        let (w, h) = img.dimensions();

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
        let md_simple_filename = format!("md_{filename}");
        let lg_simple_filename = format!("lg_{filename}");
        let xl_simple_filename = format!("xl_{filename}");
        let vt_simple_filename = format!("vt_{filename}");

        // Transcode Result Destination for all sizes
        let md_transcode_result_destination = Path::new(".temp").join(&md_simple_filename).with_extension("webp");
        let lg_transcode_result_destination = Path::new(".temp").join(&lg_simple_filename).with_extension("webp");
        let xl_transcode_result_destination = Path::new(".temp").join(&xl_simple_filename).with_extension("webp");
        let vt_transcode_result_destination = Path::new(".temp").join(&vt_simple_filename).with_extension("webp");

        // S3 Bucket Target Path
        let s3_prefix = "slideshow";
        let md_cdn_destination_path = format!("{s3_prefix}/{md_simple_filename}.webp");
        let lg_cdn_destination_path = format!("{s3_prefix}/{lg_simple_filename}.webp");
        let xl_cdn_destination_path = format!("{s3_prefix}/{xl_simple_filename}.webp");
        let vt_cdn_destination_path = format!("{s3_prefix}/{vt_simple_filename}.webp");

        // Calculate all size factors if required
        let (md_final_resolution, lg_final_resolution, xl_final_resolution, vt_final_resolution);

        let md_img = if md_is_needed {
            if md_resize_is_needed {
                let resize_factor = 700.0 / w as f64;

                md_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    md_final_resolution.0,
                    md_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                md_final_resolution = (w, h);
                img.clone()
            }
        } else {
            md_final_resolution = (w, h);
            img.clone()
        };

        let lg_img = if lg_is_needed {
            if lg_resize_is_needed {
                let resize_factor = 1600.0 / w as f64;

                lg_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);

                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    lg_final_resolution.0,
                    lg_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                lg_final_resolution = (w, h);
                img.clone()
            }
        } else {
            lg_final_resolution = (w, h);
            img.clone()
        };

        let xl_img = if xl_is_needed {
            if xl_resize_is_needed {
                let resize_factor = 2500.0 / w as f64;
                xl_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    xl_final_resolution.0,
                    xl_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                xl_final_resolution = (w, h);
                img.clone()
            }
        } else {
            xl_final_resolution = (w, h);
            img.clone()
        };

        // remember that the new aspect ratio is 1:2
        let vt_img = if vt_is_needed {
            let new_img = &img;

            if vt_resize_is_needed {
                let resize_factor = 2400.0 / h as f64;

                let resized_w = (w as f64 * resize_factor) as u32;
                let resized_h = (h as f64 * resize_factor) as u32;
                let x_crop = (resized_w / 2) - (resized_h / 4);

                vt_final_resolution = (resized_h / 2, resized_h);

                new_img.resize(resized_w, resized_h, imageops::FilterType::Triangle).crop_imm(
                    x_crop,
                    0,
                    resized_h / 2,
                    resized_h,
                )
            } else {
                vt_final_resolution = (h / 2, h);
                let x_crop = (w / 2) - (h / 4);
                new_img.crop_imm(x_crop, 0, h / 2, h)
            }
        } else {
            vt_final_resolution = (h / 2, h);
            img.clone()
        };

        // medium encode and save
        if md_is_needed {
            let md_encoder: webp::Encoder = webp::Encoder::from_image(&md_img).unwrap();
            let md_webp: WebPMemory = md_encoder.encode(80f32);
            tokio::fs::write(&md_transcode_result_destination, &*md_webp).await?;
        }

        if lg_is_needed {
            let lg_encoder: webp::Encoder = webp::Encoder::from_image(&lg_img).unwrap();
            let lg_webp: WebPMemory = lg_encoder.encode(80f32);
            tokio::fs::write(&lg_transcode_result_destination, &*lg_webp).await?;
        }

        if xl_is_needed {
            let xl_encoder: webp::Encoder = webp::Encoder::from_image(&xl_img).unwrap();
            let xl_webp: WebPMemory = xl_encoder.encode(80f32);
            tokio::fs::write(&xl_transcode_result_destination, &*xl_webp).await?;
        }

        if vt_is_needed {
            let vt_encoder: webp::Encoder = webp::Encoder::from_image(&vt_img).unwrap();
            let vt_webp: WebPMemory = vt_encoder.encode(80f32);
            tokio::fs::write(&vt_transcode_result_destination, &*vt_webp).await?;
        }

        println!("wrote webp successfully");
        /////////////////////////////////////////////////////////////
        ///////////////// Upload to S3 //////////////////////////////
        /////////////////////////////////////////////////////////////

        let md_transcode_result_destination = md_transcode_result_destination.to_string_lossy().to_string();
        let lg_transcode_result_destination = lg_transcode_result_destination.to_string_lossy().to_string();
        let xl_transcode_result_destination = xl_transcode_result_destination.to_string_lossy().to_string();
        let vt_transcode_result_destination = vt_transcode_result_destination.to_string_lossy().to_string();

        let (_a, _b, _c, _d) = tokio::join!(
            // medium upload
            upload_object(
                &client,
                &bucket_name,
                &md_cdn_destination_path,
                &md_transcode_result_destination,
                md_is_needed,
            ),
            // lg upload
            upload_object(
                &client,
                &bucket_name,
                &lg_cdn_destination_path,
                &lg_transcode_result_destination,
                lg_is_needed,
            ),
            // xl upload
            upload_object(
                &client,
                &bucket_name,
                &xl_cdn_destination_path,
                &xl_transcode_result_destination,
                xl_is_needed,
            ),
            // vt upload
            upload_object(
                &client,
                &bucket_name,
                &vt_cdn_destination_path,
                &vt_transcode_result_destination,
                vt_is_needed,
            ),
        );

        /////////////////////////////////////////////////////////////
        ///////////////// CREATE DB DATA ////////////////////////////
        /////////////////////////////////////////////////////////////

        let slideshow = Slideshow {
            id: uuid::Uuid::new_v4(),
            idx: index as i32,

            img_md: md_cdn_destination_path,
            img_md_width: md_final_resolution.0 as i32,
            img_md_height: md_final_resolution.1 as i32,

            img_lg: if lg_is_needed { Some(lg_cdn_destination_path) } else { None },
            img_lg_width: if lg_is_needed { Some(lg_final_resolution.0 as i32) } else { None },
            img_lg_height: if lg_is_needed { Some(lg_final_resolution.1 as i32) } else { None },

            img_xl: if xl_is_needed { Some(xl_cdn_destination_path) } else { None },
            img_xl_width: if xl_is_needed { Some(xl_final_resolution.0 as i32) } else { None },
            img_xl_height: if xl_is_needed { Some(xl_final_resolution.1 as i32) } else { None },

            img_vt: vt_cdn_destination_path,
            img_vt_width: vt_final_resolution.0 as i32,
            img_vt_height: vt_final_resolution.1 as i32,
        };

        sqlx::query_as!(
            Slideshow,
            r#"
            INSERT INTO slideshow (
                id,
                idx,
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id)
            DO UPDATE set
                id = EXCLUDED.id,
                idx = EXCLUDED.idx,
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
                id,
                idx,
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
            slideshow.id,
            slideshow.idx,
            slideshow.img_md,
            slideshow.img_md_width,
            slideshow.img_md_height,
            slideshow.img_lg,
            slideshow.img_lg_width,
            slideshow.img_lg_height,
            slideshow.img_xl,
            slideshow.img_xl_width,
            slideshow.img_xl_height,
            slideshow.img_vt,
            slideshow.img_vt_width,
            slideshow.img_vt_height
        )
        .fetch_one(&pool)
        .await?;

        println!("sent {} to the database", slideshow.id);
    }

    Ok(())
}
async fn create_dummy_categories() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// CONNECT TO DATABASE ///////////////////////
    /////////////////////////////////////////////////////////////

    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    /////////////////////////////////////////////////////////////
    ///////////////// CREATE S3 Client //////////////////////////
    /////////////////////////////////////////////////////////////

    let bucket_name = std::env::var("S3_BUCKET_NAME")?;
    let account_id = std::env::var("S3_ACCOUNT_ID")?;
    let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
    let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &access_key_id,
            &access_key_secret,
            None, // session token is not used with R2
            None,
            "R2",
        ))
        .region("auto") // Required by SDK but not used by R2
        .load()
        .await;
    let client = s3::Client::new(&config);

    /////////////////////////////////////////////////////////////
    ///////////////// TRANSCODE IMAGES //////////////////////////
    /////////////////////////////////////////////////////////////
    let dummy_dir = std::env::var("DUMMY_DIR")?;
    let simple_filenames =
        ["wildlife", "day-of-the-dead", "landscapes", "figurative-portraits", "illustrations", "book-art"];

    let category_pretty_names = [
        "Wildlife",
        "Day of the Dead",
        "Landscapes & Cityscapes",
        "Figurative & Portraits",
        "Illustrations",
        "Book Art",
    ];

    for (index, filename) in simple_filenames.into_iter().enumerate() {
        println!("transcoding: {:#?}", filename);

        let image_path_to_be_transcoded = format!("{dummy_dir}/categories/{filename}.jpg");
        let img = image::ImageReader::open(&image_path_to_be_transcoded)?.decode()?;
        let (w, h) = img.dimensions();

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
        let md_simple_filename = format!("md_{filename}");
        let lg_simple_filename = format!("lg_{filename}");
        let xl_simple_filename = format!("xl_{filename}");
        let vt_simple_filename = format!("vt_{filename}");

        // Transcode Result Destination for all sizes
        let md_transcode_result_destination = Path::new(".temp").join(&md_simple_filename).with_extension("webp");
        let lg_transcode_result_destination = Path::new(".temp").join(&lg_simple_filename).with_extension("webp");
        let xl_transcode_result_destination = Path::new(".temp").join(&xl_simple_filename).with_extension("webp");
        let vt_transcode_result_destination = Path::new(".temp").join(&vt_simple_filename).with_extension("webp");

        // S3 Bucket Target Path
        let s3_prefix = "categories";
        let md_cdn_destination_path = format!("{s3_prefix}/{md_simple_filename}.webp");
        let lg_cdn_destination_path = format!("{s3_prefix}/{lg_simple_filename}.webp");
        let xl_cdn_destination_path = format!("{s3_prefix}/{xl_simple_filename}.webp");
        let vt_cdn_destination_path = format!("{s3_prefix}/{vt_simple_filename}.webp");

        // Calculate all size factors if required
        let (md_final_resolution, lg_final_resolution, xl_final_resolution, vt_final_resolution);

        let md_img = if md_is_needed {
            if md_resize_is_needed {
                let resize_factor = 700.0 / w as f64;

                md_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    md_final_resolution.0,
                    md_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                md_final_resolution = (w, h);
                img.clone()
            }
        } else {
            md_final_resolution = (w, h);
            img.clone()
        };

        let lg_img = if lg_is_needed {
            if lg_resize_is_needed {
                let resize_factor = 1600.0 / w as f64;

                lg_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);

                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    lg_final_resolution.0,
                    lg_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                lg_final_resolution = (w, h);
                img.clone()
            }
        } else {
            lg_final_resolution = (w, h);
            img.clone()
        };

        let xl_img = if xl_is_needed {
            if xl_resize_is_needed {
                let resize_factor = 2500.0 / w as f64;
                xl_final_resolution = ((w as f64 * resize_factor) as u32, (h as f64 * resize_factor) as u32);
                DynamicImage::ImageRgba8(imageops::resize(
                    &img,
                    xl_final_resolution.0,
                    xl_final_resolution.1,
                    imageops::FilterType::Triangle,
                ))
            } else {
                xl_final_resolution = (w, h);
                img.clone()
            }
        } else {
            xl_final_resolution = (w, h);
            img.clone()
        };

        // remember that the new aspect ratio is 1:2
        let vt_img = if vt_is_needed {
            let new_img = &img;

            if vt_resize_is_needed {
                let resize_factor = 2400.0 / h as f64;

                let resized_w = (w as f64 * resize_factor) as u32;
                let resized_h = (h as f64 * resize_factor) as u32;
                let x_crop = (resized_w / 2) - (resized_h / 4);

                vt_final_resolution = (resized_h / 2, resized_h);

                new_img.resize(resized_w, resized_h, imageops::FilterType::Triangle).crop_imm(
                    x_crop,
                    0,
                    resized_h / 2,
                    resized_h,
                )
            } else {
                vt_final_resolution = (h / 2, h);
                let x_crop = (w / 2) - (h / 4);
                new_img.crop_imm(x_crop, 0, h / 2, h)
            }
        } else {
            vt_final_resolution = (h / 2, h);
            img.clone()
        };

        // medium encode and save
        if md_is_needed {
            let md_encoder: webp::Encoder = webp::Encoder::from_image(&md_img).unwrap();
            let md_webp: WebPMemory = md_encoder.encode(80f32);
            tokio::fs::write(&md_transcode_result_destination, &*md_webp).await?;
        }

        if lg_is_needed {
            let lg_encoder: webp::Encoder = webp::Encoder::from_image(&lg_img).unwrap();
            let lg_webp: WebPMemory = lg_encoder.encode(80f32);
            tokio::fs::write(&lg_transcode_result_destination, &*lg_webp).await?;
        }

        if xl_is_needed {
            let xl_encoder: webp::Encoder = webp::Encoder::from_image(&xl_img).unwrap();
            let xl_webp: WebPMemory = xl_encoder.encode(80f32);
            tokio::fs::write(&xl_transcode_result_destination, &*xl_webp).await?;
        }

        if vt_is_needed {
            let vt_encoder: webp::Encoder = webp::Encoder::from_image(&vt_img).unwrap();
            let vt_webp: WebPMemory = vt_encoder.encode(80f32);
            tokio::fs::write(&vt_transcode_result_destination, &*vt_webp).await?;
        }

        println!("wrote webp successfully");
        /////////////////////////////////////////////////////////////
        ///////////////// Upload to S3 //////////////////////////////
        /////////////////////////////////////////////////////////////

        let md_transcode_result_destination = md_transcode_result_destination.to_string_lossy().to_string();
        let lg_transcode_result_destination = lg_transcode_result_destination.to_string_lossy().to_string();
        let xl_transcode_result_destination = xl_transcode_result_destination.to_string_lossy().to_string();
        let vt_transcode_result_destination = vt_transcode_result_destination.to_string_lossy().to_string();

        let (_a, _b, _c, _d) = tokio::join!(
            // medium upload
            upload_object(
                &client,
                &bucket_name,
                &md_cdn_destination_path,
                &md_transcode_result_destination,
                md_is_needed,
            ),
            // lg upload
            upload_object(
                &client,
                &bucket_name,
                &lg_cdn_destination_path,
                &lg_transcode_result_destination,
                lg_is_needed,
            ),
            // xl upload
            upload_object(
                &client,
                &bucket_name,
                &xl_cdn_destination_path,
                &xl_transcode_result_destination,
                xl_is_needed,
            ),
            // vt upload
            upload_object(
                &client,
                &bucket_name,
                &vt_cdn_destination_path,
                &vt_transcode_result_destination,
                vt_is_needed,
            ),
        );

        /////////////////////////////////////////////////////////////
        ///////////////// CREATE DB DATA ////////////////////////////
        /////////////////////////////////////////////////////////////

        let category = Category {
            category_id: index as i32,
            idx: index as i32,
            name: category_pretty_names[index].to_string(),

            img_md: md_cdn_destination_path,
            img_md_width: md_final_resolution.0 as i32,
            img_md_height: md_final_resolution.1 as i32,

            img_lg: if lg_is_needed { Some(lg_cdn_destination_path) } else { None },
            img_lg_width: if lg_is_needed { Some(lg_final_resolution.0 as i32) } else { None },
            img_lg_height: if lg_is_needed { Some(lg_final_resolution.1 as i32) } else { None },

            img_xl: if xl_is_needed { Some(xl_cdn_destination_path) } else { None },
            img_xl_width: if xl_is_needed { Some(xl_final_resolution.0 as i32) } else { None },
            img_xl_height: if xl_is_needed { Some(xl_final_resolution.1 as i32) } else { None },

            img_vt: if vt_is_needed { Some(vt_cdn_destination_path) } else { None },
            img_vt_width: if vt_is_needed { Some(vt_final_resolution.0 as i32) } else { None },
            img_vt_height: if vt_is_needed { Some(vt_final_resolution.1 as i32) } else { None },
        };

        sqlx::query_as!(
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
        .fetch_one(&pool)
        .await?;

        println!("sent {} to the database", category.name);
    }

    Ok(())
}

// async fn create_dummy_slideshow() -> Result<(), Error> {
//     /////////////////////////////////////////////////////////////
//     ///////////////// CREATE DATABSE POOL ///////////////////////
//     /////////////////////////////////////////////////////////////
//     let db_connection_string = std::env::var("DATABASE_URL").unwrap();
//     let pool = PgPoolOptions::new()
//         .max_connections(5)
//         .acquire_timeout(Duration::from_secs(3))
//         .connect(&db_connection_string)
//         .await
//         .expect("can't connect to database");
//
//     /////////////////////////////////////////////////////////////
//     ///////////////// S3 PREPARATION ////////////////////////////
//     /////////////////////////////////////////////////////////////
//
//     let bucket_name = std::env::var("S3_BUCKET_NAME")?;
//     let account_id = std::env::var("S3_ACCOUNT_ID")?;
//     let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
//     let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
//     let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
//         .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
//         .credentials_provider(aws_sdk_s3::config::Credentials::new(
//             &access_key_id,
//             &access_key_secret,
//             None, // session token is not used with R2
//             None,
//             "R2",
//         ))
//         .region("auto") // Required by SDK but not used by R2
//         .load()
//         .await;
//     let client = s3::Client::new(&config);
//
//     /////////////////////////////////////////////////////////////
//     ///////////////// NEW WEBP PROCESSING ///////////////////////
//     /////////////////////////////////////////////////////////////
//     let dummy_dir = std::env::var("DUMMY_DIR")?;
//     let images_paths_to_process = [
//         format!("{dummy_dir}/slideshow/slideshow_001.jpg"),
//         format!("{dummy_dir}/slideshow/slideshow_002.jpg"),
//         format!("{dummy_dir}/slideshow/slideshow_003.jpg"),
//     ];
//
//     for (index, image_path) in images_paths_to_process.into_iter().enumerate() {
//         println!("{:#?}", image_path);
//         let id = uuid::Uuid::new_v4();
//
//         let mut img = image::ImageReader::open(&image_path)?.decode()?;
//
//         let (w, h) = img.dimensions();
//         let size_factor = get_slideshow_scale_factor((w, h));
//         if size_factor > 1.0 {
//             img = DynamicImage::ImageRgba8(imageops::resize(
//                 &img,
//                 (w as f64 / size_factor) as u32,
//                 (h as f64 / size_factor) as u32,
//                 imageops::FilterType::Triangle,
//             ));
//         }
//
//         let encoder: webp::Encoder = webp::Encoder::from_image(&img).unwrap();
//         let webp: WebPMemory = encoder.encode(80f32);
//         let transcode_path = Path::new(".temp").join(id.to_string()).with_extension("webp");
//         println!("transcode_path: {transcode_path:?}");
//         tokio::fs::write(&transcode_path, &*webp).await?;
//         println!("wront webp successfully");
//
//         /////////////////////////////////////////////////////////////
//         ///////////////// UPLOAD TO CDN /////////////////////////////
//         /////////////////////////////////////////////////////////////
//         let cdn_destination_path = format!("slideshow/{}.webp", id);
//         let upload =
//             upload_object(&client, &bucket_name, &cdn_destination_path, &transcode_path.to_string_lossy()).await;
//         let full_html_path = format!("https://cdn.dansansouci.com/{}", cdn_destination_path);
//         if upload.is_ok() {
//             /////////////////////////////////////////////////////////////
//             ///////////////// SEND INSERT QUERY TO DATABASE /////////////
//             /////////////////////////////////////////////////////////////
//             let slideshow =
//                 Slideshow { id, idx: index as i32, src_img: full_html_path, width: w as i32, height: h as i32 };
//
//             let db_slideshow = sqlx::query_as!(
//                 Slideshow,
//                 r#"
//                 INSERT INTO slideshow (
//                     id,
//                     idx,
//                     src_img,
//                     width,
//                     height
//                 )
//                 VALUES ($1, $2, $3, $4, $5)
//                 RETURNING
//                     id,
//                     idx,
//                     src_img,
//                     width,
//                     height
//                 "#,
//                 slideshow.id,
//                 slideshow.idx,
//                 slideshow.src_img,
//                 slideshow.width,
//                 slideshow.height,
//             )
//             .fetch_one(&pool)
//             .await;
//
//             if db_slideshow.is_ok() {
//                 println!("Slideshow is sent to database")
//             };
//         }
//     }
//     Ok(())
// }

// async fn create_dummy_paintings() -> Result<(), Error> {
//     /////////////////////////////////////////////////////////////
//     ///////////////// CREATE DATABSE POOL ///////////////////////
//     /////////////////////////////////////////////////////////////
//     let db_connection_string = std::env::var("DATABASE_URL").unwrap();
//     let pool = PgPoolOptions::new()
//         .max_connections(5)
//         .acquire_timeout(Duration::from_secs(3))
//         .connect(&db_connection_string)
//         .await
//         .expect("can't connect to database");
//
//     /////////////////////////////////////////////////////////////
//     ///////////////// S3 PREPARATION ////////////////////////////
//     /////////////////////////////////////////////////////////////
//
//     let bucket_name = std::env::var("S3_BUCKET_NAME")?;
//     let account_id = std::env::var("S3_ACCOUNT_ID")?;
//     let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
//     let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
//     let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
//         .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
//         .credentials_provider(aws_sdk_s3::config::Credentials::new(
//             &access_key_id,
//             &access_key_secret,
//             None, // session token is not used with R2
//             None,
//             "R2",
//         ))
//         .region("auto") // Required by SDK but not used by R2
//         .load()
//         .await;
//     let client = s3::Client::new(&config);
//
//     /////////////////////////////////////////////////////////////
//     ///////////////// JSON DUMMY DATA PREP //////////////////////
//     /////////////////////////////////////////////////////////////
//
//     // parse dummy_data json
//     let file = tokio::fs::read_to_string("dummy_data/painting_descriptions.json").await.unwrap();
//     let data: Vec<PaintingDescription> = serde_json::from_str(&file)?;
//
//     for d in &data {
//         println!("{:#?}", d.name);
//         let dummy_dir = std::env::var("DUMMY_DIR")?;
//         let path_to_image_to_process = format!("{}{}", dummy_dir, d.path);
//
//         let id = uuid::Uuid::new_v4();
//         let transcode_path_hq = format!(".temp/{id}.jpg");
//         let transcode_path_thumb = format!(".temp/{id}_thumb.jpg");
//
//         let current_image = image::ImageReader::open(path_to_image_to_process)?.decode()?;
//         let dimensions_hq = current_image.dimensions();
//         let dimensions_thumb = create_thumbnail_dimensions(current_image.dimensions());
//
//         current_image.save_with_format(&transcode_path_hq, image::ImageFormat::Jpeg)?;
//         current_image
//             .resize_to_fill(dimensions_thumb.0, dimensions_thumb.1, FilterType::CatmullRom)
//             .save_with_format(&transcode_path_thumb, image::ImageFormat::Jpeg)?;
//
//         let cdn_path_hq = format!("paintings/{}.jpg", id);
//         let cdn_path_thumb = format!("paintings/{}_thumb.jpg", id);
//
//         let (upload_hq, upload_thumb) = tokio::join!(
//             upload_object(&client, &bucket_name, &cdn_path_hq, &transcode_path_hq),
//             upload_object(&client, &bucket_name, &cdn_path_thumb, &transcode_path_thumb),
//         );
//
//         let full_html_path_hq = format!("https://cdn.dansansouci.com/{}", cdn_path_hq);
//         let full_html_path_thumb = format!("https://cdn.dansansouci.com/{}", cdn_path_thumb);
//
//         if upload_hq.is_ok() && upload_thumb.is_ok() {
//             // let's get this sucker up on the database
//             let painting = Painting {
//                 id,
//                 idx: d.idx,
//                 name: d.name.to_string(),
//                 medium: d.medium.to_string(),
//                 src_thumb: full_html_path_thumb,
//                 src_thumb_width: dimensions_thumb.0 as i32,
//                 src_thumb_height: dimensions_thumb.1 as i32,
//                 src_img: full_html_path_hq,
//                 src_img_width: dimensions_hq.0 as i32,
//                 src_img_height: dimensions_hq.1 as i32,
//                 category_id: d.category_id,
//             };
//
//             let db_painting = sqlx::query_as!(
//                 Painting,
//                 r#"
//         INSERT INTO paintings (
//             id,
//             idx,
//             name,
//             medium,
//             src_thumb,
//             src_thumb_width,
//             src_thumb_height,
//             src_img,
//             src_img_width,
//             src_img_height,
//             category_id
//         )
//         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
//         RETURNING
//             id,
//             idx,
//             name,
//             medium,
//             src_thumb,
//             src_thumb_width,
//             src_thumb_height,
//             src_img,
//             src_img_width,
//             src_img_height,
//             category_id
//         "#,
//                 painting.id,
//                 painting.idx,
//                 painting.name,
//                 painting.medium,
//                 painting.src_thumb,
//                 painting.src_thumb_width,
//                 painting.src_thumb_height,
//                 painting.src_img,
//                 painting.src_img_width,
//                 painting.src_img_height,
//                 painting.category_id,
//             )
//             .fetch_one(&pool)
//             .await;
//
//             if db_painting.is_ok() {
//                 println!("Painting is sent to database")
//             };
//         }
//     }
//
//     Ok(())
// }

async fn upload_object(
    client: &s3::Client,
    bucket: &str,
    key: &str,
    file_path: &str,
    validation: bool,
) -> Result<(), s3::Error> {
    if validation {
        let body = ByteStream::from_path(Path::new(file_path)).await.unwrap();

        client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .content_type("image/webp")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await?;

        println!("Uploaded {} to {}/{}", file_path, bucket, key);
    }

    Ok(())
}

fn create_thumbnail_dimensions(dimensions: (u32, u32)) -> (u32, u32) {
    let width = dimensions.0;
    let height = dimensions.1;

    let float_width = dimensions.0 as f32;
    let float_height = dimensions.1 as f32;

    println!("original: {width}, {height}");

    if width < 400 && height < 400 {
        (width, height)
    } else if width >= height {
        let proportion = float_width / 400.0;
        let new_width = float_width / proportion;
        let new_height = float_height / proportion;

        println!("thumb: {}, {}", new_width, new_height);
        (new_width as u32, new_height as u32)
    } else {
        let proportion = height as f32 / 400.0;

        let new_width = width as f32 / proportion;
        let new_height = height as f32 / proportion;
        println!("thumb: {}, {}", new_width, new_height);
        (new_width as u32, new_height as u32)
    }
}

// async fn _transcode_and_save_image(
//     simple_filename: &str,
//     target_subdirectory: &str,
//     dynamic_image: &DynamicImage,
//     dummy_dir: &str,
//     size: Size,
// ) -> Result<(), Error> {
//     let image_path_to_be_transcoded = format!("{dummy_dir}/categories/{simple_filename}.jpg");
//     // let mut dynamic_image = image::ImageReader::open(&image_path_to_be_transcoded)?.decode()?;
//     let dimensions = dynamic_image.dimensions();
//
//     let size_suffix;
//     let source_dimension_to_test;
//     let target_dimension;
//     let target_simple_filename;
//     let allow_downres;
//
//     match size {
//         Md => {
//             size_suffix = "md";
//             source_dimension_to_test = dimensions.0;
//             target_dimension = 700;
//             target_simple_filename = format!("{size_suffix}_{simple_filename}").as_str();
//         }
//         Lg => {
//             size_suffix = "lg";
//             source_dimension_to_test = dimensions.0;
//             target_dimension = 1_600;
//             target_simple_filename = format!("{size_suffix}_{simple_filename}").as_str();
//         }
//         Xl => {
//             size_suffix = "xl";
//             source_dimension_to_test = dimensions.0.max(dimensions.1);
//             target_dimension = 1_600;
//             target_simple_filename = format!("{size_suffix}_{simple_filename}").as_str();
//         }
//         Vt => {
//             size_suffix = "vt";
//             source_dimension_to_test = dimensions.0.max(dimensions.1);
//             target_dimension = 1_600;
//             target_simple_filename = format!("{size_suffix}_{simple_filename}").as_str();
//         }
//     }
//
//     let transcode_result_destination =
//         Path::new(".temp").join(target_subdirectory).join(target_simple_filename).with_extension("webp");
//
//     // let size_factor = calculate_scale_factor_from_one_dimension(source_dimension_to_test, target_dimension, allow_downres)
//
//     let (w, h) = dynamic_image.dimensions();
//     let size_factor = get_slideshow_scale_factor((w, h));
//     if size_factor < 1.0 {
//         let dynamic_image = DynamicImage::ImageRgba8(imageops::resize(
//             dynamic_image,
//             (w as f64 / size_factor) as u32,
//             (h as f64 / size_factor) as u32,
//             imageops::FilterType::Triangle,
//         ));
//     }
//
//     let encoder: webp::Encoder = webp::Encoder::from_image(&dynamic_image).unwrap();
//     let webp: WebPMemory = encoder.encode(80f32);
//     tokio::fs::write(&transcode_result_destination, &*webp).await?;
//     println!("wrote webp successfully");
//
//     Ok(())
// }

fn calculate_scale_factor_from_one_dimension(source_dimension: u32, target_dimension: u32, allow_downres: bool) -> f64 {
    // never upres if image is smaller than target
    if source_dimension <= target_dimension {
        1.0
    } else {
        // if downresing is allowed, return downres scale factor
        if allow_downres { target_dimension as f64 / source_dimension as f64 } else { 1.0 }
    }
}

fn get_slideshow_scale_factor(dimensions: (u32, u32)) -> f64 {
    println!("original: {}, {}", dimensions.0, dimensions.1);
    let width = dimensions.0 as f64;
    let height = dimensions.1 as f64;

    if width < 2000.0 && height < 2000.0 {
        1.0
    } else if width >= height {
        println!("new_scale_factor: {}", width / 2500.0);
        width / 2500.0
    } else {
        println!("new_scale_factor: {}", height / 2500.0);
        height / 2500.0
    }
}
