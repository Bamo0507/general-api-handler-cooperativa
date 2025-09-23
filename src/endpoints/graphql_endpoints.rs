use actix_web::web::{post, resource, ServiceConfig};

use super::handlers::{
    configs::{connection_pool::get_pool_connection, schema::create_schema},
    graphql::{fine::FineQuery, graphql, loan::LoanQuery, payment::PaymentQuery, Quota::CuotaQuery},
};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut ServiceConfig) {
    //General variables
    let pool = get_pool_connection();

    //Instance of Schemas with generic function
    let payment_schema = create_schema(PaymentQuery {});
    let loan_schema = create_schema(LoanQuery {});
    let fine_schema = create_schema(FineQuery {});
    let cuota_schema = create_schema(CuotaQuery {});

    config
        .app_data(pool)
        .app_data(payment_schema)
        .app_data(loan_schema)
        .app_data(fine_schema)
        .app_data(cuota_schema)
        .service(resource("/graphql/payment").route(post().to(graphql::<PaymentQuery>)))
        .service(resource("/graphql/loan").route(post().to(graphql::<LoanQuery>)))
        .service(resource("/graphql/fine").route(post().to(graphql::<FineQuery>)))
        .service(resource("/graphql/Quota").route(post().to(graphql::<CuotaQuery>)));
}
