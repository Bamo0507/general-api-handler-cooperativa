use crate::{
    endpoints::handlers::configs::schema_configs::GeneralSchema,
    repos::graphql::loan::{Loan, LoanRepo},
};
use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

// Fucking all context

#[derive(Clone)]
pub struct LoanContext {}

impl LoanContext {
    fn payment_repo(&self) -> LoanRepo {
        return LoanRepo::init();
    }
}

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for LoanContext {}

//TODO: see how to separate this later
pub struct LoanQuery {}

#[juniper::graphql_object(
    Context = LoanContext,
)]
impl LoanQuery {
    //TODO: add the necesary possible queries

    pub async fn get_history(context: &LoanContext) -> Vec<Loan> {
        return context.payment_repo().get_history();
    }
}

pub fn create_loan_schema() -> web::Data<GeneralSchema<LoanQuery>> {
    let schema = RootNode::new(LoanQuery {}, EmptyMutation::new(), EmptySubscription::new());

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
