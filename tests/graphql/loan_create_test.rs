#[test]
fn test_repo_get_user_loans_by_access_token() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_getloans_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass_getloans".to_string();
    let real_name = "Test User GetLoans".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    let repo = context.loan_repo();

    // crear dos préstamos para el usuario
    // capture returned keys and fail fast if creation fails
    let _res1 = repo
        .create_loan(
            affiliate_key.clone(),
            6,
            1000.0,
            0.10,
            "préstamo test 1".to_string(),
        )
        .expect("create_loan 1 failed");

    let _res2 = repo
        .create_loan(
            affiliate_key.clone(),
            12,
            2000.0,
            0.08,
            "préstamo test 2".to_string(),
        )
        .expect("create_loan 2 failed");

    // ahora obtener los préstamos usando get_user_loans
    let loans = repo
        .get_user_loans(token_info.access_token.clone())
        .expect("get_user_loans should not fail");

    // Deben existir al menos dos préstamos y deben tener los reasons correctos
    let reasons: Vec<String> = loans.iter().map(|l| l.reason.clone()).collect();
    assert!(reasons.contains(&"préstamo test 1".to_string()), "Debe contener préstamo test 1");
    assert!(reasons.contains(&"préstamo test 2".to_string()), "Debe contener préstamo test 2");
    assert!(loans.len() >= 2, "Debe haber al menos dos préstamos para el usuario");

    // registrar las keys de los préstamos para limpieza
    let mut con = context.pool.get().expect("no se pudo obtener conexión de redis");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:loans:*", db_access_token))
        .unwrap()
        .collect();
    for key in keys {
        guard.register_key(key);
    }
}
// tests para create_loan (repo-level)
// usamos los mismos helpers y patrón de runtime que en los tests existentes

// local test-only lock para serializar acceso a redis cuando los tests corren en paralelo
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}

// include shared test helpers from tests/graphql/common/mod.rs
include!("common/mod.rs");
use general_api::repos::auth::{create_user_with_access_token, utils::hashing_composite_key};
use redis::{Value as RedisValue, from_redis_value};
use general_api::models::redis::Loan as RedisLoan;
use serde_json::from_str;

#[test]
fn test_repo_create_loan_happy_path() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_loan_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass123".to_string();
    let real_name = "Test User Loan".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    // calcular affiliate_key de la misma forma que lo hace create_user_with_access_token
    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    // llamar al repo a través del contexto
    let repo = context.loan_repo();
    let res = repo.create_loan(
        affiliate_key.clone(),
        12,
        5000.0,
        0.15, // interest_rate 15%
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

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_content_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass456".to_string();
    let real_name = "Test User Content".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    let repo = context.loan_repo();
    let total_quota = 24;
    let base_needed_payment = 10000.0;
    let reason = "préstamo para vivienda".to_string();

    let res = repo.create_loan(
        affiliate_key.clone(),
        total_quota,
        base_needed_payment,
        0.12, // interest_rate 12%
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
    assert!((rl.interest_rate - 0.12).abs() < 1e-6, "interest_rate debería ser 0.12");
}

#[test]
fn test_repo_create_loan_twice_creates_two_keys() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_two_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass789".to_string();
    let real_name = "Test User Two Loans".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    let repo = context.loan_repo();

    let _ = repo.create_loan(
        affiliate_key.clone(),
        6,
        1000.0,
        0.10, // interest_rate 10%
        "préstamo 1".to_string(),
    );
    let _ = repo.create_loan(
        affiliate_key.clone(),
        12,
        2000.0,
        0.08, // interest_rate 8%
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

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_collision_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass101".to_string();
    let real_name = "Test User Collision".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    let repo = context.loan_repo();

    // crear dos loans con parámetros idénticos; current implementation usa count-based hash key
    let _ = repo.create_loan(
        affiliate_key.clone(),
        10,
        3000.0,
        0.20, // interest_rate 20%
        "mismo motivo".to_string(),
    );
    let _ = repo.create_loan(
        affiliate_key.clone(),
        10,
        3000.0,
        0.20, // interest_rate 20%
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
fn test_create_then_get_all_returns_created_loan() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // crear usuario primero para establecer el mapeo affiliate_key
    let user_name = format!("testuser_getall_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    let password = "testpass202".to_string();
    let real_name = "Test User GetAll".to_string();

    let token_info = create_user_with_access_token(user_name.clone(), password, real_name)
        .expect("Failed to create user");

    let affiliate_key = hashing_composite_key(&[&user_name]);
    let db_access_token = hashing_composite_key(&[&token_info.access_token]);

    // registrar keys de usuario para limpieza
    guard.register_key(format!("users_on_used:{}", user_name));
    guard.register_key(format!("affiliate_keys:{}", affiliate_key));
    guard.register_key(format!("affiliate_key_to_db_access:{}", affiliate_key));
    guard.register_key(format!("users:{}:complete_name", db_access_token));
    guard.register_key(format!("users:{}:affiliate_key", db_access_token));
    guard.register_key(format!("users:{}:payed_to_capital", db_access_token));
    guard.register_key(format!("users:{}:owed_capital", db_access_token));

    let repo = context.loan_repo();

    // crear un loan usando el repo
    let res = repo.create_loan(
        affiliate_key.clone(),
        18,
        7500.0,
        0.18, // interest_rate 18%
        "préstamo para get_all test".to_string(),
    );
    assert!(res.is_ok(), "create_loan failed: {:?}", res);

    // llamar a get_all_loans y verificar que encontramos al menos un loan con reason esperado
    let all = repo.get_all_loans().expect("get_all_loans failed");
    let found = all.iter().any(|l| l.reason == "préstamo para get_all test");

    // registrar keys para cleanup
    let mut con = context.pool.get().expect("no redis conn");
    let keys: Vec<String> = con
        .scan_match(format!("users:{}:loans:*", db_access_token))
        .unwrap()
        .collect();
    for key in keys {
        guard.register_key(key);
    }

    assert!(found, "expected created loan to appear in get_all_loans");
}
