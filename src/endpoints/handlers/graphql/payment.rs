use actix_web::web;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

use crate::{
    endpoints::handlers::configs::schema_configs::{GeneralContext, GeneralSchema},
    models::graphql::Payment,
};

pub struct PaymentQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl PaymentQuery {
    //TODO: add the necesary possible queries

    pub async fn get_history(context: &GeneralContext) -> Vec<Payment> {
        return context.payment_repo().get_all();
    }
}

//* Schemas side
pub type PaymentSchema = RootNode<
    'static,
    PaymentQuery,
    EmptyMutation<GeneralContext>,
    EmptySubscription<GeneralContext>,
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
