use actix_web::{web, HttpResponse};

use super::handlers::{
    configs::{connection_pool::get_pool_connection, schema::create_schema},
    graphql::{graphql, loan::LoanQuery, payment::PaymentQuery},
};

//This is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut web::ServiceConfig) {
    //General variables
    let pool = get_pool_connection();

    //Instance of Schemas with generic function
    let payment_schema = create_schema(PaymentQuery {});
    let loan_schema = create_schema(LoanQuery {});

    config
        .app_data(pool)
        .app_data(payment_schema)
        .app_data(loan_schema)
        .service(web::resource("/graphql/payment").route(web::post().to(graphql::<PaymentQuery>)))
        .service(web::resource("/graphql/loan").route(web::post().to(graphql::<LoanQuery>)));
}
