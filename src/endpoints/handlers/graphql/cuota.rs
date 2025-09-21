use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::graphql::Cuota,
};

pub struct CuotaQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl CuotaQuery {
    /// Retorna las cuotas pendientes de préstamo para el usuario
    pub async fn get_cuotas_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Cuota>, String> {
        context.cuota_repo().get_cuotas_pendientes(access_token)
    }
}
