use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands, JsonCommands, Value as RedisValue, from_redis_value};
use serde_json::from_str;

use crate::models::graphql::Quota;
use crate::repos::auth::utils::hashing_composite_key;

pub struct QuotaRepo {
    pub pool: Data<Pool<Client>>,
}

impl QuotaRepo {
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
            let Quota = quota_vec.get(0).cloned();
            if let Some(Quota) = Quota {
                // 1. Solo tipo afiliado
                if Quota.tipo != crate::models::graphql::TipoQuota::Afiliado {
                    continue;
                }
                // 2. No pagada
                if Quota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento <= mes actual
                if let Some(fecha_str) = &Quota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        // Solo mostrar quotas hasta el mes actual
                        if fecha.year() > today.year() || (fecha.year() == today.year() && fecha.month() > today.month()) {
                            continue;
                        }
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la Quota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la Quota
                }
                // 4. Permite pagos por terceros (pagada_por puede ser distinto a user_id)
                quotas.push(Quota);
            }
        }
        Ok(quotas)
    }
    // ESTE MÉTODO SE USA PARA TESTING, NO TIENE LÓGICA DE NEGOCIO, NO USAR EN PRODUCCIÓN
    pub fn save_quota(&self, access_token: String, Quota: &Quota) -> Result<(), String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let key = match &Quota.tipo {
            crate::models::graphql::TipoQuota::Prestamo => {
                let loan_id = Quota.loan_id.as_deref().ok_or("loan_id es requerido para quotas de préstamo")?;
                let fecha = Quota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para quotas de préstamo")?;
                format!("users:{}:loans:{}:quotas:{}", db_access_token, loan_id, fecha)
            },
            crate::models::graphql::TipoQuota::Afiliado => {
                let fecha = Quota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para quotas de afiliado")?;
                format!("users:{}:quotas_afiliado:{}", db_access_token, fecha)
            }
        };
        con.json_set::<_, _, _, ()>(key, "$", Quota).map_err(|_| "Error saving Quota")?;
        Ok(())
    }

    // Consulta todas las quotas  pendientes para un usuario a nivel general
    pub fn get_quotas_pendientes(&self, access_token: String) -> Result<Vec<Quota>, String> {
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
            let Quota = quota_vec.get(0).cloned();
            if let Some(Quota) = Quota {
                quotas.push(Quota);
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
            let Quota = quota_vec.get(0).cloned();
            if let Some(Quota) = Quota {
                // Filtrado fundamentado:
                // 1. Solo tipo préstamo
                if Quota.tipo != crate::models::graphql::TipoQuota::Prestamo {
                    continue;
                }
                // 2. No pagada
                if Quota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento >= hoy
                if let Some(fecha_str) = &Quota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        if fecha < today {
                            continue;
                        }
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la Quota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la Quota
                }
                quotas.push(Quota);
            }
        }
        Ok(quotas)
    }

        /// Obtiene todas las quotas asociadas a un loan_id, sin filtrar por estado de pago ni vigencia.
    pub fn get_quotas_por_loan_id(&self, access_token: String, loan_id: String) -> Result<Vec<Quota>, String> {
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
                let Quota = quota_vec.get(0).cloned();
                if let Some(Quota) = Quota {
                    // Filtrado fundamentado: solo por loan_id
                    if let Some(ref quota_loan_id) = Quota.loan_id {
                        if quota_loan_id == &loan_id {
                            quotas.push(Quota);
                        }
                    }
                }
            }
            Ok(quotas)
        }
}
