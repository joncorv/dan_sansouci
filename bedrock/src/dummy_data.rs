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

pub async fn parse_args_create_dummy_data() -> Result<(), Error> {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dummy" => {
                println!("dummy mode created");

                create_dummy_data().await?;
            }
            _ => eprintln!("Unknown argument: {arg}"),
        }
    }

    Ok(())
}

async fn create_dummy_data() -> Result<(), Error> {
    // parse dummy_data json
    let file = tokio::fs::read_to_string("/Users/joncorv/Documents/dan_sansouci/bedrock/dummy_data/painting_descriptions.json").await?;
    let data: Vec<PaintingDescription> = serde_json::from_str(&file)?;

    // Create Database Connection
    let db_connection_string = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await
        .expect("can't connect to database");

    for d in &data.first() {
        println!("{:#?}", d.name);

        let id = uuid::Uuid::new_v4();
        let transcode_path_hq = format!(".temp/{id}.jpg");
        let transcode_path_thumb = format!(".temp/{id}_thumb.jpg");

        let current_image = image::ImageReader::open(&d.path)?.decode()?;
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

        let upload_hq = upload_object(&client, &bucket_name, format!("paintings/{transcode_path_hq}").as_str(), &transcode_path_hq).await;
        let upload_thumb =
            upload_object(&client, &bucket_name, format!("paintings/{transcode_path_thumb}").as_str(), &transcode_path_thumb).await;

        if upload_hq.is_ok() && upload_thumb.is_ok() {
            // let's get this sucker up on the database
            let painting = Painting {
                id: id,
                idx: d.idx,
                name: d.name.to_string(),
                medium: d.medium.to_string(),
                src_thumb: transcode_path_thumb,
                src_thumb_height: dimensions_thumb.0 as i32,
                src_thumb_width: dimensions_thumb.1 as i32,
                src_img: transcode_path_hq,
                src_img_height: dimensions_hq.0 as i32,
                src_img_width: dimensions_hq.1 as i32,
                category_id: d.category_id,
            };

            // sqlx::query_as!(Painting, "SELECT * FROM categories", painting.0, painting.1, painting.2, painting.3).fetch_one(&pool).await?;

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
