use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

use crate::repos::graphql::{
    loan_repo::{Loan, LoanRepo},
    payment_repo::{Payment, PaymentRepo},
};

//* All context and repos

#[derive(Clone)]
pub struct Context {}

impl Context {
    fn payment_repo(&self) -> PaymentRepo {
        return PaymentRepo::init();
    }

    fn loan_repo(&self) -> LoanRepo {
        return LoanRepo::init();
    }
}

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for Context {}

//TODO: see how to separate this later
pub struct Query {}

#[juniper::graphql_object(
    Context = Context,
)]
impl Query {
    //TODO: add the necesary possible queries

    pub async fn get_history_payments(context: &Context) -> Vec<Payment> {
        return context.payment_repo().get_history();
    }

    pub async fn get_history_loans(context: &Context) -> Vec<Loan> {
        return context.loan_repo().get_history();
    }
}

//* Schemas side
pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn create_schema() -> web::Data<Schema> {
    let schema = RootNode::new(Query {}, EmptyMutation::new(), EmptySubscription::new());

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
