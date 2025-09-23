use crate::endpoints::handlers::configs::schema::GeneralContext;
use crate::models::graphql::{Quota, QuotaAfiliadoMensualResponse, QuotaPrestamoResponse};
use chrono::Datelike;

pub struct QuotaQuery {}

#[juniper::graphql_object(
    Context = GeneralContext,
)]
impl QuotaQuery {
    /// Retorna las quotas pendientes de préstamo para el usuario
    pub async fn get_quotas_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context.quota_repo().get_quotas_pendientes(access_token)
    }


        /// Refactorizado: Retorna las quotas mensuales de afiliado pendientes en formato completo fundamentado según docs/api-quota-response-format.md
        /// Cada objeto incluye: identifier, user_id, monto, nombre, fecha_vencimiento, extraordinaria
        pub async fn get_quotas_afiliado_mensuales_formateadas(
    context: &GeneralContext,
    access_token: String,
) -> Result<Vec<QuotaAfiliadoMensualResponse>, String> {
    let afiliados = context.payment_repo().get_all_users_for_affiliates()?;
    let hoy = chrono::Utc::now().date_naive();
    let mut resultado = Vec::new();
    for afiliado in afiliados {
        let quotas = context.quota_repo().get_quotas_afiliado_pendientes(afiliado.user_id.clone())?;
    for quota in quotas {
            if let Some(fecha_str) = &quota.fecha_vencimiento {
                if let Ok(fecha) = chrono::NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                    if fecha <= hoy {
                        let mes = match fecha.month() {
                            1 => "Enero",
                            2 => "Febrero",
                            3 => "Marzo",
                            4 => "Abril",
                            5 => "Mayo",
                            6 => "Junio",
                            7 => "Julio",
                            8 => "Agosto",
                            9 => "Septiembre",
                            10 => "Octubre",
                            11 => "Noviembre",
                            12 => "Diciembre",
                            _ => "Mes",
                        };
                        let anio = fecha.year();
                        let nombre = afiliado.name.clone();
                        let identifier = format!("{} - {} {}", nombre, mes, anio);
                        resultado.push(QuotaAfiliadoMensualResponse {
                            identifier,
                            user_id: access_token.clone(),
                            monto: quota.monto,
                            nombre,
                            fecha_vencimiento: fecha_str.to_string(),
                            extraordinaria: quota.extraordinaria.unwrap_or(false),
                        });
                    }
                }
            }
        }
    }
    Ok(resultado)
}
    /// Retorna solo las quotas de préstamo pendientes para el usuario (SCRUM-255, lógica fundamentada)
    pub async fn get_quotas_prestamo_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Quota>, String> {
        context.quota_repo().get_quotas_prestamo_pendientes(access_token)
    }

    /// Retorna las quotas de préstamo pendientes en formato completo según docs/api-quota-response-format.md
    /// Cada objeto incluye: user_id, monto, fecha_vencimiento, monto_pagado, multa, pagada_por, tipo, loan_id, pagada, numero_quota, nombre_prestamo
    pub async fn get_quotas_prestamo_pendientes_formateadas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<QuotaPrestamoResponse>, String> {
        let quotas = context.quota_repo().get_quotas_prestamo_pendientes(access_token.clone())?;
        let mut resultado = Vec::new();
        
        for quota in quotas {
            resultado.push(QuotaPrestamoResponse {
                user_id: access_token.clone(),
                monto: quota.monto,
                fecha_vencimiento: quota.fecha_vencimiento.unwrap_or_default(),
                monto_pagado: quota.monto_pagado,
                multa: quota.multa,
                pagada_por: quota.pagada_por,
                tipo: format!("{:?}", quota.tipo), // Convierte el enum a string
                loan_id: quota.loan_id,
                pagada: quota.pagada.unwrap_or(false),
                numero_quota: quota.numero_quota,
                nombre_prestamo: None, // Por ahora vacío porque no está implementado
            });
        }
        
        Ok(resultado)
    }
}
