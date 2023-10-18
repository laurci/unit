use unit_utils::{
    env, lazy_static,
    shared_config::{self, resolve_storage_path, ConfigRedis},
};

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

#[derive(Debug)]
pub struct Config {
    pub grpc_port: u32,
    pub grpc_api_key: String,
    pub storage_location: String,
    pub redis: ConfigRedis,
}

impl Config {
    pub fn new() -> Self {
        env::load_env();

        let grpc_port = env::value_or_default("UNIT_GRPC_PORT", 6448u32);
        let grpc_api_key: String = env::required_value("UNIT_GRPC_API_KEY");

        let storage_location = resolve_storage_path();

        let Some(redis_config) = shared_config::resolve_redis() else {
            panic!("Failed to resolve redis config");
        };

        Self {
            grpc_port,
            grpc_api_key,
            storage_location,
            redis: redis_config,
        }
    }

    #[allow(dead_code)]
    pub fn as_config(&self) -> &Config {
        return self;
    }
}
