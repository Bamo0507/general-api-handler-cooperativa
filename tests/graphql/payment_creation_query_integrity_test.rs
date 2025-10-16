// Tests adicionales para SCRUM-252: query sobre pagos creados e integridad de datos

use general_api::models::graphql::PaymentStatus;
use general_api::endpoints::handlers::graphql::payment::PaymentQuery;
// Include shared test helpers from tests/graphql/common/mod.rs
include!("common/mod.rs");
use general_api::repos::auth::utils::hashing_composite_key;

// Serialización simple para evitar race conditions con Redis cuando se ejecutan tests en paralelo
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}

#[test]
fn test_query_returns_payments_created_via_repo_helper_and_preserves_fields() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Crear dos payments mediante el helper (simula persistencia del repo)
    let _access_token = "scrum252_user".to_string();
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    let p1 = Payment {
        id: format!("test_pago_{}_scrum252_1", now),
        name: "Scrum252 One".to_string(),
        total_amount: 42.5,
        payment_date: "2025-10-13".to_string(),
        ticket_num: "S252-1".to_string(),
        account_num: "ACC-S252-1".to_string(),
        commentary: Some("coment1".to_string()),
        photo: "photo1".to_string(),
        state: PaymentStatus::OnRevision,
    };

    let p2 = Payment {
        id: format!("test_pago_{}_scrum252_2", now),
        name: "Scrum252 Two".to_string(),
        total_amount: 1000.0,
        payment_date: "2025-10-14".to_string(),
        ticket_num: "S252-2".to_string(),
        account_num: "ACC-S252-2".to_string(),
        commentary: None,
        photo: "photo2".to_string(),
        state: PaymentStatus::Accepted,
    };

    let k1 = insert_payment_helper_and_return(&context, &p1);
    let k2 = insert_payment_helper_and_return(&context, &p2);
    guard.register_key(k1);
    guard.register_key(k2);

    // Ejecutar la query
    let result = futures::executor::block_on(async {
        PaymentQuery::get_users_payments(&context, String::from("all")).await
    }).unwrap();

    // Buscar por id y verificar integridad de campos
    let found1 = result.iter().find(|r| r.id == p1.id);
    assert!(found1.is_some(), "p1 no fue retornado por la query");
    let a1 = found1.unwrap();
    assert_eq!(a1.name, p1.name);
    assert_eq!(a1.total_amount, p1.total_amount);
    assert_eq!(a1.ticket_num, p1.ticket_num);
    assert_eq!(a1.account_num, p1.account_num);

    let found2 = result.iter().find(|r| r.id == p2.id);
    assert!(found2.is_some(), "p2 no fue retornado por la query");
    let a2 = found2.unwrap();
    assert_eq!(a2.name, p2.name);
    assert_eq!(a2.total_amount, p2.total_amount);
    assert_eq!(a2.state, p2.state);
}

#[test]
fn test_query_returns_payments_created_via_mutation_and_repo_consistency() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Crear payment vía la mutation (si la mutation delega en repo, debe ser visible via la query)
    use general_api::endpoints::handlers::graphql::payment::PaymentMutation;
    let access_token = "scrum252_mut_user".to_string();
    let _now = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    let payment_name = "Scrum252 Mut".to_string();
    let ticket = "S252-MUT".to_string();
    let account = "ACC-S252-MUT".to_string();
    let total_amount = 321.0_f64;

    let res = futures::executor::block_on(async {
        PaymentMutation::create_user_payment(
            &context,
            access_token.clone(),
            payment_name.clone(),
            total_amount,
            ticket.clone(),
            account.clone(),
            vec![],
        ).await
    }).expect("mutation create failed");

    assert_eq!(res, "Payment Created");

    // Register keys created by mutation
    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        let keys: Vec<String> = con.scan_match(format!("users:{}:payments:*", hashing_composite_key(&[&access_token.clone()]))).unwrap().collect();
        for k in keys { guard.register_key(k); }
    }

    // Ahora la query debe devolver al menos un payment con los campos creados
    let result = futures::executor::block_on(async {
        PaymentQuery::get_users_payments(&context, access_token.clone()).await
    }).unwrap();

    assert!(!result.is_empty(), "Query no devolvió pagos tras la mutation");
    let r = &result[0];
    assert_eq!(r.ticket_num, ticket);
    assert_eq!(r.account_num, account);
    assert!((r.total_amount - total_amount).abs() < 1e-6);
}
