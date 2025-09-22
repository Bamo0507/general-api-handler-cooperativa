use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands, JsonCommands, Value as RedisValue, from_redis_value};
use serde_json::from_str;

use crate::models::graphql::{Cuota};
use crate::repos::auth::utils::hashing_composite_key;

pub struct CuotaRepo {
    pub pool: Data<Pool<Client>>,
}

impl CuotaRepo {
    /// Consulta solo las cuotas de afiliado pendientes para un usuario   
     /// - Solo cuotas de tipo afiliado
    /// - No pagadas (pagada == false)
    /// - Fecha de vencimiento <= mes actual (no futuras)
    /// - Permite pagos por terceros (pagada_por)
    pub fn get_cuotas_afiliado_pendientes(&self, access_token: String) -> Result<Vec<Cuota>, String> {
        use chrono::{NaiveDate, Datelike};
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_afiliado = format!("users:{}:cuotas_afiliado:*", db_access_token);
        let keys_afiliado: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_afiliado).map_err(|_| "Error scanning keys afiliado")?;
            iter.collect()
        };
        let mut cuotas = Vec::new();
        let today = chrono::Local::now().date_naive();
        for key in keys_afiliado.iter() {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting cuota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            let cuota_vec = from_str::<Vec<Cuota>>(&nested).map_err(|_| "Error deserializing cuota")?;
            if cuota_vec.len() != 1 {
                continue; // Si el array no es de tamaño 1, ignora la cuota
            }
            let cuota = cuota_vec.get(0).cloned();
            if let Some(cuota) = cuota {
                // 1. Solo tipo afiliado
                if cuota.tipo != crate::models::graphql::TipoCuota::Afiliado {
                    continue;
                }
                // 2. No pagada
                if cuota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento <= mes actual
                if let Some(fecha_str) = &cuota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        // Solo mostrar cuotas hasta el mes actual
                        if fecha.year() > today.year() || (fecha.year() == today.year() && fecha.month() > today.month()) {
                            continue;
                        }
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la cuota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la cuota
                }
                // 4. Permite pagos por terceros (pagada_por puede ser distinto a user_id)
                cuotas.push(cuota);
            }
        }
        Ok(cuotas)
    }
    // Guarda una cuota en Redis
    pub fn save_cuota(&self, access_token: String, cuota: &Cuota) -> Result<(), String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let key = match &cuota.tipo {
            crate::models::graphql::TipoCuota::Prestamo => {
                let loan_id = cuota.loan_id.as_deref().ok_or("loan_id es requerido para cuotas de préstamo")?;
                let fecha = cuota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para cuotas de préstamo")?;
                format!("users:{}:loans:{}:cuotas:{}", db_access_token, loan_id, fecha)
            },
            crate::models::graphql::TipoCuota::Afiliado => {
                let fecha = cuota.fecha_vencimiento.as_deref().ok_or("fecha_vencimiento es requerida para cuotas de afiliado")?;
                format!("users:{}:cuotas_afiliado:{}", db_access_token, fecha)
            }
        };
        con.json_set::<_, _, _, ()>(key, "$", cuota).map_err(|_| "Error saving cuota")?;
        Ok(())
    }

    // Consulta todas las cuotas  pendientes para un usuario
    pub fn get_cuotas_pendientes(&self, access_token: String) -> Result<Vec<Cuota>, String> {
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_prestamo = format!("users:{}:loans:*:cuotas:*", db_access_token);
        let pattern_afiliado = format!("users:{}:cuotas_afiliado:*", db_access_token);

        let keys_prestamo: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
            iter.collect()
        };
        let keys_afiliado: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_afiliado).map_err(|_| "Error scanning keys afiliado")?;
            iter.collect()
        };

        let mut cuotas = Vec::new();
        for key in keys_prestamo.iter().chain(keys_afiliado.iter()) {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting cuota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            println!("[DEBUG] Clave: {} | Valor nested: {}", key, nested);
            let cuota_vec = from_str::<Vec<Cuota>>(&nested).map_err(|_| "Error deserializing cuota")?;
            if cuota_vec.len() != 1 {
                return Err(format!("Unexpected cuota array size for key {}: expected 1, got {}", key, cuota_vec.len()));
            }
            let cuota = cuota_vec.get(0).cloned();
            if let Some(cuota) = cuota {
                cuotas.push(cuota);
            }
        }
        Ok(cuotas)
    }

    /// Consulta solo las cuotas de préstamo pendientes para un usuario, fundamentado en la estructura real de Redis y reglas de negocio:
    /// - Solo cuotas de tipo préstamo (no afiliado)
    /// - No pagadas (pagada == false)
    /// - Fecha de vencimiento >= hoy
    pub fn get_cuotas_prestamo_pendientes(&self, access_token: String) -> Result<Vec<Cuota>, String> {
        use chrono::NaiveDate;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let pattern_prestamo = format!("users:{}:loans:*:cuotas:*", db_access_token);
        let keys_prestamo: Vec<String> = {
            let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
            iter.collect()
        };
        let mut cuotas = Vec::new();
        let today = chrono::Local::now().date_naive();
        for key in keys_prestamo.iter() {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting cuota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            let cuota_vec = from_str::<Vec<Cuota>>(&nested).map_err(|_| "Error deserializing cuota")?;
            if cuota_vec.len() != 1 {
                continue; // Si el array no es de tamaño 1, ignora la cuota
            }
            let cuota = cuota_vec.get(0).cloned();
            if let Some(cuota) = cuota {
                // Filtrado fundamentado:
                // 1. Solo tipo préstamo
                if cuota.tipo != crate::models::graphql::TipoCuota::Prestamo {
                    continue;
                }
                // 2. No pagada
                if cuota.pagada.unwrap_or(false) {
                    continue;
                }
                // 3. Fecha de vencimiento >= hoy
                if let Some(fecha_str) = &cuota.fecha_vencimiento {
                    if let Ok(fecha) = NaiveDate::parse_from_str(fecha_str, "%Y-%m-%d") {
                        if fecha < today {
                            continue;
                        }
                    } else {
                        continue; // Si la fecha no se puede parsear, ignora la cuota
                    }
                } else {
                    continue; // Si no hay fecha, ignora la cuota
                }
                cuotas.push(cuota);
            }
        }
        Ok(cuotas)
    }

        /// Obtiene todas las cuotas asociadas a un loan_id, sin filtrar por estado de pago ni vigencia.
        pub fn get_cuotas_por_loan_id(&self, access_token: String, loan_id: String) -> Result<Vec<Cuota>, String> {
            let db_access_token = hashing_composite_key(&[&access_token]);
            let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
            let pattern_prestamo = format!("users:{}:loans:*:cuotas:*", db_access_token);
            let keys_prestamo: Vec<String> = {
                let iter = con.scan_match::<String, String>(pattern_prestamo).map_err(|_| "Error scanning keys prestamo")?;
                iter.collect()
            };
            let mut cuotas = Vec::new();
            for key in keys_prestamo.iter() {
                let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                    .map_err(|_| format!("Error getting cuota for key {}", key))?;
                let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
                let cuota_vec = from_str::<Vec<Cuota>>(&nested).map_err(|_| "Error deserializing cuota")?;
                if cuota_vec.len() != 1 {
                    continue; // Si el array no es de tamaño 1, ignora la cuota
                }
                let cuota = cuota_vec.get(0).cloned();
                if let Some(cuota) = cuota {
                    // Filtrado fundamentado: solo por loan_id
                    if let Some(ref cuota_loan_id) = cuota.loan_id {
                        if cuota_loan_id == &loan_id {
                            cuotas.push(cuota);
                        }
                    }
                }
            }
            Ok(cuotas)
        }
}
