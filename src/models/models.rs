use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use diesel::{AsChangeset, Insertable, Queryable};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::users)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub last_logged_in: Option<DateTime<Utc>>,
    pub verified: bool,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::users)]
pub struct NewUser {
    pub email: String,
    pub phone: Option<String>,
    pub verified: bool,
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_wallet)]
pub struct UserWallet {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid, //foreign key ref
    pub wallet_address: Option<String>,
    pub network_used_last: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_wallet)]
pub struct NewUserWallet {
    pub user_id: uuid::Uuid,
    pub wallet_address: Option<String>,
    pub network_used_last: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_security_logs)]
pub struct UserSecurityLog {
    pub log_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub ip_address: String,
    pub city: String,
    pub country: String,
    pub failed_login_attempts: i32,
    pub flagged_for_review: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_security_logs)]
pub struct NewUserSecurityLog {
    pub user_id: uuid::Uuid,
    pub ip_address: String,
    pub city: String,
    pub country: String,
    pub failed_login_attempts: i32,
    pub flagged_for_review: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::otp)]
pub struct Otp {
    pub otp_id: uuid::Uuid,
    pub otp_code: i32,
    pub user_id: uuid::Uuid,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::otp)]
pub struct NewOtp {
    pub otp_code: i32,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum Role {
    Admin,
    User,
}

/*      JSONWEBTOKEN TOKEN DECODE PARAMS     */
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

/*  MODEL SCHEMAS */
#[derive(Debug, Deserialize)]
pub struct CreateUserSchema {
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserWalletSchema {
    pub wallet_address: String,
    pub network: String,
}

// #[derive(Debug, Deserialize)]
// pub struct UserSecurityLogsSchema {
//     pub user_id: uuid::Uuid,
//     pub ip_address: String,
//     pub city: String,
//     pub country: String,
//     pub failed_login_attempts: i64,
//     pub flagged_for_review: bool,
// }

#[derive(Debug, Deserialize)]
pub struct OtpSchema {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ValidateOtpSchema {
    pub email: String,
    pub otp: i32,
}

// /*  DISPLAY IMPLEMENTATION FOR ENUMS */
macro_rules! impl_display {
    ($($t:ty), *) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        )*
    };
}
impl_display!(Role);
