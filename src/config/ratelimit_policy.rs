use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    #[serde(alias = "GET", alias = "get")]
    Get,
    #[serde(alias = "POST", alias = "post")]
    Post,
    #[serde(alias = "PUT", alias = "put")]
    Put,
    #[serde(alias = "DELETE", alias = "delete")]
    Delete,
    #[serde(alias = "PATCH", alias = "patch")]
    Patch,
    #[serde(alias = "OPTIONS", alias = "options")]
    Options,
}

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
}
#[derive(Debug, Deserialize, Clone)]
pub struct RouterRule {
    pub path: String,
    pub method: Method,
    pub bucket_key: String,
    pub max_tokens: Option<u64>,
    pub refill_per_sec: Option<u64>,
    pub penalty: Option<u64>,
}

impl RateLimitPolicy {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::with_name("routes.yml"));
        let config = builder.build()?;
        config.try_deserialize()
    }
}
