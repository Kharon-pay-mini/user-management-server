use crate::routes::healthz::check_health;
use crate::routes::users::profile::{
    confirm_user_bank_account_handler, create_user_handler, get_tokens_handler, get_user_bank_accounts_handler, get_user_handler, get_user_logs_handler, get_wallet_handler, logout_handler, resend_otp_handler, validate_otp_handler, verify_user_bank_account_handler
};
use actix_web::web::{self, service};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/v1")
        .service(check_health)
        .service(create_user_handler)
        .service(verify_user_bank_account_handler)
        .service(confirm_user_bank_account_handler)
        .service(get_user_bank_accounts_handler)
        .service(get_user_handler)
        .service(validate_otp_handler)
        .service(logout_handler)
        .service(get_wallet_handler)
        .service(get_user_logs_handler)
        .service(resend_otp_handler)
        .service(get_tokens_handler);
    conf.service(scope);
}
