use actix_web::{web, HttpResponse};
use juniper::{
    http::{graphiql::graphiql_source, GraphQLRequest},
    GraphQLType, GraphQLTypeAsync,
};

use super::handlers::{
    configs::schema_configs::GeneralSchema,
    graphql::payment::{create_payment_schema, PaymentContext, PaymentQuery},
};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut web::ServiceConfig) {
    let payment_schema = create_payment_schema();
    config
        .app_data(payment_schema)
        .service(
            web::resource("/graphql/Payment")
                .route(web::post().to(graphql::<PaymentQuery, PaymentContext>)),
        )
        .service(web::resource("/graphiql").route(web::get().to(graphiql)));
}

//For displaying the grapiql page (for trying queries)
async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql", None);

    return HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
}

//  expected reference `&<T as GraphQLValue>::Context`
//      found reference `&PaymentContext`
