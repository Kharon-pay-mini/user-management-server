use crate::database::{
    otp_db::OtpImpl, user_bank_account_db::UserBankImpl, user_db::UserImpl,
    user_security_log_db::UserSecurityLogsImpl, user_wallet_db::UserWalletImpl,
};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenv::dotenv;
use r2d2::{Error as PoolError, Pool, PooledConnection};

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug)]
pub enum AppError {
    DbConnectionError(PoolError),
    DieselError(diesel::result::Error),
}

#[derive(Debug)]
pub enum DatabaseSetupError {
    DbConnectionError(PoolError),
    DieselError(diesel::result::Error),
    DatabaseUrlNotSet,
    ErrorRunningMigrations,
}

#[derive(Clone)]
pub struct Database {
    pub pool: DBPool,
}

impl Database {
    pub fn new() -> Result<Self, DatabaseSetupError> {
        dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| DatabaseSetupError::DatabaseUrlNotSet)?;

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .map_err(DatabaseSetupError::DbConnectionError)?;

        // Only run migrations in development
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "prod".into());
        if env == "dev" {
            run_migrations(&pool)?;
        }

        Ok(Database { pool })
    }
}

fn run_migrations(pool: &Pool<ConnectionManager<PgConnection>>) -> Result<(), DatabaseSetupError> {
    println!("RUNNING MIGRATIONS....");
    let mut conn = pool.get().map_err(DatabaseSetupError::DbConnectionError)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|_| DatabaseSetupError::ErrorRunningMigrations)?;
    println!("MIGRATIONS COMPLETED....");
    Ok(())
}

pub trait DbAccess {
    fn conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, PoolError>;
}

impl DbAccess for Database {
    fn conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, PoolError> {
        self.pool.get()
    }
}

impl UserWalletImpl for Database {}
impl UserImpl for Database {}
impl OtpImpl for Database {}
impl UserSecurityLogsImpl for Database {}
impl UserBankImpl for Database {}
