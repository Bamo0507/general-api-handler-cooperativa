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
                            user_id: afiliado.user_id.clone(),
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
}
