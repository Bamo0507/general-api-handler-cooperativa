use actix_web::{web, HttpResponse};

use crate::{
    models::auth::{LoginInfo, SignUpInfo},
    repos::auth::{create_user_with_access_token, get_user_access_token},
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
pub async fn user_login(user_data: web::Json<LoginInfo>) -> HttpResponse {
    let data = user_data.into_inner();

    HttpResponse::Ok().json(get_user_access_token(
        data.user_name.to_string(),
        data.pass_code.to_string(),
    ))
}
