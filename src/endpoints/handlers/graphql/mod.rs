pub(crate) mod loan;
pub(crate) mod payment;

use actix_web::{web, HttpResponse};
use juniper::{http::GraphQLRequest, GraphQLType, GraphQLTypeAsync};
use r2d2::Pool;
use redis::Client;

use super::configs::schema::{GeneralContext, GeneralSchema};

// Graphql creator schema generic
pub async fn graphql<GenericQuery>(
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
