// Tests para SCRUM-202: crear pagos via mutation (mutation-level)
// Reutilizamos los helpers existentes y patrón de runtime

use general_api::models::graphql::{Payment, PaymentStatus};
use general_api::endpoints::handlers::graphql::payment::PaymentMutation;
use general_api::repos::graphql::utils::{create_test_context, clear_redis, insert_payment_helper};
use general_api::test_sync::REDIS_TEST_LOCK;
use general_api::repos::auth::utils::hashing_composite_key;
use redis::Commands;

#[test]
fn test_mutation_create_user_payment_happy_path() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_mut_create_1", now),
        name: "Mutation Create Test".to_string(),
        total_amount: 55.0,
        payment_date: "2025-10-13".to_string(),
        ticket_num: "MC1".to_string(),
        account_num: "MACC1".to_string(),
        commentary: Some("mutation create test".to_string()),
        photo: "url_mut_create".to_string(),
        state: PaymentStatus::OnRevision,
    };

    // Ejecutar la mutation que crea el pago. Según la implementación actual, la mutation
    // puede delegar a repo create_payment; ajustamos la llamada directa si existe método.
    // La firma real de la mutation es: (context, access_token, name, total_amount, ticket_number, account_number, being_payed)
    let access_token = "testuser_mut_create".to_string();
    let res = tokio::runtime::Runtime::new().unwrap().block_on(async {
        PaymentMutation::create_user_payment(
            &context,
            access_token.clone(),
            payment.name.clone(),
            payment.total_amount,
            payment.ticket_num.clone(),
            payment.account_num.clone(),
            vec![],
        ).await
    }).expect("create_user_payment mutation failed");

    // La implementación actual devuelve Result<String, String> con el mensaje "Payment Created"
    assert_eq!(res, "Payment Created");

    // Verificar existencia de la key en Redis
    let composite = hashing_composite_key(&[&access_token]);
    let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
    let keys: Vec<String> = con.scan_match(format!("users:{}:payments:*", composite)).unwrap().collect();
    assert!(!keys.is_empty(), "Expected at least one payment key in Redis after mutation");
}

#[test]
fn test_mutation_create_user_payment_with_negative_amount() {
    // Documenting current behavior: create_payment does not currently validate negative amounts.
    // This test asserts the current behavior so changes will be visible in CI if validation is added later.
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);

    let access_token = "test_neg_amount".to_string();
    let res = tokio::runtime::Runtime::new().unwrap().block_on(async {
        PaymentMutation::create_user_payment(
            &context,
            access_token.clone(),
            "Neg Amount Test".to_string(),
            -100.0,
            "T_NEG".to_string(),
            "A_NEG".to_string(),
            vec![],
        ).await
    });

    // Current implementation returns Ok("Payment Created") even for negative amounts.
    // We assert that behavior; if you prefer to reject negatives, change production code and update test.
    assert!(res.is_ok(), "Expected create_user_payment to succeed currently, got {:?}", res);
    assert_eq!(res.unwrap(), "Payment Created");
}
