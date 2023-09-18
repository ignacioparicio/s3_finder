#![allow(clippy::result_large_err)]
// use aws_sdk_s3::error::SdkError;
// use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
use aws_sdk_s3::{Client, Error};
use bytes::Bytes;
use chrono::{DateTime, Local};
use regex::Regex;
use std::env;
use std::time::SystemTime;
// use std::{fs::File, io::Write};
// use tokio_stream::StreamExt;

pub fn check_env_var_exists(env_var: &str) {
    if env::var(env_var).is_err() {
        panic!("Missing environment variable {env_var}.");
    }
}

pub async fn build_s3_client(endpoint: &str) -> Client {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    Client::new(&config)
}

async fn get_s3_object_bytes(client: &Client, bucket: &str, key: &str) -> Bytes {
    let get_object_request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .unwrap();
    let aggregated_bytes = get_object_request.body.collect().await.unwrap();
    return aggregated_bytes.into_bytes();
}

async fn content_matches_regex(
    client: &Client,
    bucket: &str,
    key: &str,
    content_regex: &regex::Regex,
) -> bool {
    let aggregated_bytes = get_s3_object_bytes(client, bucket, key).await;
    let text = std::str::from_utf8(&aggregated_bytes).unwrap_or_default();
    content_regex.is_match(text)
}

pub async fn find_s3_keys(
    client: &Client,
    bucket: &str,
    prefix: Option<&str>,
    key_regex: Option<&str>,
    content_regex: Option<&str>,
    max_keys: Option<usize>,
    max_hits: Option<usize>,
    log_interval: usize,
    log_hits: bool,
) -> Result<Vec<String>, Error> {
    let mut continuation_token: Option<String> = None;
    let mut n_keys = 0;
    let mut n_hits = 0;
    let mut keys: Vec<String> = Vec::new();

    let key_pattern = key_regex.map(|r| Regex::new(r).unwrap());
    let content_pattern = content_regex.map(|r| Regex::new(r).unwrap());

    loop {
        let mut list_objects_request = client.list_objects_v2().bucket(bucket);

        if let Some(prefix) = prefix {
            list_objects_request = list_objects_request.prefix(prefix);
        }

        if let Some(token) = &continuation_token {
            list_objects_request = list_objects_request.continuation_token(token);
        }

        let resp = list_objects_request.send().await?;

        for object in resp.contents().unwrap_or_default() {
            let key = object.key().unwrap_or_default();
            let mut key_matches = true;
            let mut content_matches = true;

            if let Some(pattern) = &key_pattern {
                key_matches = pattern.is_match(&key);
            }

            if key_matches && content_pattern.is_some() {
                content_matches =
                    content_matches_regex(client, bucket, key, content_pattern.as_ref().unwrap())
                        .await;
            }

            if key_matches && content_matches {
                keys.push(key.to_string());
                n_hits += 1;
                if log_hits {
                    eprintln!("Hit {}: {}", n_hits, key);
                }
            }

            n_keys += 1;

            if n_keys % log_interval == 0 {
                let now: DateTime<Local> = SystemTime::now().into();
                eprintln!(
                    "{} | Scanned {:>7} S3 keys, found {:>4} hits.",
                    now.format("%Y-%m-%d %H:%M:%S %Z"),
                    n_keys,
                    n_hits
                );
            }

            if let Some(max) = max_keys {
                if n_keys >= max {
                    eprintln!("Reached the max_keys limit of {} S3 keys.", max);
                    return Ok(keys);
                }
            }

            if let Some(max) = max_hits {
                if n_hits >= max {
                    eprintln!("Reached the max_n_hits limit of {} S3 keys.", max);
                    return Ok(keys);
                }
            }
        }

        if resp.is_truncated() {
            continuation_token = resp.next_continuation_token().map(|s| s.to_string());
        } else {
            break;
        }
    }

    eprintln!("Processed {} S3 keys.", n_keys);

    Ok(keys)
}

pub async fn test_access_to_bucket(client: &Client, endpoint: &str, bucket: &str) {
    let resp = client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(1)
        .send()
        .await;

    match resp {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "Unable to access bucket {} at endpoint {}. Error: {}",
                bucket, endpoint, e
            );
            std::process::exit(1);
        }
    }
}
