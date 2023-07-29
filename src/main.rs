use clap::Parser;
use s3_finder::{build_s3_client, check_env_var_exists, get_object};
/// CLI tool to search for S3 keys with a specific filename and string within the file
#[derive(Parser, Debug)]
#[command(author, version="1.0.0", about, long_about = None)]
struct Args {
    /// Name of the file to search for
    #[arg(short, long)]
    filename: String,

    /// String to search for within the files
    #[arg(short, long)]
    search_string: String,

    /// S3 endpoint
    #[arg(short, long)]
    endpoint: String,

    /// S3 bucket
    #[arg(short, long)]
    bucket: String,

    /// Optional prefix to search in the S3 bucket
    #[arg(short, long)]
    prefix: Option<String>,

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

    let client = build_s3_client(&args.endpoint).await.unwrap();
    get_object(&client, &args.bucket, "my-key").await.unwrap();

    // list_buckets(&args.endpoint, &args.bucket).await.unwrap();
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(async {
    //     list_buckets(&args.endpoint).await.unwrap();
    // });

    // list_buckets(&args.endpoint).unwrap();

    // match s3_utils::test_s3_connection(&args.bucket).await {
    //     Ok(_) => println!("Successfully connected to the bucket"),
    //     Err(e) => eprintln!("Failed to connect to the bucket: {}", e),
    // }
}
