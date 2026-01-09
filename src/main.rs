mod config;

use crate::config::RateLimitPolicy;

fn main() {
    println!(">>> Inicializando Gateway Pix...");

    let policy = match RateLimitPolicy::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprint!("Error when load rate limit policy. Error: {}", err);
            std::process::exit(1);
        }
    };

    println!(">>> test read: {:?}", policy.defaults);
    println!(">>> test routes: {}", policy.routes.len());
}
