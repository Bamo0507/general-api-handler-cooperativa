use actix_web::{web, HttpResponse};

use crate::{
    models::auth::{LoginInfo, SignUpInfo, SecurityQuestionsResponse, ValidateSecurityAnswerRequest, 
                   ValidateSecurityAnswerResponse, ResetPasswordRequest, SECURITY_QUESTIONS},
    repos::auth::{create_user_with_access_token, get_user_access_token, validate_security_answer,
                 reset_password},
};

//Just for returning the access token for the user
//Won't be use on mobile prod
pub async fn user_sign_up(user_data: web::Json<SignUpInfo>) -> HttpResponse {
    let data = user_data.into_inner();

    HttpResponse::Ok().json(create_user_with_access_token(
        data.user_name.to_string().clone(),
        data.pass_code.to_string().clone(),
        data.real_name.to_string().clone(), //Let's assume it goes correctly
    ))
}

//This will be used on mobile prod
pub async fn user_login(user_data: web::Query<LoginInfo>) -> HttpResponse {
    let data = user_data.into_inner();
    println!("{data:?}");

    HttpResponse::Ok().json(get_user_access_token(
        data.user_name.to_string(),
        data.pass_code.to_string(),
    ))
}

/// Get security questions for password recovery
/// 
/// GET /general/security-questions?user_name=username
pub async fn get_security_questions_handler(query: web::Query<String>) -> HttpResponse {
    let _user_name = query.into_inner();
    
    let response = SecurityQuestionsResponse {
        questions: SECURITY_QUESTIONS.iter().map(|q| q.to_string()).collect(),
    };
    
    HttpResponse::Ok().json(response)
}

/// Validate security answer for password recovery
/// 
/// POST /general/validate-security-answer
pub async fn validate_security_answer_handler(
    body: web::Json<ValidateSecurityAnswerRequest>,
) -> HttpResponse {
    let data = body.into_inner();
    
    match validate_security_answer(data.user_name, data.security_answer) {
        Ok(_) => {
            HttpResponse::Ok().json(ValidateSecurityAnswerResponse {
                message: "Respuesta válida".to_string(),
            })
        }
        Err(err) => {
            HttpResponse::BadRequest().json(ValidateSecurityAnswerResponse {
                message: err.message,
            })
        }
    }
}

/// Reset password with security answer validation
/// 
/// POST /general/reset-password
pub async fn reset_password_handler(
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let data = body.into_inner();
    
    match reset_password(data.user_name, data.security_answer, data.new_pass_code) {
        Ok(token_info) => HttpResponse::Ok().json(token_info),
        Err(err) => {
            HttpResponse::BadRequest().json(crate::models::StatusMessage {
                message: err.message,
            })
        }
    }
}
