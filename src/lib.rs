#![allow(clippy::result_large_err)]
use anyhow::{Context, Error};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::Client;
use bytes::Bytes;
use chrono::{DateTime, Local};
use regex::Regex;
use std::env;
use std::process;
use std::time::SystemTime;

fn get_env_var_or_exit(var_name: &str) -> String {
    match env::var(var_name) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Error: Environment variable {} must be set.", var_name);
            process::exit(1);
        }
    }
}

pub async fn build_s3_client(
    endpoint: &str,
    access_key_id_env: &str,
    secret_access_key_env: &str,
) -> Client {
    let credentials = Credentials::new(
        get_env_var_or_exit(access_key_id_env),
        get_env_var_or_exit(secret_access_key_env),
        None,
        None,
        "CustomProvider",
    );
    let config = aws_config::from_env()
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .load()
        .await;
    Client::new(&config)
}

async fn get_s3_object_bytes(client: &Client, bucket: &str, key: &str) -> Result<Bytes, Error> {
    let get_object_request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("Failed to fetch S3 key '{}'.", key))?;

    let aggregated_bytes = get_object_request
        .body
        .collect()
        .await
        .with_context(|| format!("Failed to aggregate bytes for S3 key '{}'.", key))?;

    Ok(aggregated_bytes.into_bytes())
}

async fn content_matches_regex(
    client: &Client,
    bucket: &str,
    key: &str,
    content_regex: &regex::Regex,
) -> Result<bool, Error> {
    let aggregated_bytes = get_s3_object_bytes(client, bucket, key).await?;
    let text = std::str::from_utf8(&aggregated_bytes)
        .with_context(|| format!("Failed to decode UTF-8 for S3 key '{}'.", key))?;
    Ok(content_regex.is_match(text))
}

async fn process_s3_list_objects_v2_response(
    resp: &ListObjectsV2Output,
    client: &Client,
    bucket: &str,
    key_pattern: &Option<Regex>,
    content_pattern: &Option<Regex>,
    n_keys: &mut usize,
    n_hits: &mut usize,
    keys: &mut Vec<String>,
    log_interval: usize,
    log_hits: bool,
    max_keys: Option<usize>,
    max_hits: Option<usize>,
    tolerate_response_error: bool,
    tolerate_key_error: bool,
) -> Result<Option<()>, Error> {
    let objects = match resp.contents() {
        Some(contents) => contents,
        None => {
            if tolerate_response_error {
                eprintln!("Ignored response error because --tolerate-response-error is specified.");
                return Ok(None);
            } else {
                return Err(anyhow::anyhow!("Failed to get contents from the response. Pass --tolerate-response-error to ignore such errors."));
            }
        }
    };

    for object in objects {
        let key = match object.key() {
            Some(k) => k,
            None => {
                if tolerate_key_error {
                    continue;
                } else {
                    return Err(anyhow::anyhow!("Failed to get key from an object. Pass --tolerate-key-error to ignore such errors."));
                }
            }
        };
        let mut key_matches = true;
        let mut content_matches = true;

        if let Some(pattern) = key_pattern {
            key_matches = pattern.is_match(&key);
        }

        if key_matches {
            content_matches = match content_pattern.as_ref() {
                Some(pattern) => match content_matches_regex(client, bucket, key, pattern).await {
                    Ok(result) => result,
                    Err(e) => {
                        if tolerate_key_error {
                            eprintln!("Ignored error for S3 key '{}' because --tolerate-key-error is specified. Error: {}", key, e);
                            false
                        } else {
                            return Err(e);
                        }
                    }
                },
                None => true,
            };
        }

        if key_matches && content_matches {
            keys.push(key.to_string());
            *n_hits += 1;
            if log_hits {
                eprintln!("Hit {}: {}", *n_hits, key);
            }
        }

        *n_keys += 1;

        if *n_keys % log_interval == 0 {
            let now: DateTime<Local> = SystemTime::now().into();
            eprintln!(
                "{} | Scanned {:>7} S3 keys, found {:>4} hits.",
                now.format("%Y-%m-%d %H:%M:%S %Z"),
                *n_keys,
                *n_hits
            );
        }

        if let Some(max) = max_keys {
            if *n_keys >= max {
                eprintln!("Reached the max_keys limit of {} S3 keys.", max);
                return Ok(Some(()));
            }
        }

        if let Some(max) = max_hits {
            if *n_hits >= max {
                eprintln!("Reached the max_n_hits limit of {} S3 keys.", max);
                return Ok(Some(()));
            }
        }
    }
    Ok(None)
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
    tolerate_response_error: bool,
    tolerate_key_error: bool,
) -> Result<Vec<String>, Error> {
    let mut continuation_token: Option<String> = None;
    let mut n_keys = 0;
    let mut n_hits = 0;
    let mut keys: Vec<String> = Vec::new();

    let key_pattern = key_regex
        .map(|r| Regex::new(r).with_context(|| format!("Failed to compile key regex: '{}'", r)))
        .transpose()?;

    let content_pattern = content_regex
        .map(|r| Regex::new(r).with_context(|| format!("Failed to compile content regex: '{}'", r)))
        .transpose()?;

    loop {
        let mut list_objects_request = client.list_objects_v2().bucket(bucket);

        if let Some(prefix) = prefix {
            list_objects_request = list_objects_request.prefix(prefix);
        }

        if let Some(token) = &continuation_token {
            list_objects_request = list_objects_request.continuation_token(token);
        }

        let resp = list_objects_request.send().await?;

        match process_s3_list_objects_v2_response(
            &resp,
            client,
            bucket,
            &key_pattern,
            &content_pattern,
            &mut n_keys,
            &mut n_hits,
            &mut keys,
            log_interval,
            log_hits,
            max_keys,
            max_hits,
            tolerate_response_error,
            tolerate_key_error,
        )
        .await
        {
            Ok(Some(_)) => {
                // Reached the max_keys or max_hits limit
                break;
            }
            Ok(None) => {}
            Err(e) => {
                return Err(e);
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

pub async fn test_access_to_bucket(client: &Client, bucket: &str) -> Result<(), Error> {
    client
        .list_objects_v2()
        .bucket(bucket)
        .max_keys(1)
        .send()
        .await?;
    Ok(())
}
