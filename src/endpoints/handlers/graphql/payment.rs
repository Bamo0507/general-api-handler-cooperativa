use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::graphql::{Payment, PaymentHistory},
};

pub struct PaymentQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl PaymentQuery {
    //TODO: add the necesary possible queries

    pub async fn get_history(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<PaymentHistory, String> {
        context.payment_repo().get_user_history(access_token)
    }
}
