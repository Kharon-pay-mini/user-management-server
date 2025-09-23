mod auth;
mod config;
mod database;
mod helpers;
mod middleware;
mod models;
mod routes;
mod services;

use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    http::header,
    middleware::{Logger, from_fn},
    web,
};
use config::{config::Config, config_scope};
use database::db::Database;
use dotenv::dotenv;
use services::geolocation::geolocator::GeoLocator;

use crate::middleware::security_log::security_logger_middleware;

pub struct AppState {
    db: Database,
    env: Config,
    // pub redis_pool: RedisPool,
    pub geo_locator: GeoLocator,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "actix_web=trace");
        }
    }

    env_logger::init();
    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());

    log::info!("Starting Server......");

    let config = Config::init();

    let db = match database::db::Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize DB: {:?}", e);
            std::process::exit(1);
        }
    };
    let geo_locator = GeoLocator::new(config.ip_info_token.clone());
    let port: u16 = config.port.parse().expect("PORT must be i16 type");

    let app_state = web::Data::new(AppState {
        db: db.clone(),
        env: config.clone(),
        geo_locator: geo_locator.clone(),
    });

    log::info!("Server is running on port: {}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .configure(config_scope::config)
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(from_fn(security_logger_middleware))
    })
    .bind((bind_address, port))?
    .run()
    .await
}
