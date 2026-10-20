use anyhow::Error;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use image::GenericImageView;
use image::imageops::FilterType;
use std::path::Path;
use uuid::Uuid;

struct _Painting {
    id: Uuid,
    idx: Option<i32>,
    name: String,
    medium: Option<String>,
    src_thumb: Option<String>,
    src_thumb_height: Option<i32>,
    src_thumb_width: Option<i32>,
    src_img: Option<String>,
    src_img_height: Option<i32>,
    src_img_width: Option<i32>,
    category_id: Option<i32>,
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
    let id = uuid::Uuid::new_v4();
    let image_file =
        "/home/joncorv/Documents/dan_sansouci_cdn/day-of-the-dead/day-of-the-dead-0001.jpg"
            .to_string();

    let this_image = image::ImageReader::open(image_file)?.decode()?;
    let thumbnail_dimensions = create_thumbnail_dimensions(this_image.dimensions());
    this_image.save_with_format(format!(".temp/{id}.jpg"), image::ImageFormat::Jpeg)?;
    this_image
        .resize_to_fill(
            thumbnail_dimensions.0,
            thumbnail_dimensions.1,
            FilterType::CatmullRom,
        )
        .save_with_format(format!(".temp/{id}_thumb.jpg"), image::ImageFormat::Jpeg)?;

    // Create S3 Stuff
    let bucket_name = "sdk-example";
    // Provide your Cloudflare account ID
    let account_id = "<ACCOUNT_ID>";
    // Retrieve your S3 API credentials for your R2 bucket via API tokens
    // (see: https://developers.cloudflare.com/r2/api/tokens)
    let access_key_id = "<ACCESS_KEY_ID>";
    let access_key_secret = "<SECRET_ACCESS_KEY>";

    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key_id,
            access_key_secret,
            None, // session token is not used with R2
            None,
            "R2",
        ))
        .region("auto") // Required by SDK but not used by R2
        .load()
        .await;

    let client = s3::Client::new(&config);

    Ok(())
}

async fn _upload_object(
    client: &s3::Client,
    bucket: &str,
    key: &str,
    file_path: &str,
) -> Result<(), s3::Error> {
    let body = ByteStream::from_path(Path::new(file_path)).await.unwrap();

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
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
