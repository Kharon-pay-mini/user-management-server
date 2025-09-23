#[derive(Debug, Clone)]
pub struct Config {
    pub port: String,
    pub _database_url: String,
    pub jwt_secret: String,
    pub ip_info_token: String,
    pub flutterwave_secret_key: String,
    pub flutterwave_payment_url: String,
    pub flutterwave_callback_url: String,
    pub flutterwave_secret_hash: String,
    pub hmac_key: String,
}

impl Config {
    pub fn init() -> Config {
        let _database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let ip_info_token = std::env::var("IP_INFO_TOKEN").expect("IP_INFO_TOKEN must be set");
        let port = std::env::var("PORT").expect("PORT must be set");
        let flutterwave_secret_key =
            std::env::var("FLUTTERWAVE_SECRET_KEY").expect("FLUTTERWAVE_SECRET_KEY must be set");
        let flutterwave_payment_url =
            std::env::var("FLUTTERWAVE_PAYMENT_URL").expect("FLUTTERWAVE_PAYMENT_URL must be set");
        let flutterwave_callback_url = std::env::var("FLUTTERWAVE_CALLBACK_URL")
            .expect("FLUTTERWAVE_CALLBACK_URL must be set");
        let flutterwave_secret_hash =
            std::env::var("FLUTTERWAVE_SECRET_HASH").expect("FLUTTERWAVE_SECRET_HASH must be set");
        let hmac_key = std::env::var("HMAC_KEY").expect("HMAC_KEY must be set");

        Config {
            _database_url,
            jwt_secret,
            ip_info_token,
            port,
            flutterwave_secret_key,
            flutterwave_payment_url,
            flutterwave_callback_url,
            flutterwave_secret_hash,
            hmac_key,
        }
    }
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}
