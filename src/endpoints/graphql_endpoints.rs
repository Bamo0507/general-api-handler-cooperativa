use actix_web::{web, HttpResponse};
use juniper::{
    http::{graphiql::graphiql_source, GraphQLRequest},
    GraphQLType, GraphQLTypeAsync,
};

use super::handlers::{
    configs::schema_configs::{GeneralContext, GeneralSchema},
    graphql::payment::{create_payment_schema, PaymentQuery},
};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut web::ServiceConfig) {
    let payment_schema = create_payment_schema();
    config
        .app_data(payment_schema)
        .service(web::resource("/graphql").route(web::post().to(graphql::<PaymentQuery>)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql)));
}

//For displaying the grapiql page (for trying queries)
async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql", None);

    return HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html);
}

async fn graphql<GenericQuery>(
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
    let context = GeneralContext {};
    let res = data.execute(&schema, &context).await;

    return HttpResponse::Ok().json(res);
}
