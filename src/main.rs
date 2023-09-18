use clap::Parser;
use s3_finder::{build_s3_client, check_env_var_exists, find_s3_keys, test_access_to_bucket};
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
    #[arg(long)]
    log_interval: Option<usize>,

    /// Whether to log each hit in stderr
    #[arg(long, default_value_t = false)]
    log_hits: bool,

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

    check_env_var_exists(&args.access_key_id_env_param);
    check_env_var_exists(&args.secret_access_key_env_param);

    let client = build_s3_client(&args.endpoint).await;

    test_access_to_bucket(&client, &args.endpoint, &args.bucket).await;

    let result = find_s3_keys(
        &client,
        &args.bucket,
        args.prefix.as_deref(),
        args.key_regex.as_deref(),
        args.content_regex.as_deref(),
        args.max_keys,
        args.max_hits,
        args.log_interval.unwrap_or(100000),
        args.log_hits,
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
