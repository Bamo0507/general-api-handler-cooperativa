pub mod fine;
pub mod loan;
pub mod payment;
pub mod quota;

use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use juniper::{http::GraphQLRequest, GraphQLType, GraphQLTypeAsync};
use r2d2::Pool;
use redis::Client;

use aws_sdk_s3::Client as S3Client;

use super::configs::schema::{GeneralContext, GeneralSchema};

// Graphql creator schema generic
pub async fn graphql<GenericQuery, GenericMutation>(
    client: Data<S3Client>,
    pool: Data<Pool<Client>>,
    data: Json<GraphQLRequest>,
    schema: Data<GeneralSchema<GenericQuery, GenericMutation>>,
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

    GenericMutation: GraphQLTypeAsync<Context = GeneralContext, TypeInfo = ()>
        //Also here in the context, a Trait with that specific Type/Struct
        + GraphQLType<Context = GeneralContext>
        + Send
        + Sync,
    GenericMutation::TypeInfo: Send + Sync,
{
    let context = GeneralContext { pool, client };

    let res = data.execute(&schema, &context).await;

    HttpResponse::Ok().json(res)
}
