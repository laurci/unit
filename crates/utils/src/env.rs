use std::str::FromStr;

pub fn load_env() {
    dotenv::dotenv().ok();
}

pub fn required_value<T: FromStr>(key: &str) -> T {
    match std::env::var(key) {
        Ok(value) => match value.parse::<T>() {
            Ok(value) => value,
            Err(_) => panic!("{} is not a valid value for {}", value, key),
        },
        Err(_) => panic!("{} is not set", key),
    }
}

pub fn value_or_default<T: FromStr>(key: &str, default: T) -> T {
    match std::env::var(key) {
        Ok(value) => match value.parse::<T>() {
            Ok(value) => value,
            Err(_) => panic!("{} is not a valid value for {}", value, key),
        },
        Err(_) => default,
    }
}

pub fn optional_value<T: FromStr>(key: &str) -> Option<T> {
    match std::env::var(key) {
        Ok(value) => match value.parse::<T>() {
            Ok(value) => Some(value),
            Err(_) => panic!("{} is not a valid value for {}", value, key),
        },
        Err(_) => None,
    }
}

pub fn required_str(key: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => panic!("{} is not set", key),
    }
}

pub fn optional_str(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn str_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_string(),
    }
}
