use crate::monitor::MonitorConfig;
use actix_web::dev::Server;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use prometheus::{Encoder, TextEncoder};
use std::env;
use std::net::{IpAddr, SocketAddr};

const DEFAULT_PORT: u16 = 9000;
const DEFAULT_HOST: &str = "0.0.0.0";

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub port: u16,
    pub host: IpAddr,
}

impl AppSettings {
    pub fn new() -> Self {
        let port = env::var("PORT")
            .unwrap_or(DEFAULT_PORT.to_string())
            .parse::<u16>()
            .expect("Invalid port number");

        let host: IpAddr = env::var("HOST")
            .unwrap_or(DEFAULT_HOST.to_string())
            .parse::<IpAddr>()
            .expect("Invalid IP address");
        Self { port, host }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 9000,
            host: "0.0.0.0".parse().expect("Invalid IP address"),
        }
    }
}

pub async fn run_server(config: &MonitorConfig) -> Result<(Server, SocketAddr), std::io::Error> {
    let address = SocketAddr::new(config.app_settings.host, config.app_settings.port);

    let server = HttpServer::new(|| {
        App::new().wrap(middleware::Compress::default()).service(
            web::scope("")
                .route("/metrics", web::get().to(metrics_handler))
                .route("/health", web::get().to(HttpResponse::Ok)),
        )
    })
    .bind(address)?;

    let bound_address = server.addrs().first().copied().unwrap_or(address);

    Ok((server.run(), bound_address))
}

async fn metrics_handler() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::default_registry().gather();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => HttpResponse::Ok()
            .content_type(encoder.format_type())
            .body(buffer),
        Err(_) => HttpResponse::InternalServerError().body("Failed to encode metrics"),
    }
}
