use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use diesel::{AsChangeset, Insertable, Queryable};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::users)]
pub struct User {
    pub id: String,
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
    pub id: String,
    pub email: String,
    pub phone: Option<String>,
    pub verified: bool,
    pub role: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_wallet)]
pub struct UserWallet {
    pub id: String,
    pub user_id: String, //foreign key ref
    pub wallet_address: Option<String>,
    pub network_used_last: Option<String>,
    pub controller_info: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_wallet)]
pub struct NewUserWallet {
    pub user_id: String,
    pub wallet_address: Option<String>,
    pub network_used_last: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Queryable, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_security_logs)]
pub struct UserSecurityLog {
    pub log_id: uuid::Uuid,
    pub user_id: String,
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
    pub user_id: String,
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
    pub user_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::otp)]
pub struct NewOtp {
    pub otp_code: i32,
    pub user_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum Role {
    Admin,
    User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountVerificationResponse {
    pub account_name: String,
    pub account_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccountDetails {
    pub account_name: String,
    pub account_number: String,
    pub bank_code: String,
    pub bank_name: String,
}

#[derive(Debug, Deserialize, Serialize, Queryable, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_bank_account)]
pub struct UserBankAccount {
    pub id: uuid::Uuid,
    pub user_id: String, // foreign key ref
    pub bank_name: String,
    pub account_number: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::user_bank_account)]
pub struct NewUserBankAccount {
    pub user_id: String,
    pub bank_name: String,
    pub account_number: String,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, Clone, Insertable)]
#[diesel(table_name=crate::models::schema::user_jwt_tokens)]
pub struct NewToken {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUserBankAccountRequest {
    pub bank_name: String,
    pub account_number: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FlutterwaveBankApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bank {
    id: i64,
    pub name: String,
    pub code: String,
    #[serde(rename = "type")]
    pub bank_type: Option<String>,
}

// FOR TEST PURPOSES ONLY, NOT TO BE USED IN PRODUCTION
#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Token {
    pub token_id: uuid::Uuid,
    pub user_id: String,
    pub token: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
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
