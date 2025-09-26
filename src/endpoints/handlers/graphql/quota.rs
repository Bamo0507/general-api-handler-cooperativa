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
}
