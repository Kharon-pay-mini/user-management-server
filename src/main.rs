mod config;
mod database;
mod routes;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, http::header, middleware::Logger, web};
use config::{config::Config, config_scope};
use database::db::Database;
use dotenv::dotenv;

pub struct AppState {
    db: Database,
    env: Config,
    // pub redis_pool: RedisPool,
    // pub geo_locator: GeoLocator,
    // pub price_feed: Arc<Mutex<PriceData>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting Server......");
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "actix_web=info");
        }
    }

    let config = Config::init();

    let db = match database::db::Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize DB: {:?}", e);
            std::process::exit(1);
        }
    };

    let port: u16 = config.port.parse().expect("PORT must be i16 type");

    let app_state = web::Data::new(AppState {
        db: db.clone(),
        env: config.clone(),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
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
        // .wrap(from_fn(security_logger_middleware))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
