use actix_web::{web, HttpResponse};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};

use super::handlers::graphql::payment::{create_payment_schema, PaymentContext, PaymentSchema};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut web::ServiceConfig) {
    let payment_schema = create_payment_schema();
    config
        .app_data(payment_schema)
        .service(web::resource("/graphql").route(web::post().to(graphql)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql)));
}

//For displaying the grapiql page (for trying queries)
async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql", None);

    return HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
}

//TODO: Make this to accept Generic Schemas
async fn graphql(
    data: web::Json<GraphQLRequest>,
    schema: web::Data<PaymentSchema>,
) -> HttpResponse {
    let context = PaymentContext {};
    let res = data.execute(&schema, &context).await;

    return HttpResponse::Ok().json(res);
}
