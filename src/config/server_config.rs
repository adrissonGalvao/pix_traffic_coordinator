use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub http_config: HttpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub port: u16,
}

impl ServerConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::with_name("server.yml"))
            .add_source(Environment::with_prefix("PIX").separator("__"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}
