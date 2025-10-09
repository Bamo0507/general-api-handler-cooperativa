use actix_web::{
    web::{self, Json},
    HttpResponse,
};
use serde_json::json;

use crate::models::GeneralInfo;

pub mod auth_endpoints;
pub mod graphql_endpoints;

pub mod handlers;

pub fn health_config(config: &mut web::ServiceConfig) {
    config.service(web::resource("/health").route(web::get().to(general_endpoint_info)));
}

async fn general_endpoint_info() -> HttpResponse {
    let general_info = Json(GeneralInfo {
        api_version: "v 0.10.0".to_owned(),
    });

    HttpResponse::Ok().json(Json(json!(general_info)))
}
