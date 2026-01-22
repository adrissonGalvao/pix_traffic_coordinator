mod common;
mod config;
mod routes;

use std::net::SocketAddr;

use config::{RateLimitPolicy, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_config = match ServerConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprint!("Error when load server config. Error: {}", err);
            std::process::exit(1);
        }
    };

    let policy = match RateLimitPolicy::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprint!("Error when load rate limit policy. Error: {}", err);
            std::process::exit(1);
        }
    };

    let app = routes::load_routes(&policy.routes);

    // 3. Sobe o servidor (Sintaxe do Axum 0.7+)
    let addr = SocketAddr::from(([127, 0, 0, 1], server_config.http_config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
