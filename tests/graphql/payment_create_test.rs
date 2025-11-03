// Tests para SCRUM-202: create_payment (repo-level)
// Usamos los mismos helpers y patrón de runtime que en los tests existentes

use general_api::models::graphql::PaymentStatus;

// Local test-only lock to serialize Redis access when running tests in parallel.
// Use std::sync::OnceLock to avoid adding dependencies.
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}
// Include shared test helpers from tests/graphql/common/mod.rs
include!("common/mod.rs");
use general_api::repos::auth::utils::hashing_composite_key;
use redis::{Value as RedisValue, from_redis_value};
use general_api::models::redis::Payment as RedisPayment;
use serde_json::from_str;

#[test]
fn test_repo_create_payment_happy_path() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let payment = Payment {
        id: format!("test_pago_{}_create_1", now),
        name: "Repo Create Test".to_string(),
        total_amount: 123.45,
        payment_date: "2025-10-13".to_string(),
        ticket_num: "RC1".to_string(),
        account_num: "RACC1".to_string(),
        commentary: Some("create repo test".to_string()),
        photo: "url_create".to_string(),
        state: PaymentStatus::OnRevision,
    };

    // Llamar al repo a través del contexto con la firma real
    let repo = context.payment_repo();
    let access_token = "testuser_create_repo".to_string();
    let res = repo.create_payment(
        access_token.clone(),
        payment.name.clone(),
        payment.total_amount,
        payment.ticket_num.clone(),
        payment.account_num.clone(),
        vec![],
    );
    assert!(res.is_ok(), "create_payment returned error: {:?}", res);

    // Verificar existencia de la key en Redis
    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:payments:*", composite))
        .unwrap()
        .collect();
    assert!(!keys.is_empty(), "Expected at least one payment key in Redis");
    // Register created keys so the guard will remove them after the test
    for key in keys {
        guard.register_key(key);
    }
}

#[test]
fn test_create_then_get_all_returns_created_payment() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let repo = context.payment_repo();
    let access_token = format!("testuser_all_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());

    // create a payment using the repo
    let res = repo.create_payment(
        access_token.clone(),
        "AllTest".to_string(),
        42.0,
        "T_ALL".to_string(),
        "A_ALL".to_string(),
        vec![],
    );
    assert!(res.is_ok(), "create_payment failed: {:?}", res);

    // Now call get_all_payments and assert we find at least one payment with the expected account_number
    let all = repo.get_all_payments().expect("get_all_payments failed");
    let found = all.iter().any(|p| p.account_num == "A_ALL".to_string());
    // register keys for cleanup: scan user's payments and register
    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No redis conn");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:payments:*", composite))
        .unwrap()
        .collect();
    for key in keys {
        guard.register_key(key);
    }

    assert!(found, "Expected created payment to appear in get_all_payments");
}

#[test]
fn test_repo_create_payment_persists_json_content() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let access_token = "testuser_create_repo_content".to_string();
    let payment_name = "Repo Create Content Test".to_string();
    let total_amount = 777.77_f64;

    let repo = context.payment_repo();
    let res = repo.create_payment(
        access_token.clone(),
        payment_name.clone(),
        total_amount,
        "RC_CONTENT".to_string(),
        "RACC_CONTENT".to_string(),
        vec![],
    );
    assert!(res.is_ok());

    // Buscar la key creada y leer el JSON
    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let keys_iter = con
        .scan_match::<String, String>(format!("users:{}:payments:*", composite))
        .unwrap();
    let keys: Vec<String> = keys_iter.collect();
    assert!(!keys.is_empty(), "Expected at least one payment key in Redis");
    for key in &keys {
        guard.register_key(key.clone());
    }

    // Leer primer key JSON y parsear
    let redis_raw: RedisValue = con.json_get(keys[0].as_str(), "$").expect("json_get failed");
    let nested_data = from_redis_value::<String>(&redis_raw).expect("from_redis_value failed");
    let parsed: Vec<RedisPayment> = from_str(nested_data.as_str()).expect("serde_json parse failed");
    let rp = parsed.get(0).expect("No element in parsed vector");

    assert_eq!(rp.name, payment_name);
    assert!((rp.total_amount - total_amount).abs() < 1e-6);
    assert_eq!(rp.account_number, "RACC_CONTENT");
    assert_eq!(rp.ticket_number, "RC_CONTENT");
    assert_eq!(rp.status, "ON_REVISION");
}

#[test]
fn test_repo_create_payment_twice_creates_two_keys() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let access_token = "testuser_create_repo_two".to_string();
    let repo = context.payment_repo();

    let _ = repo.create_payment(
        access_token.clone(),
        "N1".to_string(),
        1.0,
        "T1".to_string(),
        "A1".to_string(),
        vec![],
    );
    let _ = repo.create_payment(
        access_token.clone(),
        "N2".to_string(),
        2.0,
        "T2".to_string(),
        "A2".to_string(),
        vec![],
    );

    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:payments:*", composite))
        .unwrap()
        .collect();
    assert!(
        keys.len() >= 2,
        "Expected at least two payment keys after two create_payment calls"
    );
    for key in &keys {
        guard.register_key(key.clone());
    }
}

#[test]
fn test_create_payment_collision_behavior() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let access_token = "test_collision".to_string();
    let repo = context.payment_repo();

    // Create two payments with identical visible params; current implementation uses count-based hash key
    let _ = repo.create_payment(
        access_token.clone(),
        "SameName".to_string(),
        10.0,
        "T1".to_string(),
        "A1".to_string(),
        vec![],
    );
    let _ = repo.create_payment(
        access_token.clone(),
        "SameName".to_string(),
        10.0,
        "T1".to_string(),
        "A1".to_string(),
        vec![],
    );

    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:payments:*", composite))
        .unwrap()
        .collect();
    // Current behavior: should create two keys (non-overwriting). Assert >=2.
    assert!(
        keys.len() >= 2,
        "Expected at least two keys for collision behavior, got {}",
        keys.len()
    );
    for key in &keys {
        guard.register_key(key.clone());
    }
}

#[test]
fn test_guard_does_not_remove_unrelated_keys() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let guard = TestRedisGuard::new(context.pool.clone());

    // Create an unrelated key that should NOT be removed by the guard
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let unrelated_key = "unrelated:test:key".to_string();
    let _: () = con.set(&unrelated_key, "preserve").expect("couldn't set unrelated key");

    // Drop guard without registering any keys; it should not delete unrelated keys
    drop(guard);
    let exists: bool = con.exists(&unrelated_key).unwrap_or(false);
    assert!(exists, "TestRedisGuard removed an unrelated key: {}", unrelated_key);
}
