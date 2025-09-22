use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::graphql::Cuota,
};
use chrono::Datelike;
use juniper::GraphQLObject;

#[derive(GraphQLObject, Debug)]
pub struct CuotaAfiliadoMensualResponse {
    pub identifier: String,
    pub user_id: String,
    pub monto: f64,
    pub nombre: String,
    pub fecha_vencimiento: String,
    pub extraordinaria: bool,
}

#[derive(GraphQLObject, Debug)]
pub struct CuotaPrestamoResponse {
    pub user_id: String,
    pub monto: f64,
    pub fecha_vencimiento: String,
    pub monto_pagado: f64,
    pub multa: f64,
    pub pagada_por: Option<String>,
    pub tipo: String,
    pub loan_id: Option<String>,
    pub pagada: bool,
    pub numero_cuota: Option<i32>,
    pub nombre_prestamo: Option<String>,
}

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


        /// Refactorizado: Retorna las cuotas mensuales de afiliado pendientes en formato completo fundamentado según docs/api-quota-response-format.md
        /// Cada objeto incluye: identifier, user_id, monto, nombre, fecha_vencimiento, extraordinaria
        pub async fn get_cuotas_afiliado_mensuales_formateadas(
    context: &GeneralContext,
    access_token: String,
) -> Result<Vec<CuotaAfiliadoMensualResponse>, String> {
    let afiliados = context.payment_repo().get_all_users_for_affiliates()?;
    let hoy = chrono::Utc::now().date_naive();
    let mut resultado = Vec::new();
    for afiliado in afiliados {
        let cuotas = context.cuota_repo().get_cuotas_afiliado_pendientes(afiliado.user_id.clone())?;
        for cuota in cuotas {
            if let Some(fecha_str) = &cuota.fecha_vencimiento {
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
                        resultado.push(CuotaAfiliadoMensualResponse {
                            identifier,
                            user_id: access_token.clone(),
                            monto: cuota.monto,
                            nombre,
                            fecha_vencimiento: fecha_str.clone(),
                            extraordinaria: cuota.extraordinaria.unwrap_or(false),
                        });
                    }
                }
            }
        }
    }
    Ok(resultado)
}
    /// Retorna solo las cuotas de préstamo pendientes para el usuario (SCRUM-255, lógica fundamentada)
    pub async fn get_cuotas_prestamo_pendientes(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<Cuota>, String> {
        context.cuota_repo().get_cuotas_prestamo_pendientes(access_token)
    }

    /// Retorna las cuotas de préstamo pendientes en formato completo según docs/api-quota-response-format.md
    /// Cada objeto incluye: user_id, monto, fecha_vencimiento, monto_pagado, multa, pagada_por, tipo, loan_id, pagada, numero_cuota, nombre_prestamo
    pub async fn get_cuotas_prestamo_pendientes_formateadas(
        context: &GeneralContext,
        access_token: String,
    ) -> Result<Vec<CuotaPrestamoResponse>, String> {
        let cuotas = context.cuota_repo().get_cuotas_prestamo_pendientes(access_token.clone())?;
        let mut resultado = Vec::new();
        
        for cuota in cuotas {
            resultado.push(CuotaPrestamoResponse {
                user_id: access_token.clone(),
                monto: cuota.monto,
                fecha_vencimiento: cuota.fecha_vencimiento.unwrap_or_default(),
                monto_pagado: cuota.monto_pagado,
                multa: cuota.multa,
                pagada_por: cuota.pagada_por,
                tipo: format!("{:?}", cuota.tipo), // Convierte el enum a string
                loan_id: cuota.loan_id,
                pagada: cuota.pagada.unwrap_or(false),
                numero_cuota: cuota.numero_cuota,
                nombre_prestamo: None, // Por ahora vacío porque no está implementado
            });
        }
        
        Ok(resultado)
    }
}
