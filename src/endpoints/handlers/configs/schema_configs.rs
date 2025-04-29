use juniper::{EmptyMutation, EmptySubscription, RootNode};

use crate::repos::graphql::{loan::LoanRepo, payment::PaymentRepo};

#[derive(Clone)]
pub struct GeneralContext {}

impl GeneralContext {
    pub fn payment_repo(&self) -> PaymentRepo {
        return PaymentRepo::init();
    }
    pub fn loan_repo(&self) -> LoanRepo {
        return LoanRepo::init();
    }
}

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
impl juniper::Context for GeneralContext {}

pub type GeneralSchema<T> =
    RootNode<'static, T, EmptyMutation<GeneralContext>, EmptySubscription<GeneralContext>>;
