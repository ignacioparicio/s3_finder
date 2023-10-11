# About

S3 Finder is a command-line interface (CLI) tool designed to search for S3 keys which match a specified regex and/or contain another regex.

It can be used with AWS S3 or with S3-like services.

## Features

- **Regex matching for keys**: Allows users to match S3 keys with a specified regex.
- **Regex matching for content**: Optionally match the content of the S3 key with another regex.
- **Prefix searching**: Optionally specify a prefix to narrow down the search within the S3 bucket.
- **Logging**: Provides options for logging intervals and logging each hit in stderr.
- **Early stopping**: Allows users to set a maximum number of keys to process and hits to find before stopping.
- **Flexible error handling**: Offers options to tolerate errors when fetching a response or processing a single object. These errors are logged, but the process can continue, preventing single issues from disrupting larger operations.

## Usage

Check `s3-finder --help`:

```
Usage: s3-finder [OPTIONS] --endpoint <ENDPOINT> --bucket <BUCKET>

Options:
  -e, --endpoint <ENDPOINT>
          S3 endpoint
  -b, --bucket <BUCKET>
          S3 bucket
  -k, --key-regex <KEY_REGEX>
          Regex that S3 keys must match; for example, '.*photo\.jpg$' would match all keys ending in 'photo.jpg'
  -c, --content-regex <CONTENT_REGEX>
          Regex that the content of the S3 key must match; for example, '\b2022-12-31\b' will match keys that contain the word '2022-12-31'; can be used in conjunction with --key-regex; this flag incurs a performance penalty because the entire contents of the S3 key must be loaded to memory
  -p, --prefix <PREFIX>
          Optional prefix to search in the S3 bucket
      --max-keys <MAX_KEYS>
          Optional maximum number of keys to process before stopping
      --max-hits <MAX_HITS>
          Optional maximum number of hits to find before stopping
      --log-interval <LOG_INTERVAL>
          Optional logging interval [default: 10000]
      --log-hits
          Whether to log each hit in stderr
      --tolerate-response-error
          Whether to continue execution if an error occurs processing a ListObjectsV2 response. Each ignored response can affect up to 1,000 objects
      --tolerate-key-error
          Whether to continue execution if an error occurs processing a single S3 key
      --access-key-id-env-param <ACCESS_KEY_ID_ENV_PARAM>
          Environment variable containing the S3 access key ID [default: AWS_ACCESS_KEY_ID]
      --secret-access-key-env-param <SECRET_ACCESS_KEY_ENV_PARAM>
          Environment variable containing the S3 secret access key [default: AWS_SECRET_ACCESS_KEY]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

Search on https://s3.us-west-2.amazonaws.com, bucket `billing`, for keys that have `2021-12-31` in the name:

```
s3_finder -e https://s3.us-west-2.amazonaws.com -b billing -k "2021-12-31"
```

Same example, but searching only in the prefix `costs/` and passing custom environment variables for credentials:

```
s3_finder -e https://s3.us-west-2.amazonaws.com -b billing -p "costs/" -k "2021-12-31" --access-key-id-env-param MY_ACCESS_KEY_ID --secret-access-key-env-param MY_SECRET_ACCESS_KEY
```

Search on https://s3.us-east-1.amazonaws.com, bucket `fitness` for keys that end in a video format (e.g. .mp4, .mkv), to a maximum of 50 hits:

```
s3_finder -e https://s3.us-east-1.amazonaws.com -b fitness -k ".*\.(mp4|mkv|avi|mov|flv)$" --max-hits 50
```

Search on https://s3.custom-endpoint.de bucket `real-estate` for all `.csv` or `.json` keys which content has `main street 123`, case insensitive, up to a maximum of 40,000 keys, logging each hit immediately and storing the output in `output.txt` when finished:

```
s3_finder -e https://s3.custom-endpoint.de -b real-estate -k ".*\.(csv|json)$" -c "(?i)main street 123" --max-keys 40000 --log-hits > output.txt
```

# TODO:

- try to stream outputs to output.txt instead of just at the end
- log_hits also shows number of objects scanned
- improve performance / identify inefficiencies?
