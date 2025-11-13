use sha2::{Digest, Sha256};
use redis::{Commands, RedisResult};

use crate::models::StatusMessage;

/// function that giving n reference to arguments, returns the hasked key in string format
pub fn hashing_composite_key(args: &[&String]) -> String {
    //Passes the args formated

    let mut string_acc = String::new();

    for arg in args {
        string_acc = format!("{}{}", &string_acc, arg);
    }

    let hashed_args = Sha256::digest(string_acc);

    //X is for hexadecimal
    format!("{:X}", hashed_args)
}

/// Obtiene el db_composite_key a partir de un username
/// 
/// # Arguments
/// * `user_name` - Username del usuario
/// * `con` - Conexión a Redis
/// 
/// # Returns
/// * `Ok(String)` - El db_composite_key del usuario
/// * `Err(StatusMessage)` - Si el usuario no existe
pub fn get_db_key_from_username(
    user_name: &str,
    con: &mut redis::Connection,
) -> Result<String, StatusMessage> {
    let affiliate_key = hashing_composite_key(&[&user_name.to_string()]);
    con.get(format!("affiliate_key_to_db_access:{}", &affiliate_key))
        .map_err(|_| StatusMessage {
            message: "Usuario no encontrado".to_string(),
        })
}

/// Copia un campo scalar de Redis desde una key antigua a una nueva
/// 
/// # Arguments
/// * `con` - Conexión a Redis
/// * `field_name` - Nombre del campo (ej: "payed_to_capital")
/// * `old_key` - db_composite_key antiguo
/// * `new_key` - db_composite_key nuevo
/// * `default` - Valor por defecto si el campo no existe
/// 
/// # Type Parameters
/// * `T` - Tipo del valor (debe implementar ToRedisArgs + FromRedisValue + Clone)
pub fn copy_redis_field<T>(
    con: &mut redis::Connection,
    field_name: &str,
    old_key: &str,
    new_key: &str,
    default: T,
) -> Result<(), StatusMessage>
where
    T: redis::ToRedisArgs + redis::FromRedisValue + Clone,
{
    let value: T = con
        .get(format!("users:{}:{}", old_key, field_name))
        .unwrap_or(default);
    
    con.set(format!("users:{}:{}", new_key, field_name), value)
        .map_err(|_| StatusMessage {
            message: format!("No se pudo copiar campo: {}", field_name),
        })
}

/// Copia las 3 respuestas de seguridad desde un db_key antiguo a uno nuevo
/// 
/// # Arguments
/// * `con` - Conexión a Redis
/// * `old_key` - db_composite_key antiguo
/// * `new_key` - db_composite_key nuevo
pub fn copy_security_answers(
    con: &mut redis::Connection,
    old_key: &str,
    new_key: &str,
) -> Result<(), StatusMessage> {
    for i in 0..3 {
        if let Ok(answer) = con.get::<String, String>(
            format!("users:{}:security_answer_{}", old_key, i)
        ) {
            if !answer.is_empty() {
                let _: () = con
                    .set(
                        format!("users:{}:security_answer_{}", new_key, i),
                        answer
                    )
                    .map_err(|_| StatusMessage {
                        message: format!("No se pudo copiar respuesta de seguridad {}", i),
                    })?;
            }
        }
    }
    Ok(())
}

/// Elimina todas las claves que coinciden con un patrón
/// 
/// # Arguments
/// * `con` - Conexión a Redis
/// * `pattern` - Patrón de búsqueda (ej: "users:ABC123:payments:*")
pub fn delete_keys_by_pattern(
    con: &mut redis::Connection,
    pattern: String,
) -> RedisResult<()> {
    if let Ok(keys_iter) = con.scan_match::<String, String>(pattern) {
        let keys: Vec<String> = keys_iter.collect();
        for key in keys {
            let _: RedisResult<()> = con.del(&key);
        }
    }
    Ok(())
}

// TODO: do function for matching payment end value with Payment Object
