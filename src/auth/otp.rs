use crate::{
    database::{db::AppError, otp_db::OtpImpl},
    models::models::NewOtp,
    services::email_service::send_verification_email,
};
use actix_web::HttpResponse;
use chrono::{Duration, Utc};
use rand::{Rng, rng};
use serde_json::json;
use uuid::Uuid;

use crate::database::db::Database;

pub async fn generate_otp(user_id_no: Uuid, user_email: String, db: Database) -> HttpResponse {
    match db.get_otp_by_user_id(user_id_no) {
        Ok(existing_otp) => {
            let now = Utc::now();
            let time_since_creation = now - existing_otp.created_at;

            if time_since_creation < Duration::minutes(2) {
                let remaining_seconds = (Duration::minutes(2) - time_since_creation).num_seconds();
                return HttpResponse::TooManyRequests().json(json!({
                    "status": "error",
                    "message": "OTP already sent. Please wait before requesting another.",
                    "retry_after_seconds": remaining_seconds,
                    "retry_after_minutes": (remaining_seconds as f64 / 60.0).ceil() as i64
                }));
            }

            if let Err(e) = db.delete_otp_by_id(existing_otp.otp_id) {
                eprintln!("Failed to delete old OTP: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to process OTP request"
                }));
            } else {
                println!("Deleted old OTP for user {}", user_id_no);
                return send_new_otp(user_id_no, user_email, db).await;
            }
        }
        Err(AppError::DieselError(diesel::result::Error::NotFound)) => {
            println!(
                "No existing OTP found for user {}, creating new one",
                user_id_no
            );
            return send_new_otp(user_id_no, user_email, db).await;
        }
        Err(e) => {
            eprintln!("Database error checking existing OTP: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to process OTP request"
            }));
        }
    }
}

pub async fn send_new_otp(user_id: Uuid, user_email: String, db: Database) -> HttpResponse {
    let otp_code = rng().random_range(100_000..=999_999);

    let new_otp = NewOtp {
        otp_code: otp_code.clone(),
        user_id: user_id.clone(),
    };

    match db.create_otp(new_otp) {
        Ok(_) => {
            if let Err(e) = send_verification_email(user_email.as_str(), otp_code).await {
                eprint!("Failed to send email: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to send verification email"
                }));
            } else {
                HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": format!("OTP sent to {}", user_email),
                    "expires_in_minutes": 2
                }))
            }
        }
        Err(e) => {
            eprintln!("Failed to create OTP: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to create OTP"
            }))
        }
    }
}
