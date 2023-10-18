pub mod env;
pub mod err;
pub mod shared_config;
pub mod shell;

pub use anyhow::Result;
pub use lazy_static::lazy_static;

pub fn gen_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
