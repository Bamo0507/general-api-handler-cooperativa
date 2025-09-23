use chrono::Datelike;
use crate::models::graphql::{QuotaAfiliadoMensualResponse, QuotaPrestamoResponse, Affiliate};
const MESES_ES: [&str; 12] = [
    "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio", "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"
];
use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands, JsonCommands, Value as RedisValue, from_redis_value};
use serde_json::from_str;

use crate::models::graphql::{Quota, TipoQuota};
use crate::repos::auth::utils::hashing_composite_key;

pub struct QuotaRepo {
    pub pool: Data<Pool<Client>>,
}

impl QuotaRepo {
    /// Formatea las cuotas mensuales de afiliado pendientes según docs/api-quota-response-format.md
    pub fn get_monthly_affiliate_quota(&self, affiliates: Vec<Affiliate>, access_token: String) -> Result<Vec<QuotaAfiliadoMensualResponse>, String> {
        use chrono::NaiveDate;
        let hoy = chrono::Utc::now().date_naive();
        let mut resultado = Vec::new();
        for afiliado in affiliates {
            let quotas = self.get_quotas_afiliado_pendientes(afiliado.user_id.clone())?;
            for quota in quotas {
                if let Some(fecha_str) = &quota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        if fecha <= hoy {
                            let mes = MESES_ES.get((fecha.month() as usize).saturating_sub(1)).unwrap_or(&"Mes");
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

    /// Formatea las cuotas de préstamo pendientes según docs/api-quota-response-format.md
    pub fn get_pending_loans_quotas(&self, access_token: String) -> Result<Vec<QuotaPrestamoResponse>, String> {
        let quotas = self.get_quotas_prestamo_pendientes(access_token.clone())?;
        let mut resultado = Vec::new();
        for quota in quotas {
            resultado.push(QuotaPrestamoResponse {
                user_id: access_token.clone(),
                monto: quota.monto,
                fecha_vencimiento: quota.fecha_vencimiento.clone().unwrap_or_default(),
                monto_pagado: quota.monto_pagado,
                multa: quota.multa,
                pagada_por: quota.pagada_por.clone(),
                tipo: format!("{:?}", quota.tipo),
                loan_id: quota.loan_id.clone(),
                pagada: quota.pagada.unwrap_or(false),
                numero_quota: quota.numero_quota,
                nombre_prestamo: None,
            });
        }
        Ok(resultado)
    }
    /// Consulta solo las quotas de afiliado pendientes para un usuario   
     /// - Solo quotas de tipo afiliado
    /// - No pagadas (pagada == false)
    /// - Fecha de vencimiento <= mes actual (no futuras)
    /// - Permite pagos por terceros (pagada_por)
    pub fn get_quotas_afiliado_pendientes(&self, access_token: String) -> Result<Vec<Quota>, String> {
        use chrono::{NaiveDate, Datelike};
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_afiliado = format!("users:{}:quotas_afiliado:*", db_access_token);
        let keys_afiliado: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_afiliado).map_err(|_| "Error scanning keys afiliado")?;
            iter.collect()
        };
    let mut quotas = Vec::new();
        let today = chrono::Local::now().date_naive();
        for key in keys_afiliado.iter() {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting Quota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            let quota_vec = from_str::<Vec<Quota>>(&nested).map_err(|_| "Error deserializing Quota")?;
            if quota_vec.len() != 1 {
                continue; // Si el array no es de tamaño 1, ignora la Quota
            }
            let quota = quota_vec.get(0).cloned();
            if let Some(quota) = quota {
                // 1. Solo tipo afiliado
                if quota.tipo != TipoQuota::Afiliado {
                    continue;
                }
                // 2. No pagada
                if quota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento <= mes actual
                if let Some(fecha_str) = &quota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        // Solo mostrar quotas hasta el mes actual
                        if fecha.year() > today.year() || (fecha.year() == today.year() && fecha.month() > today.month()) {
                            continue;
                        }
                        // Ejemplo de uso del array de meses:
                        // Puedes usar mes_nombre para formatear identificadores, logs, etc.
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la quota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la quota
                }
                // 4. Permite pagos por terceros (pagada_por puede ser distinto a user_id)
                quotas.push(quota);
            }
        }
        Ok(quotas)
    }
    // ESTE MÉTODO SE USA PARA TESTING, NO TIENE LÓGICA DE NEGOCIO, NO USAR EN PRODUCCIÓN
    pub fn save_quota(&self, access_token: String, quota: &Quota) -> Result<(), String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let key = match &quota.tipo {
            TipoQuota::Prestamo => {
                let loan_id = quota.loan_id.as_deref().ok_or("loan_id es requerido para quotas de préstamo")?;
                let fecha = quota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para quotas de préstamo")?;
                format!("users:{}:loans:{}:quotas:{}", db_access_token, loan_id, fecha)
            },
            TipoQuota::Afiliado => {
                let fecha = quota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para quotas de afiliado")?;
                format!("users:{}:quotas_afiliado:{}", db_access_token, fecha)
            }
        };
    con.json_set::<_, _, _, ()>(key, "$", quota).map_err(|_| "Error saving Quota")?;
        Ok(())
    }

    // Consulta todas las quotas  pendientes para un usuario a nivel general
    pub fn get_pending_quotas(&self, access_token: String) -> Result<Vec<Quota>, String> {
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_prestamo = format!("users:{}:loans:*:quotas:*", db_access_token);
        let pattern_afiliado = format!("users:{}:quotas_afiliado:*", db_access_token);

        let keys_prestamo: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
            iter.collect()
        };
        let keys_afiliado: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_afiliado).map_err(|_| "Error scanning keys afiliado")?;
            iter.collect()
        };

        let mut quotas = Vec::new();
        for key in keys_prestamo.iter().chain(keys_afiliado.iter()) {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting Quota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            println!("[DEBUG] Clave: {} | Valor nested: {}", key, nested);
            let quota_vec = from_str::<Vec<Quota>>(&nested).map_err(|_| "Error deserializing Quota")?;
            if quota_vec.len() != 1 {
                return Err(format!("Unexpected Quota array size for key {}: expected 1, got {}", key, quota_vec.len()));
            }
            let quota = quota_vec.get(0).cloned();
            if let Some(quota) = quota {
                quotas.push(quota);
            }
        }
        Ok(quotas)
    }

    /// Consulta solo las quotas de préstamo pendientes para un usuario
    /// - Solo quotas de tipo préstamo (no afiliado)
    /// - No pagadas (pagada == false)
    /// - Fecha de vencimiento >= hoy
    pub fn get_quotas_prestamo_pendientes(&self, access_token: String) -> Result<Vec<Quota>, String> {
        use chrono::NaiveDate;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_prestamo = format!("users:{}:loans:*:quotas:*", db_access_token);
        let keys_prestamo: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
            iter.collect()
        };
        let mut quotas = Vec::new();
        let today = chrono::Local::now().date_naive();
        for key in keys_prestamo.iter() {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting Quota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            let quota_vec = from_str::<Vec<Quota>>(&nested).map_err(|_| "Error deserializing Quota")?;
            if quota_vec.len() != 1 {
                continue; // Si el array no es de tamaño 1, ignora la Quota
            }
            let quota = quota_vec.get(0).cloned();
            if let Some(quota) = quota {
                // Filtrado fundamentado:
                // 1. Solo tipo préstamo
                if quota.tipo != TipoQuota::Prestamo {
                    continue;
                }
                // 2. No pagada
                if quota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento >= hoy
                if let Some(fecha_str) = &quota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        if fecha < today {
                            continue;
                        }
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la quota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la quota
                }
                quotas.push(quota);
            }
        }
        Ok(quotas)
    }

        /// Obtiene todas las quotas asociadas a un loan_id, sin filtrar por estado de pago ni vigencia.
    pub fn get_quota_by_loan_id(&self, access_token: String, loan_id: String) -> Result<Vec<Quota>, String> {
            let db_access_token = hashing_composite_key(&[&access_token]);
            let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
            let pattern_prestamo = format!("users:{}:loans:*:quotas:*", db_access_token);
            let keys_prestamo: Vec<String> = {
                let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
                iter.collect()
            };
            let mut quotas = Vec::new();
            for key in keys_prestamo.iter() {
                let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                    .map_err(|_| format!("Error getting Quota for key {}", key))?;
                let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
                let quota_vec = from_str::<Vec<Quota>>(&nested).map_err(|_| "Error deserializing Quota")?;
                if quota_vec.len() != 1 {
                    continue; // Si el array no es de tamaño 1, ignora la Quota
                }
                let quota = quota_vec.get(0).cloned();
                if let Some(quota) = quota {
                    // Filtrado fundamentado: solo por loan_id
                    if let Some(ref quota_loan_id) = quota.loan_id {
                        if quota_loan_id == &loan_id {
                            quotas.push(quota);
                        }
                    }
                }
            }
            Ok(quotas)
        }
}
