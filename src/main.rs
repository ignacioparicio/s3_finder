use clap::Parser;
mod fs_utils;
use fs_utils::validate_env;

/// CLI tool to search for S3 keys with a specific filename and string within the file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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

fn main() {
    let args = Args::parse();

    validate_env(&args.access_key_id_env_param, "Access Key ID");
    validate_env(&args.secret_access_key_env_param, "Secret Access Key");
}
