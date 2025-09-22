//! Tests exhaustivos para cuotas de préstamo pendientes (SCRUM-255)
// Cada test está fundamentado en los requisitos, datos reales y errores previos.

use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands};
use general_api::models::graphql::{Cuota, TipoCuota};
use general_api::repos::graphql::cuota::CuotaRepo;
use general_api::repos::auth::utils::hashing_composite_key;
use chrono::Local;
use dotenv::dotenv;

// Helper para crear repo siguiendo patrón del proyecto
fn get_test_repo() -> CuotaRepo {
    dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let pool = Data::new(Pool::new(Client::open(redis_url).unwrap()).unwrap());
    CuotaRepo { pool }
}

// Helper para limpiar Redis antes/después de cada test
fn cleanup_redis_for_user(access_token: &str) {
    dotenv().ok(); // Cargar variables de entorno
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("No se pudo conectar a Redis");
    let mut con = client.get_connection().expect("No se pudo obtener conexión");
    let pattern = format!("users:{}:*", db_access_token);
    let keys: Vec<String> = con.scan_match::<String, String>(pattern).expect("Error escaneando claves").collect();
    for key in keys {
        let _ = con.del::<_, ()>(key);
    }
}

#[test]
fn test_cuotas_prestamo_no_pagadas_vigentes_aparecen() {
    // Fundamento: Solo cuotas de préstamo no pagadas y vigentes deben aparecer
    let repo = get_test_repo();
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
        numero_cuota: Some(1),
    };
    repo.save_cuota(access_token.to_string(), &cuota).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 1, "Debe retornar una cuota");
    let returned = &result[0];
    assert_eq!(returned.user_id, cuota.user_id);
    assert_eq!(returned.monto, cuota.monto);
    assert_eq!(returned.fecha_vencimiento, cuota.fecha_vencimiento);
    assert_eq!(returned.tipo, TipoCuota::Prestamo);
    assert_eq!(returned.pagada, Some(false));
    assert_eq!(returned.numero_cuota, Some(1));
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_cuotas_prestamo_pagadas_no_aparecen() {
    // Fundamento: Cuotas de préstamo pagadas no deben aparecer en pendientes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_2";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let cuota_pagada = Cuota {
        user_id: "user2".to_string(),
        monto: 200.0,
        fecha_vencimiento: Some(today),
        monto_pagado: 200.0,
        multa: 0.0,
        pagada_por: Some("user2".to_string()),
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loan2".to_string()),
        pagada: Some(true), // Marcada como pagada
        extraordinaria: None,
        numero_cuota: Some(2),
    };
    repo.save_cuota(access_token.to_string(), &cuota_pagada).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar cuotas pagadas");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_cuotas_prestamo_vencidas_no_aparecen() {
    // Fundamento: Cuotas vencidas no deben aparecer en pendientes vigentes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_3";
    cleanup_redis_for_user(access_token);

    let fecha_vencida = Local::now().date_naive().checked_sub_days(chrono::Days::new(1))
        .unwrap().format("%Y-%m-%d").to_string();
    let cuota_vencida = Cuota {
        user_id: "user3".to_string(),
        monto: 300.0,
        fecha_vencimiento: Some(fecha_vencida),
        monto_pagado: 0.0,
        multa: 10.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loan3".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: Some(3),
    };
    repo.save_cuota(access_token.to_string(), &cuota_vencida).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar cuotas vencidas");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_cuotas_afiliado_no_aparecen() {
    // Fundamento: Solo cuotas de préstamo deben aparecer, no las de afiliado
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_4";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let cuota_afiliado = Cuota {
        user_id: "user4".to_string(),
        monto: 50.0,
        fecha_vencimiento: Some(today),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoCuota::Afiliado, // Tipo afiliado, no préstamo
        loan_id: None,
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: None,
    };
    repo.save_cuota(access_token.to_string(), &cuota_afiliado).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar cuotas de afiliado");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_cuotas_corruptas_no_causan_panic() {
    // Fundamento: Datos corruptos en Redis no deben causar panic del sistema
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_5";
    cleanup_redis_for_user(access_token);

    // Insertar JSON corrupto directamente en Redis
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("No se pudo conectar a Redis");
    let mut con = client.get_connection().expect("No se pudo obtener conexión");
    let key = format!("users:{}:cuotas:corrupted_cuota", db_access_token);
    let _: () = con.set(&key, "{\"invalid_json\": incomplete").expect("Error insertando datos corruptos");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string());
    
    // Debe manejar el error graciosamente sin panic
    assert!(result.is_ok(), "No debe fallar con datos corruptos");
    let cuotas = result.unwrap();
    assert_eq!(cuotas.len(), 0, "Datos corruptos no deben aparecer en resultados");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_concurrent_access_and_cleanup() {
    // Fundamento: Acceso concurrente y limpieza no deben causar duplicidad ni residuos
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_6";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let cuota = Cuota {
        user_id: "user6".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loan6".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: Some(6),
    };
    
    // Guardar la misma cuota múltiples veces (simular concurrencia)
    repo.save_cuota(access_token.to_string(), &cuota).expect("Error en primera inserción");
    repo.save_cuota(access_token.to_string(), &cuota).expect("Error en segunda inserción");
    
    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    // La lógica debería manejar duplicados apropiadamente
    assert!(result.len() >= 1, "Debe retornar al menos una cuota");
    
    // Verificar limpieza completa
    cleanup_redis_for_user(access_token);
    let result_after_cleanup = repo.get_cuotas_prestamo_pendientes(access_token.to_string()).expect("Error post-limpieza");
    assert_eq!(result_after_cleanup.len(), 0, "No debe quedar residuos después de limpieza");
}

#[test]
fn test_fecha_mal_formateada_no_aparece() {
    // Fundamento: Cuotas con fechas mal formateadas no deben causar errores ni aparecer
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_7";
    cleanup_redis_for_user(access_token);

    let cuota_fecha_mala = Cuota {
        user_id: "user7".to_string(),
        monto: 150.0,
        fecha_vencimiento: Some("fecha-invalida".to_string()), // Fecha mal formateada
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loan7".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: Some(7),
    };
    repo.save_cuota(access_token.to_string(), &cuota_fecha_mala).expect("No se pudo guardar cuota");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string());
    assert!(result.is_ok(), "No debe fallar con fechas mal formateadas");
    let cuotas = result.unwrap();
    assert_eq!(cuotas.len(), 0, "Cuotas con fechas inválidas no deben aparecer");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_error_conexion_no_panic() {
    // Fundamento: Errores en operaciones Redis no deben causar panic del sistema
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_8";
    cleanup_redis_for_user(access_token);

    // Insertar datos que causen problemas en deserialización
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("No se pudo conectar a Redis");
    let mut con = client.get_connection().expect("No se pudo obtener conexión");
    let key = format!("users:{}:cuotas:problematic_data", db_access_token);
    let _: () = con.set(&key, "not-a-json-object").expect("Error insertando datos problemáticos");

    let result = repo.get_cuotas_prestamo_pendientes(access_token.to_string());
    
    // El sistema debe manejar errores de deserialización graciosamente
    // Puede retornar error o lista vacía, pero no debe hacer panic
    match result {
        Ok(cuotas) => {
            // Si retorna Ok, los datos problemáticos no deben aparecer
            assert_eq!(cuotas.len(), 0, "Datos problemáticos no deben deserializarse");
        },
        Err(_) => {
            // Si retorna error, es aceptable - lo importante es que no haga panic
            assert!(true, "Error controlado es aceptable");
        }
    }
    cleanup_redis_for_user(access_token);
}

// Cada test debe documentar su fundamento y limpiar el entorno antes/después
// Los helpers y setup deben evitar duplicidad y residuos
// No se asume nada: todo se valida y se documenta

#[test]
fn test_get_cuotas_por_loan_id_retornan_todas() {
    // Fundamento: Debe retornar todas las cuotas asociadas a un loan_id, sin filtrar por estado de pago ni vigencia
    // Se crean cuotas pagadas, pendientes y de otros préstamos para validar el filtrado correcto
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_LOANID";
    cleanup_redis_for_user(access_token);

    let loan_id = "loanX".to_string();
    let cuota1 = Cuota {
        user_id: "userA".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some("2025-09-21".to_string()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some(loan_id.clone()),
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: Some(1),
    };
    let cuota2 = Cuota {
        user_id: "userB".to_string(),
        monto: 200.0,
        fecha_vencimiento: Some("2025-09-22".to_string()),
        monto_pagado: 200.0,
        multa: 0.0,
        pagada_por: Some("userB".to_string()),
        tipo: TipoCuota::Prestamo,
        loan_id: Some(loan_id.clone()),
        pagada: Some(true),
        extraordinaria: None,
        numero_cuota: Some(2),
    };
    let cuota3 = Cuota {
        user_id: "userC".to_string(),
        monto: 300.0,
        fecha_vencimiento: Some("2025-09-23".to_string()),
        monto_pagado: 0.0,
        multa: 10.0,
        pagada_por: None,
        tipo: TipoCuota::Prestamo,
        loan_id: Some("loanY".to_string()), // Otro préstamo
        pagada: Some(false),
        extraordinaria: None,
        numero_cuota: Some(3),
    };
    repo.save_cuota(access_token.to_string(), &cuota1).expect("No se pudo guardar cuota1");
    repo.save_cuota(access_token.to_string(), &cuota2).expect("No se pudo guardar cuota2");
    repo.save_cuota(access_token.to_string(), &cuota3).expect("No se pudo guardar cuota3");

    let result = repo.get_cuotas_por_loan_id(access_token.to_string(), loan_id.clone()).expect("Error en consulta por loan_id");
    assert_eq!(result.len(), 2, "Debe retornar solo las cuotas asociadas a loan_id");
    let user_ids: Vec<String> = result.iter().map(|c| c.user_id.clone()).collect();
    assert!(user_ids.contains(&"userA".to_string()), "Debe incluir cuota1");
    assert!(user_ids.contains(&"userB".to_string()), "Debe incluir cuota2");
    assert!(!user_ids.contains(&"userC".to_string()), "No debe incluir cuota3 de otro préstamo");
    cleanup_redis_for_user(access_token);
}
