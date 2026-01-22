use crate::common::RouterRule;
use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitPolicy {
    pub defaults: BucketConfig,
    pub routes: Vec<RouterRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BucketConfig {
    pub max_tokens: u64,
    pub refill_per_sec: u64,
    pub penalty: u64,
    pub global_timeout: u64,
}

impl RateLimitPolicy {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::with_name("routes.yml"));
        let config = builder.build()?;
        config.try_deserialize()
    }
}
