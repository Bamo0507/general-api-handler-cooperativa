use crate::{endpoints::handlers::configs::schema::GeneralContext, models::graphql::Fine};

pub struct FineQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl FineQuery {
    //TODO: add the necesary possible queries

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
    pub async fn creat_fine() -> Result<String, String> {
        todo!()
    }
}
