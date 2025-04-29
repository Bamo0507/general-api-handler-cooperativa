use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

use crate::{
    endpoints::handlers::configs::schema_configs::{GeneralContext, GeneralSchema},
    models::graphql::Loan,
};

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions

pub struct LoanQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl LoanQuery {
    //TODO: add the necesary possible queries

    pub async fn get_all(context: &GeneralContext) -> Vec<Loan> {
        return context.loan_repo().get_all();
    }
}

pub fn create_loan_schema() -> web::Data<GeneralSchema<LoanQuery>> {
    let schema = RootNode::new(LoanQuery {}, EmptyMutation::new(), EmptySubscription::new());

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
