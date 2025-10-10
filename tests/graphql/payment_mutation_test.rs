// Pruebas unitarias para la mutation approve_or_reject_payment
// Estructura y helpers igual a payment_test.rs

use general_api::models::graphql::{Payment, PaymentStatus};
use general_api::endpoints::handlers::graphql::payment::PaymentMutation;
use general_api::repos::graphql::utils::{create_test_context, clear_redis, insert_payment_helper};
use redis::JsonCommands;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static REDIS_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[tokio::test]
async fn test_aprobar_pago_pendiente() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp();
    let payment = Payment {
        id: format!("test_pago_{}_1", now),
        total_amount: 100.0,
        payment_date: "2025-10-09".to_string(),
        ticket_num: "A123".to_string(),
        account_num: "ACC1".to_string(),
        commentary: "Pago test 1".to_string(),
        photo: "url1".to_string(),
        state: PaymentStatus::OnRevision,
    };
    // Insertar bajo la clave global 'all' para que la mutación lo encuentre
    let all_vec = vec![String::from("all")];
    let all_key = general_api::repos::auth::utils::hashing_composite_key(&[&all_vec[0]]);
    let redis = &mut context.pool.get().expect("Couldn't connect to pool");
    let redis_payment = general_api::models::redis::Payment {
        quantity: payment.total_amount,
        ticket_number: payment.ticket_num.clone(),
        date_created: payment.payment_date.clone(),
        comprobante_bucket: payment.photo.clone(),
        account_number: payment.account_num.clone(),
        comments: payment.commentary.clone(),
        status: payment.state.as_str().to_owned(),
    };
    let _: () = redis.json_set(
        format!("users:{}:payments:{}", all_key, payment.id),
        "$",
        &redis_payment,
    ).expect("No se pudo insertar el pago en la clave global 'all'");
    let result = PaymentMutation::approve_or_reject_payment(
        &context,
        payment.id.clone(),
        "ACCEPTED".to_string(),
        "".to_string(),
    ).await.unwrap();
    assert_eq!(result.state, PaymentStatus::Accepted);
    assert_eq!(result.commentary, payment.commentary);
}

#[tokio::test]
async fn test_rechazar_pago_pendiente_con_comentario() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp();
    let payment = Payment {
        id: format!("test_pago_{}_2", now),
        total_amount: 200.0,
        payment_date: "2025-10-10".to_string(),
        ticket_num: "B456".to_string(),
        account_num: "ACC2".to_string(),
        commentary: "Pago test 2".to_string(),
        photo: "url2".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let comentario = "Pago rechazado por pruebas".to_string();
    let result = PaymentMutation::approve_or_reject_payment(
        &context,
        payment.id.clone(),
        "REJECTED".to_string(),
        comentario.clone(),
    ).await.unwrap();
    assert_eq!(result.state, PaymentStatus::Rejected);
    assert_eq!(result.commentary, comentario);
}

#[tokio::test]
async fn test_rechazar_pago_pendiente_sin_comentario() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp();
    let payment = Payment {
        id: format!("test_pago_{}_3", now),
        total_amount: 300.0,
        payment_date: "2025-10-11".to_string(),
        ticket_num: "C789".to_string(),
        account_num: "ACC3".to_string(),
        commentary: "Pago test 3".to_string(),
        photo: "url3".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let result = PaymentMutation::approve_or_reject_payment(
        &context,
        payment.id.clone(),
        "REJECTED".to_string(),
        "".to_string(),
    ).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Commentary required when rejecting payment");
}

#[tokio::test]
async fn test_mutar_pago_ya_finalizado() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp();
    let payment = Payment {
        id: format!("test_pago_{}_4", now),
        total_amount: 400.0,
        payment_date: "2025-10-12".to_string(),
        ticket_num: "D012".to_string(),
        account_num: "ACC4".to_string(),
        commentary: "Pago test 4".to_string(),
        photo: "url4".to_string(),
        state: PaymentStatus::Accepted,
    };
    insert_payment_helper(&context, &payment);
    let result = PaymentMutation::approve_or_reject_payment(
        &context,
        payment.id.clone(),
        "REJECTED".to_string(),
        "Intento mutar pago finalizado".to_string(),
    ).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Payment already finalized");
}

#[tokio::test]
async fn test_mutar_con_estado_invalido() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp();
    let payment = Payment {
        id: format!("test_pago_{}_5", now),
        total_amount: 500.0,
        payment_date: "2025-10-13".to_string(),
        ticket_num: "E345".to_string(),
        account_num: "ACC5".to_string(),
        commentary: "Pago test 5".to_string(),
        photo: "url5".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let result = PaymentMutation::approve_or_reject_payment(
        &context,
        payment.id.clone(),
        "INVALIDO".to_string(),
        "".to_string(),
    ).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid new state, must be ACCEPTED or REJECTED");
}
