use crate::common::Method;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RouterRule {
    pub path: String,
    pub host: String,
    pub method: Method,
    pub bucket_key: String,
    pub max_tokens: Option<u64>,
    pub refill_per_sec: Option<u64>,
    pub penalty: Option<u64>,
}
