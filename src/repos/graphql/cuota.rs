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
            crate::models::graphql::TipoCuota::Prestamo { loan_id } => {
                format!("users:{}:loans:{}:cuotas:{}", db_access_token, loan_id, cuota.fecha_vencimiento)
            },
            crate::models::graphql::TipoCuota::Afiliado { .. } => {
                format!("users:{}:cuotas_afiliado:{}", db_access_token, cuota.fecha_vencimiento)
            }
        };
        let cuota_json = serde_json::to_string(&cuota).map_err(|_| "Serialization error")?;
        con.json_set(key, "$", &cuota_json).map_err(|_| "Error saving cuota")?;
        Ok(())
    }

    // Consulta todas las cuotas de préstamo pendientes para un usuario
    pub fn get_cuotas_pendientes(&self, access_token: String) -> Result<Vec<Cuota>, String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;
        let db_access_token = hashing_composite_key(&[&access_token]);
        let pattern = format!("users:{}:loans:*:cuotas:*", db_access_token);
        let keys = con.scan_match::<String, String>(pattern).map_err(|_| "Error scanning keys")?;
        let mut cuotas = Vec::new();
        for key in keys {
            let raw = con.json_get::<String, &str, RedisValue>(key.clone(), "$")
                .map_err(|_| format!("Error getting cuota for key {}", key))?;
            let nested = from_redis_value::<String>(&raw).map_err(|_| "Error parsing redis value")?;
            let cuota_vec = from_str::<Vec<Cuota>>(&nested).map_err(|_| "Error deserializing cuota")?;
            let cuota = cuota_vec.get(0).cloned();
            if let Some(cuota) = cuota {
                //  lógica para filtrar por fecha y estado de pago
                cuotas.push(cuota);
            }
        }
        Ok(cuotas)
    }
}
