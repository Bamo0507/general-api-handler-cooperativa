use crate::{
    endpoints::handlers::configs::schema::GeneralContext,
    models::graphql::Cuota,
};
use chrono::Datelike;

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

        /// Retorna las cuotas mensuales de afiliado pendientes en formato 'Nombre - Mes Año' para todos los afiliados
        pub async fn get_cuotas_afiliado_mensuales_formateadas(
            context: &GeneralContext,
        ) -> Result<Vec<String>, String> {
            // 1. Obtener todos los afiliados
            let afiliados = context.payment_repo().get_all_users_for_affiliates()?;
            let mut resultado = Vec::new();
            let hoy = chrono::Utc::now().date_naive();
            // 2. Para cada afiliado, obtener sus cuotas pendientes de afiliado
            for afiliado in afiliados {
                let cuotas = context.cuota_repo().get_cuotas_afiliado_pendientes(afiliado.user_id.clone())?;
                for cuota in cuotas {
                    // 3. Filtrar por fecha: solo cuotas con fecha <= hoy
                    if let Some(fecha_str) = &cuota.fecha_vencimiento {
                        if let Ok(fecha) = chrono::NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                            if fecha <= hoy {
                                // Mes en español
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
                                let nombre = &afiliado.name;
                                resultado.push(format!("{} - {} {}", nombre, mes, anio));
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
}
