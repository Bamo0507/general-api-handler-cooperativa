use actix_web::{
    web::{self, Json},
    HttpResponse,
};
use juniper::{http::GraphQLRequest, GraphQLType, GraphQLTypeAsync};
use r2d2::Pool;
use redis::Client;
use serde_json::json;

use crate::models::general::GeneralInfo;

use super::handlers::{
    configs::{
        connection_pool::get_pool_connection,
        schema::{create_schema, GeneralContext, GeneralSchema},
    },
    graphql::{loan::LoanQuery, payment::PaymentQuery},
};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut web::ServiceConfig) {
    //General variables
    let graphql_info = Json(GeneralInfo {
        api_version: "v 0.0.1".to_string(),
    });

    let pool = get_pool_connection();

    //Instance of Schemas with generic function
    let payment_schema = create_schema(PaymentQuery {});
    let loan_schema = create_schema(LoanQuery {});

    config
        .app_data(pool)
        .app_data(payment_schema)
        .app_data(loan_schema)
        .app_data(graphql_info)
        .service(web::resource("/graphql/payment").route(web::post().to(graphql::<PaymentQuery>)))
        .service(web::resource("/graphql/loan").route(web::post().to(graphql::<LoanQuery>)));
}

async fn graphql<GenericQuery>(
    pool: web::Data<Pool<Client>>,
    data: web::Json<GraphQLRequest>,
    schema: web::Data<GeneralSchema<GenericQuery>>,
) -> HttpResponse
where
    //Okay, first time using the where key word so time to explain

    /*
     * You can specify all type traits and generic information traits
     *
     * First 2 lines specify the GraphQLType "Which is a trait that all GraphQL Queries have",
     *
     * in the query we specify which context are we using, which in this case is made as a generic
     * to each trait.
     *
     * the ones below are more simple, cause we are handeling Async types and Normal types, we need
     * to pass the traits for async managment.
     *
     * Same goes for the TypeInfo of the generic, we have to tell the compiler that all the parts
     * of the generic are Asynchronous.
     */
    GenericQuery: GraphQLTypeAsync<Context = GeneralContext>
        + GraphQLType<Context = GeneralContext>
        + Send
        + Sync,
    GenericQuery::TypeInfo: Send + Sync,
{
    let context = GeneralContext { pool };

    let res = data.execute(&schema, &context).await;

    return HttpResponse::Ok().json(res);
}

async fn general_endpoint_info(general_info: web::Json<GeneralInfo>) -> HttpResponse {
    return HttpResponse::Ok().json(Json(json!(general_info)));
}
