//! Tests exhaustivos para cuotas de préstamo pendientes (SCRUM-255)
// Cada test está fundamentado en los requisitos, datos reales y errores previos.

use redis::Commands;
use general_api::models::graphql::{Cuota, TipoCuota};
use general_api::repos::graphql::cuota::CuotaRepo;
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::endpoints::handlers::configs::connection_pool::get_pool_connection;
use chrono::Local;
use dotenv::dotenv;

// Helper para limpiar Redis antes/después de cada test
fn cleanup_redis_for_user(access_token: &str) {
    dotenv().ok(); // Cargar variables de entorno
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let mut con = get_pool_connection().into_inner().get().expect("No se pudo obtener conexión");
    let pattern = format!("users:{}:*", db_access_token);
    let keys: Vec<String> = con.scan_match::<String, String>(pattern).expect("Error escaneando claves").collect();
    for key in keys {
        let _ = con.del::<_, ()>(key);
    }
}

#[test]
fn test_cuotas_prestamo_no_pagadas_vigentes_aparecen() {
    // Fundamento: Solo cuotas de préstamo no pagadas y vigentes deben aparecer
    dotenv().ok(); // Cargar variables de entorno
    let pool = get_pool_connection();
    let access_token = "TEST_TOKEN_1";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let cuota = Cuota {
        user_id: "user1".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loan1".to_string()),
        pagada: Some(false),
        extraordinaria: None,
    };
    let repo = CuotaRepo { pool: pool.clone() };
    repo.save_cuota(access_token.to_string(), &cuota).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 1, "Debe retornar una cuota");
    let returned = &result[0];
    assert_eq!(returned.user_id, cuota.user_id);
    assert_eq!(returned.monto, cuota.monto);
    assert_eq!(returned.fecha_vencimiento, cuota.fecha_vencimiento);
    assert_eq!(returned.tipo, TipoCuota::Prestamo);
    assert_eq!(returned.pagada, Some(false));
    cleanup_redis_for_user(access_token);
}

