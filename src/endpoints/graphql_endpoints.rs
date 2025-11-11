use actix_web::web::{post, resource, ServiceConfig};

use crate::endpoints::handlers::graphql::{
    fine::FineMutation, loan::LoanMutation, payment::PaymentMutation, quota::QuotaMutation,
};

use super::handlers::{
    configs::{connection_pool::get_pool_connection, schema::create_schema},
    graphql::{
        fine::FineQuery, graphql, loan::LoanQuery, payment::PaymentQuery, quota::QuotaQuery,
    },
};

//this is pretty much boilerplate for any Graphql api

pub fn graphql_config(config: &mut ServiceConfig) {
    // redis pool
    let pool = get_pool_connection();

    //instance of Schemas with generic function
    let payment_schema = create_schema(PaymentQuery {}, PaymentMutation {});
    let loan_schema = create_schema(LoanQuery {}, LoanMutation {});
    let fine_schema = create_schema(FineQuery {}, FineMutation {});
    let quota_schema = create_schema(QuotaQuery {}, QuotaMutation {});

    config
        .app_data(pool)
        .app_data(payment_schema)
        .app_data(loan_schema)
        .app_data(fine_schema)
        .app_data(quota_schema)
        .service(
            resource("/graphql/payment").route(post().to(graphql::<PaymentQuery, PaymentMutation>)),
        )
        .service(resource("/graphql/loan").route(post().to(graphql::<LoanQuery, LoanMutation>)))
        .service(resource("/graphql/fine").route(post().to(graphql::<FineQuery, FineMutation>)))
        .service(resource("/graphql/quota").route(post().to(graphql::<QuotaQuery, QuotaMutation>)));
}
