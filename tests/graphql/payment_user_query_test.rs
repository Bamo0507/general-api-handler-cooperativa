// Tests para SCRUM-201: query get_users_payments
// No modificar archivos de producción; usar helpers existentes y runtime local

// Payment struct is brought in by the included common helpers
use general_api::models::graphql::PaymentStatus;
use general_api::models::redis::Payment as RedisPayment;
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::endpoints::handlers::graphql::payment::PaymentQuery;
// Include shared test helpers from tests/graphql/common/mod.rs
include!("common/mod.rs");

// Local lock helper (avoid depending on non-existent general_api::test_sync)
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}
// Inserta directamente en Redis bajo la clave del access_token provisto
fn insert_payment_for_user(context: &GeneralContext, access_token: &str, payment: &Payment) {
    let composite = hashing_composite_key(&[&access_token.to_string()]);
    let redis_key = format!("users:{}:payments:{}", composite, payment.id);
    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
    let redis_payment = RedisPayment {
        date_created: payment.payment_date.clone(),
        account_number: payment.account_num.clone(),
        total_amount: payment.total_amount,
        name: payment.name.clone(),
        comments: payment.commentary.clone(),
        comprobante_bucket: payment.photo.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_string(),
        being_payed: vec![],
    };
    let _: () = con.json_set(&redis_key, "$", &redis_payment).expect("No se pudo insertar payment en redis");
}

#[test]
fn test_get_users_payments_returns_inserted_payments() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Preparar datos
    let access_token = "testuser_a".to_string();
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let payments = vec![
        Payment {
            id: format!("test_pago_{}_a1", now),
            name: "User A 1".to_string(),
            total_amount: 10.0,
            payment_date: "2025-10-12".to_string(),
            ticket_num: "T1".to_string(),
            account_num: "ACC1".to_string(),
            commentary: Some("c1".to_string()),
            photo: "p1".to_string(),
            state: PaymentStatus::OnRevision,
        },
        Payment {
            id: format!("test_pago_{}_a2", now),
            name: "User A 2".to_string(),
            total_amount: 20.0,
            payment_date: "2025-10-12".to_string(),
            ticket_num: "T2".to_string(),
            account_num: "ACC2".to_string(),
            commentary: Some("c2".to_string()),
            photo: "p2".to_string(),
            state: PaymentStatus::OnRevision,
        },
    ];

    for p in &payments {
        insert_payment_for_user(&context, &access_token, p);
        // register the key created using known pattern
        let composite = hashing_composite_key(&[&access_token.clone()]);
        let key = format!("users:{}:payments:{}", composite, p.id);
        guard.register_key(key);
    }

    let result = futures::executor::block_on(async {
        PaymentQuery::get_users_payments(&context, access_token.clone()).await
    }).unwrap();

    // Verificar que los pagos insertados están presentes
    for expected in payments.iter() {
        let found = result.iter().find(|r| r.id == expected.id);
        assert!(found.is_some(), "No se encontró el pago esperado {}", expected.id);
        let actual = found.unwrap();
        assert_eq!(actual.total_amount, expected.total_amount);
        assert_eq!(actual.payment_date, expected.payment_date);
        assert_eq!(actual.ticket_num, expected.ticket_num);
        assert_eq!(actual.account_num, expected.account_num);
    }
}

#[test]
fn test_get_users_payments_filters_other_users() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let user_a = "user_a".to_string();
    let user_b = "user_b".to_string();
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    let payment_a = Payment {
        id: format!("test_pago_{}_ua", now),
        name: "A".to_string(),
        total_amount: 11.0,
        payment_date: "2025-10-12".to_string(),
        ticket_num: "TA".to_string(),
        account_num: "ACCA".to_string(),
        commentary: None,
        photo: "p".to_string(),
        state: PaymentStatus::OnRevision,
    };

    let payment_b = Payment {
        id: format!("test_pago_{}_ub", now),
        name: "B".to_string(),
        total_amount: 22.0,
        payment_date: "2025-10-12".to_string(),
        ticket_num: "TB".to_string(),
        account_num: "ACCB".to_string(),
        commentary: None,
        photo: "p".to_string(),
        state: PaymentStatus::OnRevision,
    };

    insert_payment_for_user(&context, &user_a, &payment_a);
    insert_payment_for_user(&context, &user_b, &payment_b);
    // register both keys so they are cleaned up by the guard
    guard.register_key(format!("users:{}:payments:{}", hashing_composite_key(&[&user_a.clone()]), payment_a.id));
    guard.register_key(format!("users:{}:payments:{}", hashing_composite_key(&[&user_b.clone()]), payment_b.id));

    let result_a = futures::executor::block_on(async {
        PaymentQuery::get_users_payments(&context, user_a.clone()).await
    }).unwrap();

    assert!(result_a.iter().any(|p| p.id == payment_a.id));
    assert!(!result_a.iter().any(|p| p.id == payment_b.id));
}

#[test]
fn test_get_users_payments_no_payments_returns_empty() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let _guard_local = TestRedisGuard::new(context.pool.clone());

    let user = "no_payments_user".to_string();
    let result = futures::executor::block_on(async {
        PaymentQuery::get_users_payments(&context, user.clone()).await
    }).unwrap();

    assert!(result.is_empty(), "Esperábamos vector vacío si no hay pagos, obtuvimos {:?}", result);
}
