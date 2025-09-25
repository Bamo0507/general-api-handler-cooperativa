use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::graphql::{Affiliate, Payment, PaymentHistory},
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
    /// Obtiene todos los pagos de todos los socios
    pub async fn get_all_payments(context: &GeneralContext) -> Result<Vec<Payment>, String> {
        context.payment_repo().get_all_payments()
    }

    /// Get's all the members names with pg_id (mostly for payments and affiliates)
    pub async fn get_all_memembers(context: &GeneralContext) -> Result<Vec<Affiliate>, String> {
        context.payment_repo().get_all_users_for_affiliates()
    }

    pub async fn create_user_payment(context: &GeneralContext) -> Result<String, String> {
        //TODO: implement repo method call
        todo!()
    }
}
