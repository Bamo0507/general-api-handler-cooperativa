use actix_web::web::{get, post, resource, ServiceConfig};

use super::handlers::rest::auth::{user_login, user_sign_up, get_security_questions_handler,
                                 validate_security_answer_handler, reset_password_handler, configure_all_security_answers_handler};

//TODO: add the necessary config
pub fn auth_config(config: &mut ServiceConfig) {
    config
        .service(resource("/general/signup").route(post().to(user_sign_up)))
        .service(resource("/general/login").route(get().to(user_login)))
        .service(resource("/general/security-questions").route(get().to(get_security_questions_handler)))
        .service(resource("/general/validate-security-answer").route(post().to(validate_security_answer_handler)))
        .service(resource("/general/reset-password").route(post().to(reset_password_handler)))
        .service(resource("/general/configure-security-answers").route(post().to(configure_all_security_answers_handler)));
}
