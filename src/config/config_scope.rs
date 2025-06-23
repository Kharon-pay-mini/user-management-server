use crate::routes::healthz::check_health;
use crate::routes::users::profile::{
    create_user_handler, get_user_bank_accounts_handler, get_user_handler, logout_handler,
    register_user_bank_account_handler, resend_otp_handler, validate_otp_handler,
};
use actix_web::web::{self, service};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/v1")
        .service(check_health)
        .service(create_user_handler)
        .service(register_user_bank_account_handler)
        .service(get_user_bank_accounts_handler)
        .service(get_user_handler)
        .service(validate_otp_handler)
        .service(logout_handler)
        .service(resend_otp_handler);
    conf.service(scope);
}
