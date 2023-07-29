#![allow(clippy::result_large_err)]
use aws_sdk_s3::{Client, Error};
use std::env;
use std::{fs::File, io::Write};
use tokio_stream::StreamExt;

pub fn check_env_var_exists(env_var: &str) {
    if env::var(env_var).is_err() {
        panic!("Missing environment variable {env_var}.");
    }
}

pub async fn build_s3_client(endpoint: &str) -> Result<Client, aws_sdk_s3::Error> {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    Ok(Client::new(&config))
}

pub async fn get_object(
    client: &Client,
    bucket: &str,
    object: &str,
) -> Result<usize, anyhow::Error> {
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

pub async fn list_s3_keys(
    client: &Client,
    bucket: &str,
    prefix: Option<&str>,
) -> Result<(), Error> {
    let mut list_objects_request = client.list_objects_v2().bucket(bucket);

    if let Some(prefix) = prefix {
        list_objects_request = list_objects_request.prefix(prefix);
    }

    let resp = list_objects_request.send().await?;

    for object in resp.contents().unwrap_or_default() {
        println!("{}", object.key().unwrap_or_default());
    }

    Ok(())
}

pub async fn test_access_to_bucket(client: &Client, bucket: &str) {
    list_s3_keys(client, bucket, None)
        .await
        .expect(&format!("No access to bucket {}", bucket));
}
