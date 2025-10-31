use crate::{endpoints::handlers::configs::schema::GeneralContext, models::graphql::Loan};

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
pub struct LoanQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl LoanQuery {
    //TODO: add the necesary possible queries

    pub async fn get_user_loans(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Loan>, String> {
        context.loan_repo().get_user_loans(access_token)
    }
}

pub struct LoanMutation;

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl LoanMutation {
    pub async fn create_user_loan(
        context: &GeneralContext,
        affiliate_key: String,
        total_quota: i32,
        base_needed_payment: f64,
        reason: String,
    ) -> Result<String, String> {
        context
            .loan_repo()
            .create_loan(affiliate_key, total_quota, base_needed_payment, reason)
    }
}
