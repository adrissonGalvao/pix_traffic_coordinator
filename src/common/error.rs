use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub enum AppError {
    BadGateway(String),
    InternalServerError(anyhow::Error),
}

// Transformar o erro numa resposta HTTP
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::InternalServerError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro interno: {}", err),
            ),
        };

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
