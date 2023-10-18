use unit_utils::{
    env, lazy_static,
    shared_config::{self, ConfigRedis},
};

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

#[derive(Debug)]
pub struct Config {
    pub storage_path: String,
    pub ws_port: u32,
    pub redis: ConfigRedis,
}

impl Config {
    pub fn new() -> Self {
        env::load_env();

        let storage_path = shared_config::resolve_storage_path();
        let ws_port = env::value_or_default("UNIT_WS_PORT", 6447u32);

        let Some(redis_config) = shared_config::resolve_redis() else {
            panic!("Failed to resolve redis config");
        };

        Self {
            storage_path,
            ws_port,
            redis: redis_config,
        }
    }

    #[allow(dead_code)]
    pub fn as_config(&self) -> &Config {
        return self;
    }
}
