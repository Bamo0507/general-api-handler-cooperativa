use crate::{
    endpoints::handlers::configs::schema_configs::GeneralContext, models::graphql::Payment,
};

pub struct PaymentQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl PaymentQuery {
    //TODO: add the necesary possible queries

    pub async fn get_history(context: &GeneralContext, id: String) -> Result<Vec<Payment>, String> {
        return context.payment_repo().get_user_payments(id);
    }
}
