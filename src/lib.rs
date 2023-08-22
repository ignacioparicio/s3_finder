#![allow(clippy::result_large_err)]
use aws_sdk_s3::{Client, Error};
use regex::Regex;
use std::env;
use std::{fs::File, io::Write};
use tokio_stream::StreamExt;

pub fn check_env_var_exists(env_var: &str) {
    if env::var(env_var).is_err() {
        panic!("Missing environment variable {env_var}.");
    }
}

pub async fn build_s3_client(endpoint: &str) -> Client {
    let config = aws_config::from_env().endpoint_url(endpoint).load().await;
    Client::new(&config)
}

pub async fn get_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<usize, anyhow::Error> {
    let mut object = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    let mut byte_n_keys = 0_usize;
    let mut file = File::create("output.txt")?;
    while let Some(bytes) = object.body.try_next().await? {
        let bytes = file.write(&bytes)?;
        byte_n_keys += bytes;
    }

    Ok(byte_n_keys)
}

pub async fn find_s3_keys(
    client: &Client,
    bucket: &str,
    prefix: Option<&str>,
    regex: Option<&str>,
    max_keys: Option<usize>,
    max_hits: Option<usize>,
) -> Result<Vec<String>, Error> {
    let mut continuation_token: Option<String> = None;
    let mut n_keys = 0;
    let mut n_hits = 0;
    let mut keys: Vec<String> = Vec::new();

    let regex_pattern = regex.map(|r| Regex::new(r).unwrap());

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

            if let Some(pattern) = &regex_pattern {
                if pattern.is_match(&key) {
                    keys.push(key.to_string());
                    n_hits += 1;
                }
            } else {
                keys.push(key.to_string());
                n_hits += 1;
            }

            n_keys += 1;

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

        // for object in resp.contents().unwrap_or_default() {
        //     println!("{}", object.key().unwrap_or_default());
        //     n_keys += 1;
        // }

        // if n_keys >= 3000 {
        //     println!("Reached the limit of 3,000 objects.");
        //     break;
        // }

        // Check if there are more objects to fetch
        if resp.is_truncated() {
            continuation_token = resp.next_continuation_token().map(|s| s.to_string());
        } else {
            break;
        }
    }

    eprintln!("Processed {} S3 keys.", n_keys);

    Ok(keys)
}

pub async fn test_access_to_bucket(client: &Client, bucket: &str) {
    client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(1)
        .send()
        .await
        .expect(&format!("No access to bucket {}", bucket));
}
