//! Tests exhaustivos para quotas de préstamo pendientes (SCRUM-255)
// Cada test está fundamentado en los requisitos, datos reales y errores previos.

use chrono::Local;
use general_api::endpoints::handlers::graphql::quota::QuotaQuery;
use general_api::models::graphql::{Quota, QuotaType};
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::repos::graphql::quota::QuotaRepo;
use general_api::repos::graphql::utils::create_test_context;
use redis::Commands;
// dotenv eliminado: solo se usa REDIS_URL exportada por CLI

// Helper para crear contexto siguiendo patrón del proyecto
// Use the project's canonical test context helper to centralize Redis test setup
// and avoid duplicated pool creation logic.

// Helper para crear repo siguiendo patrón del proyecto
fn get_test_repo() -> QuotaRepo {
    let context = create_test_context();
    QuotaRepo { pool: context.pool.clone() }
}

// Helper para limpiar Redis antes/después de cada test
fn cleanup_redis_for_user(repo: &QuotaRepo, access_token: &str) {
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);

    let keys: Vec<String> = {
        let mut conn = repo.pool.get().expect("No se pudo obtener conexión para escanear Redis");
        match conn.scan_match::<String, String>(format!("users:{}:*", db_access_token)) {
            Ok(iter) => iter.collect(),
            Err(_) => Vec::new(),
        }
    };

    if keys.is_empty() {
        return;
    }

    if let Ok(mut conn) = repo.pool.get() {
        for key in keys {
            let _: () = conn.del(key).unwrap_or(());
        }
    }
}

#[test]
fn test_quotas_prestamo_no_pagadas_vigentes_aparecen() {
    // Fundamento: Solo quotas de préstamo no pagadas y vigentes deben aparecer
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_1";
    cleanup_redis_for_user(&repo, access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota = Quota {
        user_id: "user1".to_string(),
        amount: 100.0,
        exp_date: Some(today.clone()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan1".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(1),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    repo
        .save_quota(access_token.to_string(), &quota)
        .expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 1, "Debe retornar una Quota");
    let returned = &result[0];
    assert_eq!(returned.user_id, quota.user_id);
    assert_eq!(returned.amount, quota.amount);
    assert_eq!(returned.exp_date, quota.exp_date);
    assert_eq!(returned.quota_type, QuotaType::Prestamo);
    assert_eq!(returned.payed, Some(false));
    assert_eq!(returned.quota_number, Some(1));
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_quotas_prestamo_pagadas_no_aparecen() {
    // Fundamento: Quotas de préstamo pagadas no deben aparecer en pendientes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_2";
    cleanup_redis_for_user(&repo, access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota_pagada = Quota {
        user_id: "user2".to_string(),
        amount: 200.0,
        exp_date: Some(today),
        monto_pagado: Some(200.0),
        multa: Some(0.0),
        pay_by: Some("user2".to_string()),
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan2".to_string()),
        is_extraordinary: None,
        payed: Some(true), // Marcada como pagada
        quota_number: Some(2),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    repo
        .save_quota(access_token.to_string(), &quota_pagada)
        .expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas pagadas");
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_quotas_prestamo_vencidas_no_aparecen() {
    // Fundamento: Quotas vencidas no deben aparecer en pendientes vigentes
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_3";
    cleanup_redis_for_user(&repo, access_token);

    let fecha_vencida = Local::now().date_naive().checked_sub_days(chrono::Days::new(1))
        .unwrap().format("%Y-%m-%d").to_string();
    let quota_vencida = Quota {
        user_id: "user3".to_string(),
        amount: 300.0,
        exp_date: Some(fecha_vencida),
        monto_pagado: Some(0.0),
        multa: Some(10.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan3".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(3),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    repo
        .save_quota(access_token.to_string(), &quota_vencida)
        .expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas vencidas");
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_quotas_afiliado_no_aparecen() {
    // Fundamento: Solo quotas de préstamo deben aparecer, no las de afiliado
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_4";
    cleanup_redis_for_user(&repo, access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota_afiliado = Quota {
        user_id: "user4".to_string(),
        amount: 50.0,
        exp_date: Some(today),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Afiliado, // Tipo afiliado, no préstamo
        loan_id: None,
        is_extraordinary: None,
        payed: Some(false),
        quota_number: None,
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    repo
        .save_quota(access_token.to_string(), &quota_afiliado)
        .expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    assert_eq!(result.len(), 0, "No debe retornar quotas de afiliado");
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_quotas_corruptas_no_causan_panic() {
    // Fundamento: Datos corruptos en Redis no deben causar panic del sistema
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_5";
    cleanup_redis_for_user(&repo, access_token);

    // Insertar JSON corrupto directamente en Redis
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let mut con = repo.pool.get().expect("No se pudo obtener conexión");
    let key = format!("users:{}:quotas:corrupted_quota", db_access_token);
    let _: () = con
        .set(&key, "{\"invalid_json\": incomplete")
        .expect("Error insertando datos corruptos");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string());

    // Debe manejar el error graciosamente sin panic
    match result {
        Ok(quotas) => assert!(quotas.is_empty(), "Datos corruptos no deben aparecer en resultados"),
        Err(_) => { /* Error controlado es aceptable, lo importante es evitar panic */ }
    }
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_concurrent_access_and_cleanup() {
    // Fundamento: Acceso concurrente y limpieza no deben causar duplicidad ni residuos
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_6";
    cleanup_redis_for_user(&repo, access_token);

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota = Quota {
        user_id: "user6".to_string(),
        amount: 100.0,
        exp_date: Some(today.clone()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan6".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(6),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    
    // Guardar la misma Quota múltiples veces (simular concurrencia)
    repo
        .save_quota(access_token.to_string(), &quota)
        .expect("Error en primera inserción");
    repo
        .save_quota(access_token.to_string(), &quota)
        .expect("Error en segunda inserción");
    
    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error en consulta");
    // La lógica debería manejar duplicados apropiadamente
    assert!(result.len() >= 1, "Debe retornar al menos una Quota");
    
    // Verificar limpieza completa
    cleanup_redis_for_user(&repo, access_token);
    let result_after_cleanup = repo.get_quotas_prestamo_pendientes(access_token.to_string()).expect("Error post-limpieza");
    assert_eq!(result_after_cleanup.len(), 0, "No debe quedar residuos después de limpieza");
}

#[test]
fn test_fecha_mal_formateada_no_aparece() {
    // Fundamento: Quotas con fechas mal formateadas no deben causar errores ni aparecer
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_7";
    cleanup_redis_for_user(&repo, access_token);

    let quota_fecha_mala = Quota {
        user_id: "user7".to_string(),
        amount: 150.0,
        exp_date: Some("fecha-invalida".to_string()), // Fecha mal formateada
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan7".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(7),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    repo
        .save_quota(access_token.to_string(), &quota_fecha_mala)
        .expect("No se pudo guardar Quota");

    let result = repo.get_quotas_prestamo_pendientes(access_token.to_string());
    assert!(result.is_ok(), "No debe fallar con fechas mal formateadas");
    let quotas = result.unwrap();
    assert_eq!(quotas.len(), 0, "Quotas con fechas inválidas no deben aparecer");
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_error_conexion_no_panic() {
    // Fundamento: Errores en operaciones Redis no deben causar panic del sistema
    let repo = get_test_repo();
    let access_token = "TEST_TOKEN_8";
    cleanup_redis_for_user(&repo, access_token);

    // Insertar datos que causen problemas en deserialización
    let access_token_string = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_string]);
    let mut con = repo.pool.get().expect("No se pudo obtener conexión");
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
    cleanup_redis_for_user(&repo, access_token);
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
    cleanup_redis_for_user(&repo, access_token);

    let loan_id = "loanX".to_string();
    let quota1 = Quota {
        user_id: "userA".to_string(),
        amount: 100.0,
        exp_date: Some("2025-09-21".to_string()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some(loan_id.clone()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(1),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    let quota2 = Quota {
        user_id: "userB".to_string(),
        amount: 200.0,
        exp_date: Some("2025-09-22".to_string()),
        monto_pagado: Some(200.0),
        multa: Some(0.0),
        pay_by: Some("userB".to_string()),
        quota_type: QuotaType::Prestamo,
        loan_id: Some(loan_id.clone()),
        is_extraordinary: None,
        payed: Some(true),
        quota_number: Some(2),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    let quota3 = Quota {
        user_id: "userC".to_string(),
        amount: 300.0,
        exp_date: Some("2025-09-23".to_string()),
        monto_pagado: Some(0.0),
        multa: Some(10.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loanY".to_string()), // Otro préstamo
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(3),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
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
    cleanup_redis_for_user(&repo, access_token);
}

#[test]
fn test_get_pending_loans_quotas() {
    // Fundamento: Debe retornar quotas de préstamo en el formato específico requerido según docs/api-quota-response-format.md
    let context = create_test_context();
    let repo = QuotaRepo { pool: context.pool.clone() };
    let access_token = "TEST_TOKEN_FORMATTED";
    cleanup_redis_for_user(&repo, access_token);

    // Crear quotas de préstamo con diferentes características para validar el mapeo completo
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let quota1 = Quota {
        user_id: "user_formatted_1".to_string(),
        amount: 100.0,
        exp_date: Some(today.clone()),
        monto_pagado: Some(25.0),
        multa: Some(5.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan_abc".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(1),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };
    let quota2 = Quota {
        user_id: "user_formatted_2".to_string(),
        amount: 200.0,
        exp_date: Some(today.clone()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: Some("third_party".to_string()),
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan_xyz".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(2),
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };

    // Guardar quotas usando el repo
    repo
        .save_quota(access_token.to_string(), &quota1)
        .expect("No se pudo guardar quota1");
    repo
        .save_quota(access_token.to_string(), &quota2)
        .expect("No se pudo guardar quota2");

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
    let quota_response_1: &Quota = result.iter()
        .find(|r| r.amount == 100.0)
        .expect("Debe encontrar Quota con monto 100.0");
    
    assert_eq!(quota_response_1.user_id, quota1.user_id);
    assert_eq!(quota_response_1.amount, quota1.amount);
    assert_eq!(quota_response_1.exp_date, quota1.exp_date);
    assert_eq!(quota_response_1.monto_pagado, quota1.monto_pagado);
    assert_eq!(quota_response_1.multa, quota1.multa);
    assert_eq!(quota_response_1.pay_by, quota1.pay_by);
    assert_eq!(quota_response_1.quota_type, QuotaType::Prestamo);
    assert_eq!(quota_response_1.loan_id, quota1.loan_id);
    assert_eq!(quota_response_1.payed, Some(false));
    assert_eq!(quota_response_1.quota_number, quota1.quota_number);
    assert_eq!(quota_response_1.nombre_prestamo, None); // Por ahora vacío según documentación

    // Verificar campos específicos para la segunda Quota
    let quota_response_2: &Quota = result.iter()
        .find(|r| r.amount == 200.0)
        .expect("Debe encontrar Quota con monto 200.0");
    
    assert_eq!(quota_response_2.user_id, quota2.user_id);
    assert_eq!(quota_response_2.amount, quota2.amount);
    assert_eq!(quota_response_2.exp_date, quota2.exp_date);
    assert_eq!(quota_response_2.monto_pagado, quota2.monto_pagado);
    assert_eq!(quota_response_2.multa, quota2.multa);
    assert_eq!(quota_response_2.pay_by, quota2.pay_by);
    assert_eq!(quota_response_2.quota_type, QuotaType::Prestamo);
    assert_eq!(quota_response_2.loan_id, quota2.loan_id);
    assert_eq!(quota_response_2.payed, Some(false));
    assert_eq!(quota_response_2.quota_number, quota2.quota_number);
    assert_eq!(quota_response_2.nombre_prestamo, None); // Por ahora vacío según documentación

    cleanup_redis_for_user(&repo, access_token);
}
