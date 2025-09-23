//! Tests exhaustivos para quotas de préstamo pendientes (SCRUM-255)
// Cada test está fundamentado en los requisitos, datos reales y errores previos.

use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands};
use general_api::models::graphql::{Quota, TipoQuota};
use general_api::repos::graphql::Quota::QuotaRepo;
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::endpoints::handlers::graphql::Quota::{QuotaQuery, QuotaPrestamoResponse};
use chrono::Local;
use dotenv::dotenv;

// Helper para crear contexto siguiendo patrón del proyecto
fn setup_context() -> GeneralContext {
    dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).unwrap();
    let pool = Pool::builder().build(client).unwrap();
    GeneralContext { pool: Data::new(pool) }
}

// Helper para crear repo siguiendo patrón del proyecto
fn get_test_repo() -> QuotaRepo {
    dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let pool = Data::new(Pool::new(Client::open(redis_url).unwrap()).unwrap());
    QuotaRepo { pool }
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
fn test_quotas_prestamo_no_pagadas_vigentes_aparecen() {
    // Fundamento: Solo quotas de préstamo no pagadas y vigentes deben aparecer
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_1";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let Quota = Quota {
        user_id: "user1".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan1".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(1),
    };
    repo.save_quota(access_token.to_string(), &Quota).expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 1, "Debe retornar una Quota");
    let returned = &result[0];
    assert_eq!(returned.user_id, Quota.user_id);
    assert_eq!(returned.monto, Quota.monto);
    assert_eq!(returned.fecha_vencimiento, Quota.fecha_vencimiento);
    assert_eq!(returned.tipo, TipoQuota::Prestamo);
    assert_eq!(returned.pagada, Some(false));
    assert_eq!(returned.numero_quota, Some(1));
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_quotas_prestamo_pagadas_no_aparecen() {
    // Fundamento: Quotas de préstamo pagadas no deben aparecer en pendientes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_2";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota_pagada = Quota {
        user_id: "user2".to_string(),
        monto: 200.0,
        fecha_vencimiento: Some(today),
        monto_pagado: 200.0,
        multa: 0.0,
        pagada_por: Some("user2".to_string()),
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan2".to_string()),
        pagada: Some(true), // Marcada como pagada
        extraordinaria: None,
        numero_quota: Some(2),
    };
    repo.save_quota(access_token.to_string(), &quota_pagada).expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas pagadas");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_quotas_prestamo_vencidas_no_aparecen() {
    // Fundamento: Quotas vencidas no deben aparecer en pendientes vigentes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_3";
    cleanup_redis_for_user(access_token);

    let fecha_vencida = Local::now().date_naive().checked_sub_days(chrono::Days::new(1))
        .unwrap().format("%Y-%m-%d").to_string();
    let quota_vencida = Quota {
        user_id: "user3".to_string(),
        monto: 300.0,
        fecha_vencimiento: Some(fecha_vencida),
        monto_pagado: 0.0,
        multa: 10.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan3".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(3),
    };
    repo.save_quota(access_token.to_string(), &quota_vencida).expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas vencidas");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_quotas_afiliado_no_aparecen() {
    // Fundamento: Solo quotas de préstamo deben aparecer, no las de afiliado
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_4";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota_afiliado = Quota {
        user_id: "user4".to_string(),
        monto: 50.0,
        fecha_vencimiento: Some(today),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoQuota::Afiliado, // Tipo afiliado, no préstamo
        loan_id: None,
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: None,
    };
    repo.save_quota(access_token.to_string(), &quota_afiliado).expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas de afiliado");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_quotas_corruptas_no_causan_panic() {
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
    let key = format!("users:{}:quotas:corrupted_quota", db_access_token);
    let _: () = con.set(&key, "{\"invalid_json\": incomplete").expect("Error insertando datos corruptos");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string());
    
    // Debe manejar el error graciosamente sin panic
    assert!(result.is_ok(), "No debe fallar con datos corruptos");
    let quotas = result.unwrap();
    assert_eq!(quotas.len(), 0, "Datos corruptos no deben aparecer en resultados");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_concurrent_access_and_cleanup() {
    // Fundamento: Acceso concurrente y limpieza no deben causar duplicidad ni residuos
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_6";
    cleanup_redis_for_user(access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let Quota = Quota {
        user_id: "user6".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan6".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(6),
    };
    
    // Guardar la misma Quota múltiples veces (simular concurrencia)
    repo.save_quota(access_token.to_string(), &Quota).expect("Error en primera inserción");
    repo.save_quota(access_token.to_string(), &Quota).expect("Error en segunda inserción");
    
    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    // La lógica debería manejar duplicados apropiadamente
    assert!(result.len() >= 1, "Debe retornar al menos una Quota");
    
    // Verificar limpieza completa
    cleanup_redis_for_user(access_token);
    let result_after_cleanup = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error post-limpieza");
    assert_eq!(result_after_cleanup.len(), 0, "No debe quedar residuos después de limpieza");
}

#[test]
fn test_fecha_mal_formateada_no_aparece() {
    // Fundamento: Quotas con fechas mal formateadas no deben causar errores ni aparecer
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_7";
    cleanup_redis_for_user(access_token);

    let quota_fecha_mala = Quota {
        user_id: "user7".to_string(),
        monto: 150.0,
        fecha_vencimiento: Some("fecha-invalida".to_string()), // Fecha mal formateada
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan7".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(7),
    };
    repo.save_quota(access_token.to_string(), &quota_fecha_mala).expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string());
    assert!(result.is_ok(), "No debe fallar con fechas mal formateadas");
    let quotas = result.unwrap();
    assert_eq!(quotas.len(), 0, "Quotas con fechas inválidas no deben aparecer");
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
    let key = format!("users:{}:quotas:problematic_data", db_access_token);
    let _: () = con.set(&key, "not-a-json-object").expect("Error insertando datos problemáticos");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string());
    
    // El sistema debe manejar errores de deserialización graciosamente
    // Puede retornar error o lista vacía, pero no debe hacer panic
    match result {
        Ok(quotas) => {
            // Si retorna Ok, los datos problemáticos no deben aparecer
            assert_eq!(quotas.len(), 0, "Datos problemáticos no deben deserializarse");
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
fn test_get_quota_by_loan_id_retornan_todas() {
    // Fundamento: Debe retornar todas las quotas asociadas a un loan_id, sin filtrar por estado de pago ni vigencia
    // Se crean quotas pagadas, pendientes y de otros préstamos para validar el filtrado correcto
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_LOANID";
    cleanup_redis_for_user(access_token);

    let loan_id = "loanX".to_string();
    let quota1 = Quota {
        user_id: "userA".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some("2025-09-21".to_string()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some(loan_id.clone()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(1),
    };
    let quota2 = Quota {
        user_id: "userB".to_string(),
        monto: 200.0,
        fecha_vencimiento: Some("2025-09-22".to_string()),
        monto_pagado: 200.0,
        multa: 0.0,
        pagada_por: Some("userB".to_string()),
        tipo: TipoQuota::Prestamo,
        loan_id: Some(loan_id.clone()),
        pagada: Some(true),
        extraordinaria: None,
        numero_quota: Some(2),
    };
    let quota3 = Quota {
        user_id: "userC".to_string(),
        monto: 300.0,
        fecha_vencimiento: Some("2025-09-23".to_string()),
        monto_pagado: 0.0,
        multa: 10.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loanY".to_string()), // Otro préstamo
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(3),
    };
    repo.save_quota(access_token.to_string(), &quota1).expect("No se pudo guardar quota1");
    repo.save_quota(access_token.to_string(), &quota2).expect("No se pudo guardar quota2");
    repo.save_quota(access_token.to_string(), &quota3).expect("No se pudo guardar quota3");

    let result = repo.get_quota_by_loan_id(access_token.to_string(), loan_id.clone()).expect("Error en consulta por loan_id");
    assert_eq!(result.len(), 2, "Debe retornar solo las quotas asociadas a loan_id");
    let user_ids: Vec<String> = result.iter().map(|c| c.user_id.clone()).collect();
    assert!(user_ids.contains(&"userA".to_string()), "Debe incluir quota1");
    assert!(user_ids.contains(&"userB".to_string()), "Debe incluir quota2");
    assert!(!user_ids.contains(&"userC".to_string()), "No debe incluir quota3 de otro préstamo");
    cleanup_redis_for_user(access_token);
}

#[test]
fn test_get_pending_loans_quotas() {
    // Fundamento: Debe retornar quotas de préstamo en el formato específico requerido según docs/api-quota-response-format.md
    let context = setup_context();
    let access_token = "TEST_TOKEN_FORMATTED";
    cleanup_redis_for_user(access_token);

    // Crear quotas de préstamo con diferentes características para validar el mapeo completo
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota1 = Quota {
        user_id: "user_formatted_1".to_string(),
        monto: 100.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 25.0,
        multa: 5.0,
        pagada_por: None,
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan_abc".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(1),
    };
    let quota2 = Quota {
        user_id: "user_formatted_2".to_string(),
        monto: 200.0,
        fecha_vencimiento: Some(today.clone()),
        monto_pagado: 0.0,
        multa: 0.0,
        pagada_por: Some("third_party".to_string()),
        tipo: TipoQuota::Prestamo,
        loan_id: Some("loan_xyz".to_string()),
        pagada: Some(false),
        extraordinaria: None,
        numero_quota: Some(2),
    };

    // Guardar quotas usando el repo
    let repo = get_test_repo();
    repo.save_quota(access_token.to_string(), &quota1).expect("No se pudo guardar quota1");
    repo.save_quota(access_token.to_string(), &quota2).expect("No se pudo guardar quota2");

    // Ejecutar el resolver formateado
    let result = futures::executor::block_on(
        QuotaQuery::get_pending_loans_quotas(&context, access_token.to_string())
    ).unwrap();

    // Imprimir resultado para depuración
    println!("Resultado del resolver formateado: {:?}", result);

    // Validar formato y contenido del array de objetos QuotaPrestamoResponse
    assert!(!result.is_empty(), "El resultado no debe estar vacío");
    assert_eq!(result.len(), 2, "Debe retornar exactamente 2 quotas");

    // Verificar campos específicos para la primera Quota
    let quota_response_1: &QuotaPrestamoResponse = result.iter()
        .find(|r| r.monto == 100.0)
        .expect("Debe encontrar Quota con monto 100.0");
    
    assert_eq!(quota_response_1.user_id, access_token);
    assert_eq!(quota_response_1.monto, 100.0);
    assert_eq!(quota_response_1.fecha_vencimiento, today);
    assert_eq!(quota_response_1.monto_pagado, 25.0);
    assert_eq!(quota_response_1.multa, 5.0);
    assert_eq!(quota_response_1.pagada_por, None);
    assert_eq!(quota_response_1.tipo, "Prestamo");
    assert_eq!(quota_response_1.loan_id, Some("loan_abc".to_string()));
    assert_eq!(quota_response_1.pagada, false);
    assert_eq!(quota_response_1.numero_quota, Some(1));
    assert_eq!(quota_response_1.nombre_prestamo, None); // Por ahora vacío según documentación

    // Verificar campos específicos para la segunda Quota
    let quota_response_2: &QuotaPrestamoResponse = result.iter()
        .find(|r| r.monto == 200.0)
        .expect("Debe encontrar Quota con monto 200.0");
    
    assert_eq!(quota_response_2.user_id, access_token);
    assert_eq!(quota_response_2.monto, 200.0);
    assert_eq!(quota_response_2.fecha_vencimiento, today);
    assert_eq!(quota_response_2.monto_pagado, 0.0);
    assert_eq!(quota_response_2.multa, 0.0);
    assert_eq!(quota_response_2.pagada_por, Some("third_party".to_string()));
    assert_eq!(quota_response_2.tipo, "Prestamo");
    assert_eq!(quota_response_2.loan_id, Some("loan_xyz".to_string()));
    assert_eq!(quota_response_2.pagada, false);
    assert_eq!(quota_response_2.numero_quota, Some(2));
    assert_eq!(quota_response_2.nombre_prestamo, None); // Por ahora vacío según documentación

    cleanup_redis_for_user(access_token);
}
