use axum::routing::MethodFilter;
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
impl Method {
    pub fn to_method_filter(&self) -> MethodFilter {
        match self {
            Method::Get => MethodFilter::GET,
            Method::Post => MethodFilter::POST,
            Method::Put => MethodFilter::PUT,
            Method::Delete => MethodFilter::DELETE,
            Method::Patch => MethodFilter::PATCH,
            Method::Options => MethodFilter::OPTIONS,
        }
    }
}
