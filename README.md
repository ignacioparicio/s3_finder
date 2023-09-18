# References

- Sample S3 code https://github.com/awslabs/aws-sdk-rust/tree/main/examples/s3
- Docs https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/

# TODO:

- try to stream outputs to output.txt instead of just at the end
- Replace expect() with proper error handling
- Checks:
  - warning if neither key_regex nor content_regex is passed
  - warning if content_regex is passed but key_regex isn't
