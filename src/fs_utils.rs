use std::env;

pub fn validate_env(env_var: &str, display_name: &str) {
    if env::var(env_var).is_err() {
        panic!("{} is not set", display_name);
    }
}
