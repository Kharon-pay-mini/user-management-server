#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub _database_url: String,
    pub jwt_secret: String,
    pub ip_info_token: String,
}

impl Config {
    pub fn init() -> Config {
        let _database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let ip_info_token = std::env::var("IP_INFO_TOKEN").expect("IP_INFO_TOKEN must be set");
        let port = std::env::var("PORT").expect("PORT must be set");

        Config {
            _database_url,
            jwt_secret,
            ip_info_token,
            port,
        }
    }
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}
