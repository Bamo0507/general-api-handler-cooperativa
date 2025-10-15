use crate::{endpoints::handlers::configs::schema::GeneralContext, models::graphql::Fine};

pub struct FineQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl FineQuery {
    //TODO: add the necesary possible queries

    /// query for returning the fines of one specific user
    pub async fn get_fines_by_id(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Fine>, String> {
        context.fine_repo().get_user_fines(access_token)
    }
}

pub struct FineMutation;

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl FineMutation {
    /// Mutation for creating fines
    pub async fn create_fine(
        context: &GeneralContext,
        affiliate_key: String,
        amount: f64,
        motive: String,
    ) -> Result<String, String> {
        context
            .fine_repo()
            .create_fine(affiliate_key, amount as f32, motive)
    }
}
