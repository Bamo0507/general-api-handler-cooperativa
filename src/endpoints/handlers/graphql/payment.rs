use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::{
        graphql::{Affiliate, Payment, PaymentHistory, PaymentType},
        PayedTo,
    },
};

pub struct PaymentQuery {}

//TODO: refactor error and ok string with status message
#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl PaymentQuery {
    //TODO: add the necesary possible queries

    /// Get's the user's big picture history
    pub async fn get_history(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<PaymentHistory, String> {
        context.payment_repo().get_user_history(access_token)
    }

    /// Get's all user's payments
    pub async fn get_users_payments(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Payment>, String> {
        context.payment_repo().get_user_payments(access_token)
    }

    /// Get's all the members names with there affiliate_keys
    pub async fn get_all_members(context: &GeneralContext) -> Result<Vec<Affiliate>, String> {
        context.payment_repo().get_all_users_for_affiliates()
    }
}

pub struct PaymentMutation;

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl PaymentMutation {
    //TODO: add for adding ticket path
    pub async fn create_user_payment(
        context: &GeneralContext,
        access_token: String,
        name: String,
        total_amount: f64,
        ticket_number: String,
        account_number: String,
        being_payed: Vec<PayedTo>,
    ) -> Result<String, String> {
        context.payment_repo().create_payment(
            access_token,
            name,
            total_amount,
            ticket_number,
            account_number,
            being_payed,
        )
    }
}
