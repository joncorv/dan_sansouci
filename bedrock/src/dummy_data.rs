use anyhow::Error;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use image::GenericImageView;
use image::imageops::FilterType;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::fmt::Debug;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

struct Painting {
    id: Uuid,
    idx: i32,
    name: String,
    medium: String,
    src_thumb: String,
    src_thumb_height: i32,
    src_thumb_width: i32,
    src_img: String,
    src_img_height: i32,
    src_img_width: i32,
    category_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaintingDescription {
    path: String,
    idx: i32,
    name: String,
    medium: String,
    category_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Slideshow {
    id: Uuid,
    idx: i32,
    src_img: String,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Category {
    category_id: i32,
    idx: i32,
    name: String,
    src_img: String,
}

pub async fn parse_args_create_dummy_data() -> Result<(), Error> {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dummy" => {
                println!("dummy mode created");

                create_dummy_categories().await?;
                create_dummy_paintings().await?;
                create_dummy_slideshow().await?;
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
    let simple_filenames = [
        "wildlife.jpg",
        "day-of-the-dead.jpg",
        "landscapes.jpg",
        "figurative-portraits.jpg",
        "illustrations.jpg",
        "book-art.jpg",
    ];

    for filename in simple_filenames {
        println!("transcoding: {:#?}", filename);

        let image_path_to_be_transcoded = format!("{dummy_dir}/categories/{filename}");
        let transcode_result_destination = format!(".temp/{filename}");
        let current_image = image::ImageReader::open(image_path_to_be_transcoded)?.decode()?;
        // let dimensions_hq = current_image.dimensions();
        current_image.save_with_format(transcode_result_destination, image::ImageFormat::Jpeg)?;
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
        let file_to_upload = format!(".temp/{filename}");
        let cdn_destination_path = format!("categories/{filename}");
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
    let mut index: i32 = 0;
    let dummy_dir = std::env::var("DUMMY_DIR")?;
    let images_paths_to_process = [
        format!("{dummy_dir}/slideshow/slideshow_001.jpg"),
        format!("{dummy_dir}/slideshow/slideshow_002.jpg"),
        format!("{dummy_dir}/slideshow/slideshow_003.jpg"),
    ];
    // Create Database Connection
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    for image_path in images_paths_to_process {
        println!("{:#?}", image_path);
        let id = uuid::Uuid::new_v4();
        let transcode_path = format!(".temp/{id}.jpg");
        let current_image = image::ImageReader::open(&image_path)?.decode()?;
        let dimensions_hq = current_image.dimensions();
        current_image.save_with_format(&transcode_path, image::ImageFormat::Jpeg)?;

        // Create S3 Stuff
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
        let cdn_destination_path = format!("slideshow/{}.jpg", id);
        let upload_hq = upload_object(&client, &bucket_name, &cdn_destination_path, &transcode_path).await;
        let full_html_path = format!("https://cdn.dansansouci.com/{}", cdn_destination_path);
        if upload_hq.is_ok() {
            // let's get this sucker up on the database
            //
            let slideshow = Slideshow {
                id,
                idx: index,
                src_img: full_html_path,
                width: dimensions_hq.0 as i32,
                height: dimensions_hq.1 as i32,
            };

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
        index += 1;
    }
    Ok(())
}

async fn create_dummy_paintings() -> Result<(), Error> {
    // parse dummy_data json
    let file = tokio::fs::read_to_string("dummy_data/painting_descriptions.json").await.unwrap();
    let data: Vec<PaintingDescription> = serde_json::from_str(&file)?;

    // Create Database Connection
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

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

        // Create S3 Stuff
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
                src_thumb_height: dimensions_thumb.0 as i32,
                src_thumb_width: dimensions_thumb.1 as i32,
                src_img: full_html_path_hq,
                src_img_height: dimensions_hq.0 as i32,
                src_img_width: dimensions_hq.1 as i32,
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
            src_thumb_height,
            src_thumb_width,
            src_img,
            src_img_height,
            src_img_width,
            category_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            idx,
            name,
            medium,
            src_thumb,
            src_thumb_height,
            src_thumb_width,
            src_img,
            src_img_height,
            src_img_width,
            category_id
        "#,
                painting.id,
                painting.idx,
                painting.name,
                painting.medium,
                painting.src_thumb,
                painting.src_thumb_height,
                painting.src_thumb_width,
                painting.src_img,
                painting.src_img_height,
                painting.src_img_width,
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

    client.put_object().bucket(bucket).key(key).body(body).metadata("Content-Type", "image/jpeg").send().await?;

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
