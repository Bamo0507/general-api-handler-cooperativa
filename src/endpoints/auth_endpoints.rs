use actix_web::web::{get, post, resource, ServiceConfig};

use super::handlers::rest::auth::{user_login, user_sign_up};

//TODO: add the necessary config
pub fn auth_config(config: &mut ServiceConfig) {
    config
        .service(resource("/general/signup").route(post().to(user_sign_up)))
        .service(resource("/general/login").route(get().to(user_login)));
}
