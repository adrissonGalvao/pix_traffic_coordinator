use crate::common::RouterRule;
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
}

pub fn load_routes(routes: &Vec<RouterRule>) -> Router {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(10)
        .build()
        .expect("Http client error");

    let state = AppState { client };

    let mut router =
        Router::new().route("/health", get(|| async { Json(json!({ "status": "ok" })) }));

    for route in routes {
        let host_url = route.host.clone();
        let method = route.method.to_method_filter();
        let handler = {
            let host_url = host_url.clone();
            move |state: State<AppState>, req: Request| handler(state, req, host_url)
        };

        router = router.route(&route.path, on(method, handler));
    }

    router.with_state(state)
}

use axum::extract::Request;
use axum::routing::on;

async fn handler(
    State(state): State<AppState>,
    req: Request,
    host_url: String,
) -> impl IntoResponse {
    let path = req.uri().path();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_url = format!("{}{}{}", host_url, path, query);

    let method = req.method().clone();
    let body = req.into_body();

    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading body: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let result = state
        .client
        .request(method, &target_url)
        .body(bytes)
        .send()
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            let stream = response.bytes_stream();
            let body_resp = Body::from_stream(stream);

            Response::builder()
                .status(status)
                .body(body_resp)
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            eprintln!("Error while making request: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "Bad Gateway" })),
            )
                .into_response()
        }
    }
}
