use crate::{
    endpoints::handlers::configs::schema_configs::GeneralSchema,
    repos::graphql::payment::{Payment, PaymentRepo},
};
use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

// Fucking all context

#[derive(Clone)]
pub struct PaymentContext {}

impl PaymentContext {
    fn payment_repo(&self) -> PaymentRepo {
        return PaymentRepo::init();
    }
}

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for PaymentContext {}

//TODO: see how to separate this later
pub struct PaymentQuery {}

#[juniper::graphql_object(
    Context = PaymentContext,
)]
impl PaymentQuery {
    //TODO: add the necesary possible queries

    pub async fn get_history(context: &PaymentContext) -> Vec<Payment> {
        return context.payment_repo().get_history();
    }
}

//* Schemas side
pub type PaymentSchema = RootNode<
    'static,
    PaymentQuery,
    EmptyMutation<PaymentContext>,
    EmptySubscription<PaymentContext>,
>;

pub fn create_payment_schema() -> web::Data<GeneralSchema<PaymentQuery>> {
    let schema = RootNode::new(
        PaymentQuery {},
        EmptyMutation::new(),
        EmptySubscription::new(),
    );

    // I always need for passing the squema to actix
    return web::Data::new(schema);
}
