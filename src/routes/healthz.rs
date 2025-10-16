use crate::AppState;
use actix_web::{HttpResponse, Responder, get, web};
use serde_json::json;

#[get("/healthz")]
pub async fn check_health(_data: web::Data<AppState>) -> impl Responder {
    let json_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "health": "Server is active"
        })
    });

    HttpResponse::Ok().json(json_response)
}

#[get("/")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Service is healthy!"
    }))
}
