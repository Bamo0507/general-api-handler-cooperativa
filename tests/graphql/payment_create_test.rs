// Tests para SCRUM-202: create_payment (repo-level)
// Usamos los mismos helpers y patrón de runtime que en los tests existentes

use general_api::models::graphql::{Payment, PaymentStatus};
use general_api::repos::graphql::utils::{create_test_context, clear_redis};
use general_api::test_sync::REDIS_TEST_LOCK;
use general_api::repos::auth::utils::hashing_composite_key;
use redis::Commands;
use redis::{JsonCommands, Value as RedisValue, from_redis_value};
use general_api::models::redis::Payment as RedisPayment;
use serde_json::from_str;

#[test]
fn test_repo_create_payment_happy_path() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

    let now = chrono::Utc::now().timestamp_nanos();
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
}

#[test]
fn test_repo_create_payment_persists_json_content() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

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
    let keys_iter = con.scan_match::<String, String>(format!("users:{}:payments:*", composite)).unwrap();
    let keys: Vec<String> = keys_iter.collect();
    assert!(!keys.is_empty(), "Expected at least one payment key in Redis");

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
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

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
    let keys: Vec<String> = con.scan_match(format!("users:{}:payments:*", composite)).unwrap().collect();
    assert!(keys.len() >= 2, "Expected at least two payment keys after two create_payment calls");
}

#[test]
fn test_create_payment_collision_behavior() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

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
    let keys: Vec<String> = con.scan_match(format!("users:{}:payments:*", composite)).unwrap().collect();
    // Current behavior: should create two keys (non-overwriting). Assert >=2.
    assert!(keys.len() >= 2, "Expected at least two keys for collision behavior, got {}", keys.len());
}

#[test]
fn test_clear_redis_does_not_remove_unrelated_keys() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

    // Create an unrelated key that should NOT be removed by clear_redis
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let unrelated_key = "unrelated:test:key".to_string();
    let _: () = con.set(&unrelated_key, "preserve").expect("couldn't set unrelated key");

    // Run clear_redis and assert unrelated key still exists
    clear_redis(&context);
    let exists: bool = con.exists(&unrelated_key).unwrap_or(false);
    assert!(exists, "clear_redis removed an unrelated key: {}", unrelated_key);
}
