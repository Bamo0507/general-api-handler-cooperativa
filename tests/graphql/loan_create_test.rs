// tests para create_loan (repo-level)
// usamos los mismos helpers y patrón de runtime que en los tests existentes

use general_api::models::graphql::LoanStatus;

// local test-only lock para serializar acceso a redis cuando los tests corren en paralelo
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}

// include shared test helpers from tests/graphql/common/mod.rs
include!("common/mod.rs");
use general_api::repos::auth::utils::hashing_composite_key;
use redis::{Value as RedisValue, from_redis_value};
use general_api::models::redis::Loan as RedisLoan;
use serde_json::from_str;

/// helper para crear el mapping affiliate_key -> db_access_token en redis
fn setup_affiliate_key_mapping(
    context: &GeneralContext,
    affiliate_key: &str,
    db_access_token: &str,
) {
    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let key = format!("affiliate_key_to_db_access:{}", affiliate_key);
    let _: () = con.set(&key, db_access_token).expect("no se pudo crear mapping");
}

#[test]
fn test_repo_create_loan_happy_path() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let affiliate_key = format!("test_affiliate_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let db_access_token = hashing_composite_key(&[&affiliate_key]);

    // setup del mapping affiliate_key -> db_access_token
    setup_affiliate_key_mapping(&context, &affiliate_key, &db_access_token);
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));

    // llamar al repo a través del contexto
    let repo = context.loan_repo();
    let res = repo.create_loan(
        affiliate_key.clone(),
        12,
        5000.0,
        "compra de equipo".to_string(),
    );
    assert!(res.is_ok(), "create_loan retornó error: {:?}", res);

    // verificar existencia de la key en redis
    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:loans:*", db_access_token))
        .unwrap()
        .collect();
    assert!(!keys.is_empty(), "expected at least one loan key in redis");

    // registrar keys para limpieza
    for key in keys {
        guard.register_key(key);
    }
}

#[test]
fn test_repo_create_loan_persists_json_content() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let affiliate_key = format!("test_affiliate_content_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let db_access_token = hashing_composite_key(&[&affiliate_key]);

    // setup del mapping
    setup_affiliate_key_mapping(&context, &affiliate_key, &db_access_token);
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));

    let repo = context.loan_repo();
    let total_quota = 24;
    let base_needed_payment = 10000.0;
    let reason = "préstamo para vivienda".to_string();

    let res = repo.create_loan(
        affiliate_key.clone(),
        total_quota,
        base_needed_payment,
        reason.clone(),
    );
    assert!(res.is_ok());

    // buscar la key creada y leer el json
    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let keys_iter = con
        .scan_match::<String, String>(format!("users:{}:loans:*", db_access_token))
        .unwrap();
    let keys: Vec<String> = keys_iter.collect();
    assert!(!keys.is_empty(), "expected at least one loan key in redis");

    for key in &keys {
        guard.register_key(key.clone());
    }

    // leer primer key json y parsear
    let redis_raw: RedisValue = con.json_get(keys[0].as_str(), "$").expect("json_get failed");
    let nested_data = from_redis_value::<String>(&redis_raw).expect("from_redis_value failed");
    let parsed: Vec<RedisLoan> = from_str(nested_data.as_str()).expect("serde_json parse failed");
    let rl = parsed.get(0).expect("no element in parsed vector");

    // verificar campos
    assert_eq!(rl.total_quota, total_quota);
    assert!((rl.base_needed_payment - base_needed_payment).abs() < 1e-6);
    assert!((rl.payed - 0.0).abs() < 1e-6, "payed debería ser 0.0");
    assert!((rl.debt - base_needed_payment).abs() < 1e-6, "debt debería ser igual a base_needed_payment");
    assert!((rl.total - base_needed_payment).abs() < 1e-6, "total debería ser igual a base_needed_payment");
    assert_eq!(rl.status, "PENDING");
    assert_eq!(rl.reason, reason);
}

#[test]
fn test_repo_create_loan_twice_creates_two_keys() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let affiliate_key = format!("test_affiliate_two_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let db_access_token = hashing_composite_key(&[&affiliate_key]);

    // setup del mapping
    setup_affiliate_key_mapping(&context, &affiliate_key, &db_access_token);
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));

    let repo = context.loan_repo();

    let _ = repo.create_loan(
        affiliate_key.clone(),
        6,
        1000.0,
        "préstamo 1".to_string(),
    );
    let _ = repo.create_loan(
        affiliate_key.clone(),
        12,
        2000.0,
        "préstamo 2".to_string(),
    );

    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:loans:*", db_access_token))
        .unwrap()
        .collect();
    assert!(
        keys.len() >= 2,
        "expected at least two loan keys after two create_loan calls"
    );

    for key in &keys {
        guard.register_key(key.clone());
    }
}

#[test]
fn test_create_loan_collision_behavior() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let affiliate_key = format!("test_collision_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let db_access_token = hashing_composite_key(&[&affiliate_key]);

    // setup del mapping
    setup_affiliate_key_mapping(&context, &affiliate_key, &db_access_token);
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));

    let repo = context.loan_repo();

    // crear dos loans con parámetros idénticos; current implementation usa count-based hash key
    let _ = repo.create_loan(
        affiliate_key.clone(),
        10,
        3000.0,
        "mismo motivo".to_string(),
    );
    let _ = repo.create_loan(
        affiliate_key.clone(),
        10,
        3000.0,
        "mismo motivo".to_string(),
    );

    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:loans:*", db_access_token))
        .unwrap()
        .collect();

    // current behavior: debería crear dos keys (non-overwriting). assert >=2
    assert!(
        keys.len() >= 2,
        "expected at least two keys for collision behavior, got {}",
        keys.len()
    );

    for key in &keys {
        guard.register_key(key.clone());
    }
}

#[test]
fn test_create_loan_invalid_affiliate_key() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();

    let repo = context.loan_repo();

    // intentar crear loan con affiliate_key que no existe
    let res = repo.create_loan(
        "affiliate_inexistente".to_string(),
        12,
        5000.0,
        "test".to_string(),
    );

    assert!(res.is_err(), "expected error for invalid affiliate_key");
}
