#![allow(clippy::result_large_err)]
use aws_sdk_s3::Client;
use std::{fs::File, io::Write, path::PathBuf, process::exit};
use tokio_stream::StreamExt;

/// Lists your buckets.
#[tokio::main]
pub async fn list_buckets(endpoint: &str, bucket: &str) -> Result<(), aws_sdk_s3::Error> {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;

    let client = Client::new(&config);

    let a = get_object(&client, &bucket, "my-key").await;

    // let resp = client.list_buckets().send().await?;
    // let buckets = resp.buckets().unwrap_or_default();
    // let num_buckets = buckets.len();

    // for bucket in buckets {
    //     println!("{}", bucket.name().unwrap_or_default());
    // }

    // println!();
    // println!("Found {num_buckets} buckets.");

    Ok(())
}

async fn get_object(client: &Client, bucket: &str, object: &str) -> Result<usize, anyhow::Error> {
    let mut object = client
        .get_object()
        .bucket(bucket)
        .key(object)
        .send()
        .await?;

    let mut byte_count = 0_usize;
    let mut file = File::create("output.txt")?;
    while let Some(bytes) = object.body.try_next().await? {
        let bytes = file.write(&bytes)?;
        byte_count += bytes;
    }

    Ok(byte_count)
}
