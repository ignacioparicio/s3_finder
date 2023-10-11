use clap::Parser;
use s3_finder::{build_s3_client, find_s3_keys, test_access_to_bucket};
/// CLI tool to search for S3 keys which match a regex and optionally contain a regex
#[derive(Parser, Debug)]
#[command(author, version="1.0.0", about, long_about = None)]
struct Args {
    /// S3 endpoint
    #[arg(short, long)]
    endpoint: String,

    /// S3 bucket
    #[arg(short, long)]
    bucket: String,

    /// Regex that S3 keys must match; for example, '.*photo\.jpg$' would match all keys ending in 'photo.jpg'
    #[arg(short, long)]
    key_regex: Option<String>,

    /// Regex that the content of the S3 key must match; for example, '\b2022-12-31\b' will match keys that contain the word '2022-12-31'; can be used in conjunction with --key-regex; this flag incurs a performance penalty because the entire contents of the S3 key must be loaded to memory
    #[arg(short, long)]
    content_regex: Option<String>,

    /// Optional prefix to search in the S3 bucket
    #[arg(short, long)]
    prefix: Option<String>,

    /// Optional maximum number of keys to process before stopping
    #[arg(long)]
    max_keys: Option<usize>,

    /// Optional maximum number of hits to find before stopping
    #[arg(long)]
    max_hits: Option<usize>,

    /// Optional logging interval
    #[arg(long, default_value_t = 10000)]
    log_interval: usize,

    /// Whether to log each hit in stderr
    #[arg(long, default_value_t = false)]
    log_hits: bool,

    /// Whether to continue execution if an error occurs processing a ListObjectsV2 response. Each ignored response can affect up to 1,000 objects.
    #[arg(long, default_value_t = true)]
    tolerate_response_error: bool,

    /// Whether to continue execution if an error occurs processing a single S3 key.
    #[arg(long, default_value_t = true)]
    tolerate_key_error: bool,

    /// Environment variable containing the S3 access key ID
    #[arg(long, default_value_t = String::from("AWS_ACCESS_KEY_ID"))]
    access_key_id_env_param: String,

    /// Environment variable containing the S3 secret access key
    #[arg(long, default_value_t = String::from("AWS_SECRET_ACCESS_KEY"))]
    secret_access_key_env_param: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.content_regex.is_some() && args.key_regex.is_none() {
        eprintln!("Warning: --content-regex is specified without --key-regex. This may lead to scanning content in binary files, negatively impacting performance. Consider using --key-regex to refine file selection.");
    }

    if args.content_regex.is_none() && args.key_regex.is_none() {
        eprintln!("Warning: Neither --key-regex nor --content-regex is specified. This will list all keys in the specified bucket/prefix combination.");
    }

    let client = build_s3_client(
        &args.endpoint,
        &args.access_key_id_env_param,
        &args.secret_access_key_env_param,
    )
    .await;

    match test_access_to_bucket(&client, &args.bucket).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "Unable to access bucket {} at endpoint {}. Error: {}",
                &args.bucket, &args.endpoint, e
            );
            std::process::exit(1);
        }
    }

    let result = find_s3_keys(
        &client,
        &args.bucket,
        args.prefix.as_deref(),
        args.key_regex.as_deref(),
        args.content_regex.as_deref(),
        args.max_keys,
        args.max_hits,
        args.log_interval,
        args.log_hits,
        args.tolerate_response_error,
        args.tolerate_key_error,
    )
    .await;

    match result {
        Ok(keys) => {
            for key in keys {
                println!("{}", key);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
