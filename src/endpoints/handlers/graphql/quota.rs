use crate::endpoints::handlers::configs::schema::GeneralContext;
use crate::models::graphql::Quota;

pub struct QuotaQuery {}
const MESES_ES: [&str; 12] = [
    "Enero",
    "Febrero",
    "Marzo",
    "Abril",
    "Mayo",
    "Junio",
    "Julio",
    "Agosto",
    "Septiembre",
    "Octubre",
    "Noviembre",
    "Diciembre",
];

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl QuotaQuery {
    /// Retorna todas las cuotas pendientes para el usuario usando el modelo Quota unificado
    pub async fn get_pending_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context.quota_repo().get_pending_quotas(access_token)
    }

    /// Retorna las cuotas mensuales de afiliado pendientes con campos adicionales para frontend
    pub async fn get_monthly_affiliate_quota(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        let afiliados = context.payment_repo().get_all_users_for_affiliates()?;
        context
            .quota_repo()
            .get_monthly_affiliate_quota(afiliados, access_token)
    }

    /// Retorna solo las cuotas de préstamo pendientes filtradas por lógica de negocio
    pub async fn get_quotas_prestamo_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context
            .quota_repo()
            .get_quotas_prestamo_pendientes(access_token)
    }

    /// Retorna las cuotas de préstamo pendientes con campos adicionales para frontend
    pub async fn get_pending_loans_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context.quota_repo().get_pending_loans_quotas(access_token)
    }
}

pub struct QuotaMutation;

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl QuotaMutation {
    /// Crea una nueva cuota en el sistema (implementación pendiente)
    pub async fn create_quota() -> Result<String, String> {
        todo!()
    }

    /// DESARROLLO: Inserta 20 cuotas dummy (10 afiliado + 10 préstamo) para testing
    /// Usa convenciones: user_id = access_token, loan_id SHA256, fechas 2025
    pub async fn insert_dummy_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<String, String> {
        use crate::repos::graphql::utils::dummy_data::insert_20_dummy_quotas;
        let pool = context.pool.clone();
        insert_20_dummy_quotas(pool, access_token)
    }

    /// DESARROLLO: Borra todas las cuotas de un usuario para limpiar datos de testing
    pub async fn delete_all_user_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<String, String> {
        use crate::repos::graphql::utils::dummy_data::delete_all_quotas;
        let pool = context.pool.clone();
        delete_all_quotas(pool, access_token)
    }
}
