use crate::routes::healthz::check_health;
use crate::routes::users::profile::{
    create_user_handler, get_user_bank_accounts_handler, get_user_handler,
    register_user_bank_account_handler,
};
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/v1")
        .service(check_health)
        .service(create_user_handler)
        .service(register_user_bank_account_handler)
        .service(get_user_bank_accounts_handler)
        .service(get_user_handler);
    conf.service(scope);
}
