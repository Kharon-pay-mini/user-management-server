use crate::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use chrono::{Duration, Utc};

use serde_json::json;

use crate::auth::{
    auth::verify_admin_role,
    jwt_auth::JwtMiddleware,
    models::{FlaggedUserQuery, LoginHistoryQuery, UserLoginHistoryItem, UserLoginStats},
};
use crate::database::{user_db::UserImpl, user_security_log_db::UserSecurityLogsImpl};
use crate::models::models::CreateUserSchema;

#[get("/admin/users/login-stats")]
pub async fn get_user_login_stats(
    body: web::Json<CreateUserSchema>,
    data: web::Data<AppState>,
    auth: JwtMiddleware,
) -> impl Responder {
    let admin_user_id = auth.user_id;
    let user_email = body.email.to_string().to_lowercase();

    let target_user_id = match data.db.get_user_by_email(user_email.clone()) {
        Ok(user) => user.id,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "User not found"
            }));
        }
    };

    if let Err(response) = verify_admin_role(admin_user_id.as_str().clone(), &data).await {
        return response;
    }

    let total_attempts = match data.db.get_user_security_logs_count(target_user_id.clone()) {
        Ok(count) => count,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve login stats"
            }));
        }
    };

    let failed_attempts = match data.db.get_user_total_failed_logins(target_user_id.clone()) {
        Ok(count) => count,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve login stats"
            }));
        }
    };

    let recent_logs = match data
        .db
        .get_user_security_logs_with_limit(target_user_id.clone(), Some(10))
    {
        Ok(logs) => logs,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve recent logs"
            }));
        }
    };

    let successful_logins = total_attempts - failed_attempts;
    let last_successful_login = recent_logs
        .iter()
        .find(|log| log.failed_login_attempts == 0)
        .and_then(|log| log.created_at);

    let last_failed_login = recent_logs
        .iter()
        .find(|log| log.failed_login_attempts > 0)
        .and_then(|log| log.created_at);

    let is_flagged_for_review = recent_logs.iter().any(|log| log.flagged_for_review);

    let now = Utc::now();
    let twenty_four_hours_ago = now - Duration::hours(24);

    let recent_failed_attempts = recent_logs
        .iter()
        .filter(|log| log.created_at > Some(twenty_four_hours_ago) && log.failed_login_attempts > 0)
        .map(|log| log.failed_login_attempts)
        .sum();

    let stats = UserLoginStats {
        user_id: target_user_id.clone(),
        total_logins: total_attempts,
        successful_logins,
        failed_logins: failed_attempts,
        last_successful_login,
        last_failed_login,
        is_flagged_for_review,
        recent_failed_attempts,
    };

    HttpResponse::Ok().json(json!({
        "status": "success",
        "data": stats
    }))
}

#[get("/admin/users/login-history")]
pub async fn get_user_login_history(
    body: web::Json<LoginHistoryQuery>,
    data: web::Data<AppState>,
    auth: JwtMiddleware,
) -> impl Responder {
    let admin_user_id = auth.user_id;
    println!("admin id: {}", admin_user_id);
    let user_email = body.email.to_string().to_lowercase();

    let target_user_id = match data.db.get_user_by_email(user_email.clone()) {
        Ok(user) => user.id,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "User not found"
            }));
        }
    };

    if let Err(response) = verify_admin_role(admin_user_id.as_str().clone(), &data).await {
        return response;
    }

    let limit = body.limit.unwrap_or(50);
    let offset = body.offset.unwrap_or(0);

    let logs = match data
        .db
        .get_user_security_logs_paginated(target_user_id.clone(), limit, offset)
    {
        Ok(logs) => logs,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve login history"
            }));
        }
    };

    let history: Vec<UserLoginHistoryItem> = logs
        .into_iter()
        .map(|log| UserLoginHistoryItem {
            id: log.log_id,
            timestamp: log.created_at.unwrap_or_else(|| chrono::Utc::now()),
            ip_address: log.ip_address,
            city: log.city,
            country: log.country,
            was_successful: log.failed_login_attempts == 0,
            failed_login_attempts: log.failed_login_attempts,
            flagged_for_review: log.flagged_for_review,
        })
        .collect();

    let total_count = data
        .db
        .get_user_security_logs_count(target_user_id.clone())
        .unwrap_or(history.len() as i64);

    HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "history": history,
            "pagination": {
                "total": total_count,
                "limit": limit,
                "offset": offset,
                "has_more": (offset + limit) < total_count
            }
        }
    }))
}

#[get("/admin/flagged-users")]
pub async fn get_flagged_users(
    body: web::Json<FlaggedUserQuery>,
    data: web::Data<AppState>,
    auth: JwtMiddleware,
) -> impl Responder {
    let admin_user_id = auth.user_id;
    let limit = body.limit.unwrap_or(100);
    let offset = body.offset.unwrap_or(0);

    if let Err(response) = verify_admin_role(admin_user_id.as_str().clone(), &data).await {
        return response;
    }

    let flagged_users = match data.db.get_flagged_users_security_logs() {
        Ok(users) => users,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve flagged users"
            }));
        }
    };

    let total_count = match data.db.get_flagged_users_count() {
        Ok(count) => count,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "Failed to retrieve flagged users count"
            }));
        }
    };

    HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "users": flagged_users,
            "pagination": {
                "total": total_count,
                "limit": limit,
                "offset": offset,
                "has_more": (offset + limit) < total_count
            }
        }
    }))
}
