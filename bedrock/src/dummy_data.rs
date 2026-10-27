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
use uuid::Uuid;
use webp::Encoder;
use webp::WebPMemory;

use crate::datatypes::Category;
use crate::datatypes::Painting;
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
                // create_dummy_categories().await?;
                // create_dummy_paintings().await?;
            }
            _ => eprintln!("Unknown argument: {arg}"),
        }
    }

    Ok(())
}

async fn create_dummy_categories() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// BASIC DATA PREP ///////////////////////////
    /////////////////////////////////////////////////////////////
    let dummy_dir = std::env::var("DUMMY_DIR")?;
    let simple_filenames =
        ["wildlife", "day-of-the-dead", "landscapes", "figurative-portraits", "illustrations", "book-art"];

    for filename in simple_filenames {
        println!("transcoding: {:#?}", filename);

        let image_path_to_be_transcoded = format!("{dummy_dir}/categories/{filename}.jpg");
        let transcode_result_destination = Path::new(".temp").join(filename).with_extension("webp");

        let mut img = image::ImageReader::open(&image_path_to_be_transcoded)?.decode()?;

        let (w, h) = img.dimensions();
        let size_factor = get_slideshow_scale_factor((w, h));
        if size_factor > 1.0 {
            img = DynamicImage::ImageRgba8(imageops::resize(
                &img,
                (w as f64 / size_factor) as u32,
                (h as f64 / size_factor) as u32,
                imageops::FilterType::Triangle,
            ));
        }

        let encoder: webp::Encoder = webp::Encoder::from_image(&img).unwrap();
        let webp: WebPMemory = encoder.encode(80f32);
        tokio::fs::write(&transcode_result_destination, &*webp).await?;
        println!("wrote webp successfully");
    }

    /////////////////////////////////////////////////////////////
    ///////////////// Upload to S3 //////////////////////////////
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

    for filename in simple_filenames {
        let file_to_upload = format!(".temp/{filename}.webp");
        let cdn_destination_path = format!("categories/{filename}.webp");
        upload_object(&client, &bucket_name, &cdn_destination_path, &file_to_upload).await?;
    }
    /////////////////////////////////////////////////////////////
    ///////////////// CREATE DB DATA ////////////////////////////
    /////////////////////////////////////////////////////////////
    let cdn_prefix = "https://cdn.dansansouci.com/categories/".to_string();

    let categories = [
        Category {
            category_id: 0,
            idx: 0,
            name: "Wildlife".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[0]),
        },
        Category {
            category_id: 0,
            idx: 0,
            name: "Day of the Dead".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[1]),
        },
        Category {
            category_id: 0,
            idx: 0,
            name: "Landscapes & Cityscapes".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[2]),
        },
        Category {
            category_id: 0,
            idx: 0,
            name: "Figurative & Portraits".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[3]),
        },
        Category {
            category_id: 0,
            idx: 0,
            name: "Illustrations".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[4]),
        },
        Category {
            category_id: 0,
            idx: 0,
            name: "Book Art".to_string(),
            src_img: format!("{}{}", cdn_prefix, simple_filenames[5]),
        },
    ];
    /////////////////////////////////////////////////////////////
    ///////////////// SEND TO THE DATABSE ///////////////////////
    /////////////////////////////////////////////////////////////
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    for (index, category) in categories.into_iter().enumerate() {
        sqlx::query_as!(
            Category,
            r#"
            INSERT INTO categories (
                category_id,
                idx,
                name,
                src_img
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (category_id) 
            DO UPDATE set
                category_id = EXCLUDED.category_id,
                idx = EXCLUDED.idx,
                name = EXCLUDED.name,
                src_img = EXCLUDED.src_img
            RETURNING
                category_id,
                idx,
                name,
                src_img
            "#,
            index as i32,
            index as i32,
            category.name,
            category.src_img,
        )
        .fetch_one(&pool)
        .await?;

        println!("sent {} to the database", category.name);
    }

    Ok(())
}

async fn create_dummy_slideshow() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// CREATE DATABSE POOL ///////////////////////
    /////////////////////////////////////////////////////////////
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    /////////////////////////////////////////////////////////////
    ///////////////// S3 PREPARATION ////////////////////////////
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
    ///////////////// NEW WEBP PROCESSING ///////////////////////
    /////////////////////////////////////////////////////////////
    let dummy_dir = std::env::var("DUMMY_DIR")?;
    let images_paths_to_process = [
        format!("{dummy_dir}/slideshow/slideshow_001.jpg"),
        format!("{dummy_dir}/slideshow/slideshow_002.jpg"),
        format!("{dummy_dir}/slideshow/slideshow_003.jpg"),
    ];

    for (index, image_path) in images_paths_to_process.into_iter().enumerate() {
        println!("{:#?}", image_path);
        let id = uuid::Uuid::new_v4();

        let mut img = image::ImageReader::open(&image_path)?.decode()?;

        let (w, h) = img.dimensions();
        let size_factor = get_slideshow_scale_factor((w, h));
        if size_factor > 1.0 {
            img = DynamicImage::ImageRgba8(imageops::resize(
                &img,
                (w as f64 / size_factor) as u32,
                (h as f64 / size_factor) as u32,
                imageops::FilterType::Triangle,
            ));
        }

        let encoder: webp::Encoder = webp::Encoder::from_image(&img).unwrap();
        let webp: WebPMemory = encoder.encode(80f32);
        let transcode_path = Path::new(".temp").join(id.to_string()).with_extension("webp");
        println!("transcode_path: {transcode_path:?}");
        tokio::fs::write(&transcode_path, &*webp).await?;
        println!("wront webp successfully");

        /////////////////////////////////////////////////////////////
        ///////////////// UPLOAD TO CDN /////////////////////////////
        /////////////////////////////////////////////////////////////
        let cdn_destination_path = format!("slideshow/{}.webp", id);
        let upload =
            upload_object(&client, &bucket_name, &cdn_destination_path, &transcode_path.to_string_lossy()).await;
        let full_html_path = format!("https://cdn.dansansouci.com/{}", cdn_destination_path);
        if upload.is_ok() {
            /////////////////////////////////////////////////////////////
            ///////////////// SEND INSERT QUERY TO DATABASE /////////////
            /////////////////////////////////////////////////////////////
            let slideshow =
                Slideshow { id, idx: index as i32, src_img: full_html_path, width: w as i32, height: h as i32 };

            let db_slideshow = sqlx::query_as!(
                Slideshow,
                r#"
                INSERT INTO slideshow (
                    id,
                    idx,
                    src_img,
                    width,
                    height
                )
                VALUES ($1, $2, $3, $4, $5)
                RETURNING
                    id,
                    idx,
                    src_img,
                    width,
                    height
                "#,
                slideshow.id,
                slideshow.idx,
                slideshow.src_img,
                slideshow.width,
                slideshow.height,
            )
            .fetch_one(&pool)
            .await;

            if db_slideshow.is_ok() {
                println!("Slideshow is sent to database")
            };
        }
    }
    Ok(())
}

async fn create_dummy_paintings() -> Result<(), Error> {
    /////////////////////////////////////////////////////////////
    ///////////////// CREATE DATABSE POOL ///////////////////////
    /////////////////////////////////////////////////////////////
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    /////////////////////////////////////////////////////////////
    ///////////////// S3 PREPARATION ////////////////////////////
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
    ///////////////// JSON DUMMY DATA PREP //////////////////////
    /////////////////////////////////////////////////////////////

    // parse dummy_data json
    let file = tokio::fs::read_to_string("dummy_data/painting_descriptions.json").await.unwrap();
    let data: Vec<PaintingDescription> = serde_json::from_str(&file)?;

    for d in &data {
        println!("{:#?}", d.name);
        let dummy_dir = std::env::var("DUMMY_DIR")?;
        let path_to_image_to_process = format!("{}{}", dummy_dir, d.path);

        let id = uuid::Uuid::new_v4();
        let transcode_path_hq = format!(".temp/{id}.jpg");
        let transcode_path_thumb = format!(".temp/{id}_thumb.jpg");

        let current_image = image::ImageReader::open(path_to_image_to_process)?.decode()?;
        let dimensions_hq = current_image.dimensions();
        let dimensions_thumb = create_thumbnail_dimensions(current_image.dimensions());

        current_image.save_with_format(&transcode_path_hq, image::ImageFormat::Jpeg)?;
        current_image
            .resize_to_fill(dimensions_thumb.0, dimensions_thumb.1, FilterType::CatmullRom)
            .save_with_format(&transcode_path_thumb, image::ImageFormat::Jpeg)?;

        let cdn_path_hq = format!("paintings/{}.jpg", id);
        let cdn_path_thumb = format!("paintings/{}_thumb.jpg", id);

        let (upload_hq, upload_thumb) = tokio::join!(
            upload_object(&client, &bucket_name, &cdn_path_hq, &transcode_path_hq),
            upload_object(&client, &bucket_name, &cdn_path_thumb, &transcode_path_thumb),
        );

        let full_html_path_hq = format!("https://cdn.dansansouci.com/{}", cdn_path_hq);
        let full_html_path_thumb = format!("https://cdn.dansansouci.com/{}", cdn_path_thumb);

        if upload_hq.is_ok() && upload_thumb.is_ok() {
            // let's get this sucker up on the database
            let painting = Painting {
                id,
                idx: d.idx,
                name: d.name.to_string(),
                medium: d.medium.to_string(),
                src_thumb: full_html_path_thumb,
                src_thumb_width: dimensions_thumb.0 as i32,
                src_thumb_height: dimensions_thumb.1 as i32,
                src_img: full_html_path_hq,
                src_img_width: dimensions_hq.0 as i32,
                src_img_height: dimensions_hq.1 as i32,
                category_id: d.category_id,
            };

            let db_painting = sqlx::query_as!(
                Painting,
                r#"
        INSERT INTO paintings (
            id,
            idx,
            name,
            medium,
            src_thumb,
            src_thumb_width,
            src_thumb_height,
            src_img,
            src_img_width,
            src_img_height,
            category_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            idx,
            name,
            medium,
            src_thumb,
            src_thumb_width,
            src_thumb_height,
            src_img,
            src_img_width,
            src_img_height,
            category_id
        "#,
                painting.id,
                painting.idx,
                painting.name,
                painting.medium,
                painting.src_thumb,
                painting.src_thumb_width,
                painting.src_thumb_height,
                painting.src_img,
                painting.src_img_width,
                painting.src_img_height,
                painting.category_id,
            )
            .fetch_one(&pool)
            .await;

            if db_painting.is_ok() {
                println!("Painting is sent to database")
            };
        }
    }

    Ok(())
}

async fn upload_object(client: &s3::Client, bucket: &str, key: &str, file_path: &str) -> Result<(), s3::Error> {
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
