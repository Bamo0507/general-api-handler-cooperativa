use crate::{endpoints::handlers::configs::schema_configs::GeneralContext, models::graphql::Loan};

//* Queries

//I don't like this rust boilerplate, but meh, Ig rust doesn't adapt that good to abstractions
pub struct LoanQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl LoanQuery {
    //TODO: add the necesary possible queries

    pub async fn get_all(context: &GeneralContext, id: String) -> Result<Vec<Loan>, String> {
        return context.loan_repo().get_user_loans(id);
    }
}
