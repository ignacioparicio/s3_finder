use std::env;

pub fn check_env_var_exists(env_var: &str) {
    if env::var(env_var).is_err() {
        panic!("Missing environment variable {env_var}.");
    }
}
