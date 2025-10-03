use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::models::graphql::{Quota, QuotaType};
use crate::repos::graphql::quota::QuotaRepo;
use super::return_n_dummies;

/// Genera un ID SHA256 basado en un contador para simular IDs únicos como en la documentación
fn generate_sha256_id(counter: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(counter.to_be_bytes());
    format!("{:x}", hasher.finalize())
}

/// Genera una cuota dummy de tipo Afiliado siguiendo las convenciones exactas del código
fn generate_dummy_afiliado_quota_with_token(access_token: String, counter: u32) -> Quota {
    let mut rng = rand::thread_rng();
    
    // Fechas aleatorias de 2025 (meses 1-12, días 1-28)
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28);
    let fecha = format!("2025-{:02}-{:02}", month, day);
    
    Quota {
        // CONVENCIÓN: user_id debe ser igual al access_token (sin hashear)
        user_id: access_token,
        amount: rng.gen_range(100.0..500.0),
        exp_date: Some(fecha),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Afiliado,
        loan_id: None,
        is_extraordinary: Some(false),
        payed: Some(false),
        quota_number: None,
        nombre_prestamo: None,
        nombre_usuario: Some(format!("Usuario Afiliado {}", rng.gen_range(1..100))),
        identifier: None,
    }
}

/// Genera una cuota dummy de tipo Prestamo siguiendo las convenciones exactas del código
fn generate_dummy_prestamo_quota_with_token(access_token: String, counter: u32) -> Quota {
    let mut rng = rand::thread_rng();
    
    // Fechas aleatorias de 2025 (meses 1-12, días 1-28)
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28);
    let fecha = format!("2025-{:02}-{:02}", month, day);
    
    // CONVENCIÓN: loan_id usa SHA256 como en la documentación
    let loan_id = generate_sha256_id(counter + 1000);
    let quota_number = rng.gen_range(1..=24);
    
    Quota {
        // CONVENCIÓN: user_id debe ser igual al access_token (sin hashear)
        user_id: access_token,
        amount: rng.gen_range(500.0..2000.0),
        exp_date: Some(fecha),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some(loan_id.clone()),
        is_extraordinary: Some(false),
        payed: Some(false),
        quota_number: Some(quota_number),
        nombre_prestamo: Some(format!("Préstamo {}", &loan_id[..8])), // Solo primeros 8 chars para legibilidad
        nombre_usuario: Some(format!("Usuario Préstamo {}", rng.gen_range(1..100))),
        identifier: None,
    }
}

/// Inserta datos dummy en Redis usando el access_token especificado
/// Sigue las convenciones exactas: user_id = access_token, loan_id con SHA256
/// - afiliado_count: número de cuotas de afiliado a crear
/// - prestamo_count: número de cuotas de préstamo a crear
pub fn insert_dummy_quotas(
    pool: Data<Pool<Client>>,
    access_token: String,
    afiliado_count: i32,
    prestamo_count: i32,
) -> Result<String, String> {
    let quota_repo = QuotaRepo { pool };
    let mut inserted_count = 0;
    
    // Generar cuotas de afiliado con access_token correcto
    for i in 0..afiliado_count {
        let quota = generate_dummy_afiliado_quota_with_token(access_token.clone(), i as u32);
        match quota_repo.save_quota(access_token.clone(), &quota) {
            Ok(_) => inserted_count += 1,
            Err(e) => return Err(format!("Error insertando cuota afiliado: {}", e)),
        }
    }
    
    // Generar cuotas de préstamo con access_token correcto
    for i in 0..prestamo_count {
        let quota = generate_dummy_prestamo_quota_with_token(access_token.clone(), (i + afiliado_count) as u32);
        match quota_repo.save_quota(access_token.clone(), &quota) {
            Ok(_) => inserted_count += 1,
            Err(e) => return Err(format!("Error insertando cuota préstamo: {}", e)),
        }
    }
    
    Ok(format!(
        "Datos dummy insertados exitosamente siguiendo convenciones: {} cuotas de afiliado + {} cuotas de préstamo = {} total",
        afiliado_count, prestamo_count, inserted_count
    ))
}

/// Función de conveniencia para insertar exactamente 10 afiliado + 10 préstamo
/// con las convenciones correctas (user_id = access_token)
pub fn insert_20_dummy_quotas(
    pool: Data<Pool<Client>>,
    access_token: String,
) -> Result<String, String> {
    insert_dummy_quotas(pool, access_token, 10, 10)
}

/// Función para borrar todos los datos dummy existentes de un access_token
pub fn delete_all_quotas(
    pool: Data<Pool<Client>>,
    access_token: String,
) -> Result<String, String> {
    use redis::{Commands, pipe};
    use crate::repos::auth::utils::hashing_composite_key;
    
    let mut con = pool.get().map_err(|_| "Couldn't connect to pool")?;
    let db_access_token = hashing_composite_key(&[&access_token]);
    
    // Buscar todas las claves de cuotas (tanto afiliado como préstamo)
    let pattern_prestamo = format!("users:{}:loans:*:quotas:*", db_access_token);
    let pattern_afiliado = format!("users:{}:quotas_afiliado:*", db_access_token);
    
    let mut deleted_count = 0;
    
    // Borrar cuotas de préstamo
    let keys_prestamo: Vec<String> = {
        let iter = con
            .scan_match::<String, String>(pattern_prestamo)
            .map_err(|_| "Error scanning keys prestamo")?;
        iter.collect()
    };
    
    for key in keys_prestamo {
        con.del::<String, ()>(key).map_err(|_| "Error deleting prestamo quota")?;
        deleted_count += 1;
    }
    
    // Borrar cuotas de afiliado
    let keys_afiliado: Vec<String> = {
        let iter = con
            .scan_match::<String, String>(pattern_afiliado)
            .map_err(|_| "Error scanning keys afiliado")?;
        iter.collect()
    };
    
    for key in keys_afiliado {
        con.del::<String, ()>(key).map_err(|_| "Error deleting afiliado quota")?;
        deleted_count += 1;
    }
    
    Ok(format!("Borrados {} registros de cuotas exitosamente", deleted_count))
}