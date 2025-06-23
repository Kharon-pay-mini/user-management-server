use crate::{
    AppState,
    auth::otp::generate_otp,
    helpers::bank_helpers::get_bank_code_and_verify_account,
    models::{
        models::{NewUserBankAccount, NewUserBankAccountRequest, UserBankAccount},
        response::FilteredBankDetails,
        schema::user_bank_account::bank_name,
    },
};
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder,
    cookie::{Cookie, time::Duration as ActixWebDuration},
    get, post, web,
};
use awc::cookie::{SameSite, time::OffsetDateTime};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::json;
use std::usize;

use crate::auth::{
    auth::{log_failed_login, log_successful_login, logout},
    jwt_auth::JwtMiddleware,
};
use crate::database::{
    db::AppError, otp_db::OtpImpl, user_bank_account_db::UserBankImpl, user_db::UserImpl,
    user_security_log_db::UserSecurityLogsImpl, user_wallet_db::UserWalletImpl,
};
use crate::models::models::{
    CreateUserSchema, NewUser, NewUserWallet, OtpSchema, TokenClaims, User, UserSecurityLog,
    UserWallet, UserWalletSchema, ValidateOtpSchema,
};

use crate::models::response::{FilteredUser, FilteredUserSecurityLogs, FilteredWallet};

fn filtered_security_logs(security_log: &UserSecurityLog) -> FilteredUserSecurityLogs {
    FilteredUserSecurityLogs {
        log_id: security_log.log_id.to_string(),
        user_id: security_log.user_id.to_string(),
        ip_address: security_log.ip_address.to_string(),
        city: security_log.city.to_string(),
        country: security_log.country.to_string(),
        failed_login_attempts: security_log.failed_login_attempts,
        flagged_for_review: security_log.flagged_for_review,
        created_at: security_log.created_at.unwrap_or_else(|| Utc::now()),
    }
}

// fn filtered_otp(otp: &Otp) -> FilteredOtp {
//     FilteredOtp {
//         otp_id: otp.otp_id.to_string(),
//         user_id: otp.user_id.to_string(),
//         otp: otp.otp_code,
//         created_at: otp.created_at,
//         expires_at: otp.expires_at,
//     }
// }

fn filtered_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_string(),
        phone: user.phone.clone(),
        last_logged_in: user.last_logged_in.unwrap_or_else(|| Utc::now()),
        verified: user.verified,
        role: user.role.clone(),
        created_at: user.created_at.unwrap(),
    }
}

fn filtered_wallet_record(wallet: &UserWallet) -> FilteredWallet {
    FilteredWallet {
        user_id: wallet.user_id.to_string(),
        wallet_address: wallet
            .wallet_address
            .as_ref()
            .map_or("Unknown".to_string(), |s| s.to_string()),
        network_used_last: wallet
            .network_used_last
            .as_ref()
            .map_or("Unknown".to_string(), |s| s.to_string()),
        created_at: wallet.created_at,
        updated_at: wallet.updated_at.unwrap_or_else(|| Utc::now()),
    }
}

fn filtered_bank_record(bank: &UserBankAccount) -> FilteredBankDetails {
    FilteredBankDetails {
        bank_details_id: bank.id.to_string(),
        user_id: bank.user_id.to_string(),
        bank_name: bank.bank_name.to_string(),
        bank_account_number: bank.account_number.to_string(),
    }
}

#[post("/auth/create")]
async fn create_user_handler(
    body: web::Json<CreateUserSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let email = body.email.to_string().to_lowercase();
    let phone: Option<String> = body.phone.as_ref().map(|s| s.to_string());

    // Early return pattern - check email first
    match data.db.get_user_by_email(email.clone()) {
        Ok(existing_user) => {
            return generate_otp(existing_user.id, existing_user.email, data.db.clone()).await;
        }
        Err(AppError::DbConnectionError(e)) => {
            eprintln!("DB connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("{:?}", e)
            }));
        }
        Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
            // Continue to phone check
            eprintln!("Email {} not found for user, checking phone.", email);
        }
        Err(AppError::DieselError(e)) => {
            eprintln!("Query error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("{:?}", e)
            }));
        }
    }

    // Check phone if provided
    if let Some(phone_number) = phone.clone() {
        match data.db.get_user_by_phone(phone_number) {
            Ok(existing_user) => {
                return HttpResponse::Ok().json(json!({
                    "status": "success",
                    "data": filtered_user_record(&existing_user)
                }));
            }
            Err(AppError::DbConnectionError(e)) => {
                eprintln!("DB connection error: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                }));
            }
            Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
                // Continue to user creation
                eprintln!("Phone not found for user, creating user...");
            }
            Err(AppError::DieselError(e)) => {
                eprintln!("Query error: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                }));
            }
        }
    }

    // Neither email nor phone found - create new user
    let new_user = NewUser {
        // Generate a new UUID for user with a simple format to parse felt correctly
        // This is a workaround to ensure UUIDs are compatible with the Felt format in cartridge controller calls
        id: uuid::Uuid::new_v4().simple().to_string(),
        email: email.clone(),
        phone: phone.clone(),
        verified: false,
        role: String::from("user"),
    };

    match data.db.create_user(new_user.clone()) {
        Ok(user) => {
            println!("User created successfully: {:?}", user.id);
            return generate_otp(user.id, user.email, data.db.clone()).await;
        }
        Err(e) => {
            eprintln!("Failed to create user: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Failed to create user: {:?}", e)
            }))
        }
    }
}

#[post("/users/me/bank-accounts")]
async fn register_user_bank_account_handler(
    body: web::Json<NewUserBankAccountRequest>,
    data: web::Data<AppState>,
    auth: JwtMiddleware,
) -> impl Responder {
    let user_id = auth.user_id;
    let account_number = body.account_number.clone();
    let bank_acc_name = body.bank_name.clone();

    match data.db.get_user_by_id(user_id.as_str().clone()) {
        Ok(_) => {
            let (_account_details, _bank_code) = match get_bank_code_and_verify_account(
                &data,
                bank_acc_name.clone(),
                account_number.clone(),
            )
            .await
            {
                Ok(details) => details,
                Err(e) => {
                    eprintln!("Failed to verify bank account: {:?}", e);
                    return e;
                }
            };

            let bank_details = NewUserBankAccount {
                user_id: user_id.clone(),
                account_number: account_number.clone(),
                bank_name: bank_acc_name.clone(),
            };
            println!("Bank details: {:?}", bank_details);

            match data.db.create_user_bank(bank_details) {
                Ok(bank) => {
                    let filtered_bank_details = filtered_bank_record(&bank);
                    HttpResponse::Created().json(filtered_bank_details)
                }
                Err(e) => {
                    eprintln!("Failed to create bank details: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": format!("Failed to create bank details: {:?}", e)
                    }));
                }
            }
        }
        Err(e) => {
            match e {
                AppError::DieselError(diesel::result::Error::NotFound) => {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "User not found"
                    }));
                }
                _ => {
                    eprintln!("Failed to create wallet: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": format!("Failed to create bank details: {:?}", e)
                    }));
                }
            };
        }
    }
}

#[get("/users/me/bank-accounts")]
async fn get_user_bank_accounts_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> impl Responder {
    let ext = req.extensions();
    let user_id = ext.get::<String>().unwrap();

    match data.db.get_banks_by_user_id(*&user_id) {
        Ok(banks) => {
            let filtered_banks: Vec<FilteredBankDetails> = banks
                .into_iter()
                .map(|bank| filtered_bank_record(&bank))
                .collect();

            let json_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "banks": filtered_banks
                })
            });

            HttpResponse::Ok().json(json_response)
        }
        Err(e) => {
            eprintln!("Error fetching user bank accounts: {:?}", e);
            HttpResponse::InternalServerError().json("Error fetching user bank accounts")
        }
    }
}

#[post("/users/me/wallet")]
async fn update_user_wallet_handler(
    body: web::Json<UserWalletSchema>,
    data: web::Data<AppState>,
    auth: JwtMiddleware,
) -> impl Responder {
    let user_id = auth.user_id;
    let wallet_address = body.wallet_address.to_string();
    let network = body.network.to_string();

    match data.db.get_user_by_id(user_id.as_str().clone()) {
        Ok(_) => {
            let wallet = NewUserWallet {
                user_id: user_id.clone(),
                wallet_address: Some(wallet_address.clone()),
                network_used_last: Some(network.clone()),
            };

            match data.db.create_user_wallet(wallet) {
                Ok(wallet) => {
                    let filtered_wallet = filtered_wallet_record(&wallet);
                    HttpResponse::Created().json(filtered_wallet)
                }
                Err(e) => {
                    eprintln!("Failed to create wallet: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": format!("Failed to create user: {:?}", e)
                    }));
                }
            }
        }
        Err(e) => {
            match e {
                AppError::DieselError(diesel::result::Error::NotFound) => {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "User not found"
                    }));
                }
                _ => {
                    eprintln!("Failed to create wallet: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": format!("Failed to create wallet: {:?}", e)
                    }));
                }
            };
        }
    }
}

#[post("/users/resend-otp")]
async fn resend_otp_handler(
    body: web::Json<OtpSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let email = body.email.clone();
    let user = match data.db.get_user_by_email(email) {
        Ok(user) => user,
        Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "User with request id not found"
            }));
        }
        Err(e) => {
            eprintln!("Failed to retrieve user email: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to process request"
            }));
        }
    };

    return generate_otp(user.id, user.email, data.db.clone()).await;
}

#[post("/users/validate-otp")]
async fn validate_otp_handler(
    body: web::Json<ValidateOtpSchema>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let user_email = body.email.clone();
    let otp = body.otp;

    let user = match data.db.get_user_by_email(user_email.clone()) {
        Ok(user) => user,
        Err(e) => match e {
            AppError::DieselError(diesel::result::Error::NotFound) => {
                return HttpResponse::Unauthorized().json("User not found");
            }
            _ => {
                eprintln!("Database error getting user: {:?}", e);
                return HttpResponse::InternalServerError().json("Database error.");
            }
        },
    };

    let user_id = user.id;
    let stored_otp = data.db.get_otp_by_user_id(user_id.clone());

    match stored_otp {
        Ok(otp_record) => {
            if otp_record.expires_at < Utc::now() {
                if let Err(e) = data.db.delete_expired_otps() {
                    eprint!("Failed to clean expired OTPs: {:?}", e);
                }
                return HttpResponse::Unauthorized().json("OTP has expired.");
            }

            if otp_record.otp_code != otp {
                log_failed_login(&data, &req, user_id, Some("Invalid otp".to_string())).await;
                return HttpResponse::Unauthorized().json("Invalid otp");
            }

            if let Err(e) = data.db.delete_otp_by_id(otp_record.otp_id) {
                eprint!("Failed to delete used OTP: {:?}", e);
                return HttpResponse::InternalServerError().json("Failed to clean up OTP");
            }

            let now = Utc::now();
            let iat = now.timestamp() as usize;
            let exp = (now + Duration::seconds(24 * 60 * 60)).timestamp() as usize;
            let claims: TokenClaims = TokenClaims {
                sub: user_id.to_string().clone(),
                iat,
                exp,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
            )
            .unwrap();

            let cookie = Cookie::build("token", token.to_owned())
                .path("/")
                .secure(true)
                .max_age(ActixWebDuration::new(24 * 60 * 60, 0)) //24h
                .http_only(true)
                .same_site(SameSite::None) //TODO SameSite::Strict in production
                .finish();

            log_successful_login(&data, &req, user_id).await;

            HttpResponse::Ok()
                .cookie(cookie)
                .json(json!({"status": "success", "message": "Sign in successful"}))
        }
        Err(e) => {
            match e {
                AppError::DieselError(diesel::result::Error::NotFound) => {
                    return HttpResponse::Unauthorized().json("No OTP found");
                }
                _ => {
                    eprint!("Database error: {:?}", e);
                    return HttpResponse::InternalServerError().json("Database error.");
                }
            };
        }
    }
}

#[get("/users/me")]
async fn get_user_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> impl Responder {
    let ext = req.extensions();
    let user_id = ext.get::<String>().unwrap();
    let user = match data.db.get_user_by_id(*&user_id.as_str().clone()) {
        Ok(user) => user,
        Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
            return HttpResponse::NotFound().json("User not found");
        }
        Err(e) => {
            eprint!("Error fetching user: {:?}", e);
            return HttpResponse::InternalServerError().json("Error fetching user");
        }
    };

    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "user": filtered_user_record(&user)
        })
    });

    HttpResponse::Ok().json(json_response)
}

#[get("/users/me/logs")]
async fn get_user_logs_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> impl Responder {
    let ext = req.extensions();
    let user_id = ext.get::<String>().unwrap();

    let logs = match data.db.get_security_logs_by_user_id(*&user_id.as_str().clone()) {
        Ok(log) => log,
        Err(e) => {
            eprint!("Error fetching user logs: {:?}", e);
            return HttpResponse::InternalServerError().json("Error fetching user logs");
        }
    };

    let filtered_logs: Vec<FilteredUserSecurityLogs> = logs
        .into_iter()
        .map(|log| filtered_security_logs(&log))
        .collect();

    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "user_logs": filtered_logs
        })
    });

    HttpResponse::Ok().json(json_response)
}

#[get("/users/me/wallet")]
async fn get_wallet_handler(
    req: HttpRequest,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> impl Responder {
    let ext = req.extensions();
    let user_id = ext.get::<String>().unwrap();

    let wallet = match data.db.get_wallet_by_user_id(*&user_id.as_str().clone()) {
        Ok(wallet) => wallet,
        Err(e) => {
            match e {
                AppError::DieselError(diesel::result::Error::NotFound) => {
                    return HttpResponse::NotFound().json("Wallet not found");
                }
                _ => {
                    eprint!("Database error: {:?}", e);
                    return HttpResponse::InternalServerError().json("Error fetching user wallet");
                }
            };
        }
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "wallet": filtered_wallet_record(&wallet)
        })
    });

    HttpResponse::Ok().json(json_response)
}

#[post("/users/logout")]
async fn logout_handler(
    data: web::Data<AppState>,
    req: HttpRequest,
    auth: JwtMiddleware,
) -> impl Responder {
    let user_id = auth.user_id;

    logout(&data, &req, user_id.as_str().clone()).await;

    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(0, 0))
        .http_only(true)
        .expires(OffsetDateTime::now_utc())
        .finish();

    HttpResponse::Ok().cookie(cookie).json(json!({
        "status": "success",
        "message": "Logged out successfully"
    }))
}
