use crate::env;
use std::{fs, path};

#[derive(Clone, Debug)]
pub struct ConfigRedis {
    pub host: String,
    pub port: u32,
    pub auth: Option<String>,
    pub db: u32,
}

impl ConfigRedis {
    pub fn to_url(&self) -> String {
        if self.auth.is_some() {
            format!(
                "redis://{}@{}:{}/{}",
                self.auth.clone().unwrap(),
                self.host,
                self.port,
                self.db
            )
        } else {
            format!("redis://{}:{}/{}", self.host, self.port, self.db)
        }
    }
}

pub fn resolve_storage_path() -> String {
    let storage_path = env::str_or_default("UNIT_STORAGE_PATH", "./data");
    let storage_path = path::Path::new(&storage_path);

    if !storage_path.exists() {
        if !fs::create_dir_all(storage_path).is_ok() {
            panic!("Failed to create storage path: {:?}", storage_path);
        }
    }

    if !storage_path.is_dir() {
        panic!("Storage path is not a directory: {:?}", storage_path);
    }

    let Ok(storage_path) = fs::canonicalize(storage_path) else {
        panic!("Failed to canonicalize storage path: {:?}", storage_path);
    };

    return storage_path.to_str().unwrap().to_owned();
}

pub fn resolve_redis() -> Option<ConfigRedis> {
    match env::optional_str("UNIT_REDIS_HOST") {
        Some(host) => {
            let port = env::value_or_default("UNIT_REDIS_PORT", 6379u32);
            let auth = env::optional_str("UNIT_REDIS_AUTH");
            let db = env::value_or_default("UNIT_REDIS_DB", 3u32);

            Some(ConfigRedis {
                host,
                port,
                auth,
                db,
            })
        }
        None => None,
    }
}
