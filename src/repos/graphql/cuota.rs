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
    // Guarda una cuota en Redis
    pub fn save_cuota(&self, access_token: String, cuota: &Cuota) -> Result<(), String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let key = match &cuota.tipo {
            crate::models::graphql::TipoCuota::Prestamo => {
                let loan_id = cuota.loan_id.as_deref().ok_or("loan_id es requerido para cuotas de préstamo")?;
                format!("users:{}:loans:{}:cuotas:{}", db_access_token, loan_id, cuota.fecha_vencimiento)
            },
            crate::models::graphql::TipoCuota::Afiliado => {
                format!("users:{}:cuotas_afiliado:{}", db_access_token, cuota.fecha_vencimiento)
            }
        };
        con.json_set::<_, _, _, ()>(key, "$", cuota).map_err(|_| "Error saving cuota")?;
        Ok(())
    }

    // Consulta todas las cuotas de préstamo pendientes para un usuario
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
}
