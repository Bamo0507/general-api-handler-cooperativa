use actix_web::web;

use super::handlers::rest::auth::{user_login, user_sign_up};

//TODO: add the necessary config
pub fn auth_config(config: &mut web::ServiceConfig) {
    config.service(web::resource("/general/signup").route(web::get().to(user_sign_up)));
}
