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
        todo!()
    }
}
