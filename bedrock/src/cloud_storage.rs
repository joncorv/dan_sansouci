use anyhow::Error;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3 as s3;

pub async fn new() -> Result<s3::Client, Error> {
    // let bucket_name = std::env::var("S3_BUCKET_NAME")?;
    let account_id = std::env::var("S3_ACCOUNT_ID")?;
    let access_key_id = std::env::var("S3_ACCESS_KEY_ID")?;
    let access_key_secret = std::env::var("S3_ACCESS_KEY_SECRET")?;
    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
        .credentials_provider(s3::config::Credentials::new(
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

    Ok(client)
}
