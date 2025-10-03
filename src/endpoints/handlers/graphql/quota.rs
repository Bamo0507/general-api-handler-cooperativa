use crate::endpoints::handlers::configs::schema::GeneralContext;
// TODO: Remove legacy response types imports (QuotaAfiliadoMensualResponse, QuotaPrestamoResponse)
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
    /// Retorna las quotas pendientes de préstamo para el usuario
    pub async fn get_pending_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context.quota_repo().get_pending_quotas(access_token)
    }

    /// TODO: Refactorizado: Retorna las quotas mensuales de afiliado pendientes usando solo el tipo Quota
    /// TODO: Adaptar frontend para consumir el nuevo formato si es necesario
    pub async fn get_monthly_affiliate_quota(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        let afiliados = context.payment_repo().get_all_users_for_affiliates()?;
        context
            .quota_repo()
            .get_monthly_affiliate_quota(afiliados, access_token)
    }
    /// Retorna solo las quotas de préstamo pendientes para el usuario (SCRUM-255, lógica fundamentada)
    pub async fn get_quotas_prestamo_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context
            .quota_repo()
            .get_quotas_prestamo_pendientes(access_token)
    }

    /// TODO: Refactorizado: Retorna las quotas de préstamo pendientes usando solo el tipo Quota
    /// TODO: Adaptar frontend para consumir el nuevo formato si es necesario
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
    pub async fn create_quota() -> Result<String, String> {
        todo!()
    }

    /// MUTACIÓN TEMPORAL PARA INSERTAR DATOS DUMMY - SOLO PARA TESTING/DESARROLLO
    /// Inserta 10 cuotas de afiliado + 10 cuotas de préstamo con fechas aleatorias de 2025
    /// Siguiendo las convenciones exactas del código (user_id = access_token, SHA256 loan_id)
    pub async fn insert_dummy_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<String, String> {
        use crate::repos::graphql::utils::dummy_data::insert_20_dummy_quotas;
        let pool = context.pool.clone();
        insert_20_dummy_quotas(pool, access_token)
    }

    /// MUTACIÓN PARA BORRAR TODAS LAS CUOTAS DE UN USUARIO - SOLO PARA TESTING/DESARROLLO
    /// Borra todas las cuotas existentes de un access_token (para limpiar datos incorretos)
    pub async fn delete_all_user_quotas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<String, String> {
        use crate::repos::graphql::utils::dummy_data::delete_all_quotas;
        let pool = context.pool.clone();
        delete_all_quotas(pool, access_token)
    }
}
