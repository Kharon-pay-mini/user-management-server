use crate::{
    AppState,
    database::token_db::TokenImpl,
    helpers::bank_helpers::get_bank_code_and_verify_account,
    models::{
        models::{
            BankAccountDetails, GetBankAccountQuery, NewUserBankAccount,
            NewUserBankAccountRequest, UserBankAccount,
        },
        response::FilteredBankDetails,
    },
};
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder,
    get, post, web,
};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::query;
use std::usize;

use crate::database::{db::AppError, user_bank_account_db::UserBankImpl, user_db::UserImpl};
use crate::models::models::{CreateUserSchema, NewUser, User, UserWallet};

use crate::models::response::FilteredUser;

fn filtered_bank_record(bank: &UserBankAccount) -> FilteredBankDetails {
    FilteredBankDetails {
        bank_details_id: bank.id.to_string(),
        user_id: bank.user_id.to_string(),
        bank_name: bank.bank_name.to_string(),
        bank_account_number: bank.account_number.to_string(),
        account_name: bank.account_name.clone(),
    }
}

fn filtered_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        phone: user.phone.clone(),
        last_logged_in: user.last_logged_in.unwrap(),
        verified: user.verified,
        role: user.role.clone(),
        created_at: user.created_at.unwrap(),
    }
}

#[post("/auth/create")]
async fn create_user_handler(
    req: HttpRequest,
    body: web::Json<CreateUserSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let expected_api_key = &data.env.hmac_key;

    match req.headers().get("x-api-key") {
        Some(provided_key) => {
            if *provided_key != *expected_api_key {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid API key"
                }));
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "API key missing"
            }));
        }
    }

    let phone: String = body.phone.clone();

    // Check if user with phone already exists
    match data.db.get_user_by_phone(phone.clone().as_str()) {
        Ok(existing_user) => {
            return HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "User already exists",
                "data": filtered_user_record(&existing_user)
            }));
        }
        Err(AppError::DbConnectionError(e)) => {
            eprintln!("DB connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database connection failed"
            }));
        }
        Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
            // User not found, continue to create new user
        }
        Err(AppError::DieselError(e)) => {
            eprintln!("Query error: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Database query failed"
            }));
        }
    }

    // Create new user with phone number
    let new_user = NewUser {
        id: uuid::Uuid::new_v4().simple().to_string(),
        phone: phone.clone(),
        verified: false,
        role: String::from("user"),
    };

    match data.db.create_user(new_user.clone()) {
        Ok(user) => HttpResponse::Created().json(json!({
            "status": "success",
            "message": "User created successfully",
            "data": filtered_user_record(&user)
        })),
        Err(e) => {
            eprintln!("Failed to create user: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to create user"
            }))
        }
    }
}

#[post("/users/me/bank-accounts/verify")]
async fn verify_user_bank_account_handler(
    req: HttpRequest,
    query: web::Query<NewUserBankAccountRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let expected_api_key = &data.env.hmac_key;

    match req.headers().get("x-api-key") {
        Some(provided_key) => {
            if *provided_key != *expected_api_key {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid API key"
                }));
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "API key missing"
            }));
        }
    }

    let account_number = query.account_number.clone();
    let bank_name = query.bank_name.clone();
    let user_phone = query.phone.clone();

    match data.db.get_user_by_phone(user_phone.as_str()) {
        Ok(_) => {
            let (account_details, bank_code) = match get_bank_code_and_verify_account(
                &data,
                bank_name.clone(),
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

            let verification_response = BankAccountDetails {
                phone: user_phone.clone(),
                account_name: account_details.account_name,
                account_number: account_details.account_number,
                bank_name: bank_name.clone(),
                bank_code: bank_code.clone(),
            };

            HttpResponse::Ok().json(json!({
                "status": "success",
                "data": verification_response
            }))
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
                    eprintln!("Failed to verify bank account: {:?}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": format!("Failed to verify bank account: {:?}", e)
                    }));
                }
            };
        }
    }
}

#[post("/users/me/bank-accounts/confirm")]
async fn confirm_user_bank_account_handler(
    req: HttpRequest,
    body: web::Json<BankAccountDetails>,
    data: web::Data<AppState>,
) -> impl Responder {
    let expected_api_key = &data.env.hmac_key;

    match req.headers().get("x-api-key") {
        Some(provided_key) => {
            if *provided_key != *expected_api_key {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid API key"
                }));
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "API key missing"
            }));
        }
    }

    let user_phone = body.phone.clone();
    let account_number = body.account_number.clone();
    let bank_acc_name = body.bank_name.clone();

    match data.db.get_user_by_phone(&user_phone.as_str()) {
        Ok(user) => {
            let bank_details = NewUserBankAccount {
                user_id: user.id.clone(),
                account_number: account_number.clone(),
                bank_name: bank_acc_name.clone(),
                account_name: Some(body.account_name.clone()),
                phone: Some(user_phone.clone()),
            };
            println!("Confirming bank details: {:?}", bank_details);

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
    query: web::Query<GetBankAccountQuery>,
) -> impl Responder {
    let expected_api_key = &data.env.hmac_key;
    match req.headers().get("x-api-key") {
        Some(provided_key) => {
            if *provided_key != *expected_api_key {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": "Invalid API key"
                }));
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "error",
                "message": "API key missing"
            }));
        }
    }

    let user_phone = query.phone.clone();

    match data.db.get_banks_by_user_phone(&user_phone) {
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
