// Pruebas unitarias para la mutation approve_or_reject_payment
// Estructura y helpers igual a payment_test.rs

use general_api::models::graphql::{Payment, PaymentStatus};
use general_api::models::redis::Payment as RedisPayment;
use general_api::models::PayedTo;
use general_api::endpoints::handlers::graphql::payment::PaymentMutation;
use general_api::repos::graphql::utils::{create_test_context, clear_redis, insert_payment_helper};
use general_api::repos::auth::utils::hashing_composite_key;
use redis::JsonCommands;
use general_api::test_sync::REDIS_TEST_LOCK;

#[test]
fn test_aprobar_pago_pendiente() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_1", now),
        name: "Test".to_string(),
        total_amount: 100.0,
        payment_date: "2025-10-09".to_string(),
        ticket_num: "A123".to_string(),
        account_num: "ACC1".to_string(),
        commentary: Some("Pago test 1".to_string()),
        photo: "url1".to_string(),
        state: PaymentStatus::OnRevision,
    };
    // Insertar bajo la clave global 'all' para que la mutación lo encuentre
    let all_vec = vec![String::from("all")];
    let all_key = hashing_composite_key(&[&all_vec[0]]);
    let redis = &mut context.pool.get().expect("Couldn't connect to pool");
    let redis_payment = RedisPayment {
        date_created: payment.payment_date.clone(),
        account_number: payment.account_num.clone(),
        total_amount: payment.total_amount,
        name: payment.name.clone(),
        comments: payment.commentary.clone(),
        comprobante_bucket: payment.photo.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_owned(),
    being_payed: vec![PayedTo::default()],
    };
    let _: () = redis.json_set(
        format!("users:{}:payments:{}", all_key, payment.id),
        "$",
        &redis_payment,
    ).expect("No se pudo insertar el pago en la clave global 'all'");
    let result = futures::executor::block_on(async {
        PaymentMutation::approve_or_reject_payment(
            &context,
            payment.id.clone(),
            "ACCEPTED".to_string(),
            "".to_string(),
        ).await
    }).unwrap();
    assert_eq!(result.state, PaymentStatus::Accepted);
    assert_eq!(result.commentary, payment.commentary);
}

#[test]
fn test_rechazar_pago_pendiente_con_comentario() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_2", now),
        name: "Test".to_string(),
        total_amount: 200.0,
        payment_date: "2025-10-10".to_string(),
        ticket_num: "B456".to_string(),
        account_num: "ACC2".to_string(),
        commentary: Some("Pago test 2".to_string()),
        photo: "url2".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let comentario = "Pago rechazado por pruebas".to_string();
    let result = futures::executor::block_on(async {
        PaymentMutation::approve_or_reject_payment(
            &context,
            payment.id.clone(),
            "REJECTED".to_string(),
            comentario.clone(),
        ).await
    }).unwrap();
    assert_eq!(result.state, PaymentStatus::Rejected);
    assert_eq!(result.commentary, Some(comentario));
}

#[test]
fn test_rechazar_pago_pendiente_sin_comentario() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_3", now),
        name: "Test".to_string(),
        total_amount: 300.0,
        payment_date: "2025-10-11".to_string(),
        ticket_num: "C789".to_string(),
        account_num: "ACC3".to_string(),
        commentary: Some("Pago test 3".to_string()),
        photo: "url3".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let result = futures::executor::block_on(async {
        PaymentMutation::approve_or_reject_payment(
            &context,
            payment.id.clone(),
            "REJECTED".to_string(),
            "".to_string(),
        ).await
    });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Se requiere comentario al rechazar el pago");
}

#[test]
fn test_mutar_pago_ya_finalizado() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_4", now),
        name: "Test".to_string(),
        total_amount: 400.0,
        payment_date: "2025-10-12".to_string(),
        ticket_num: "D012".to_string(),
        account_num: "ACC4".to_string(),
        commentary: Some("Pago test 4".to_string()),
        photo: "url4".to_string(),
        state: PaymentStatus::Accepted,
    };
    insert_payment_helper(&context, &payment);
    let result = futures::executor::block_on(async {
        PaymentMutation::approve_or_reject_payment(
            &context,
            payment.id.clone(),
            "REJECTED".to_string(),
            "Intento mutar pago finalizado".to_string(),
        ).await
    });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "El pago ya está finalizado");
}

#[test]
fn test_mutar_con_estado_invalido() {
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    let context = create_test_context();
    clear_redis(&context);
    let now = chrono::Utc::now().timestamp_nanos();
    let payment = Payment {
        id: format!("test_pago_{}_5", now),
        name: "Test".to_string(),
        total_amount: 500.0,
        payment_date: "2025-10-13".to_string(),
        ticket_num: "E345".to_string(),
        account_num: "ACC5".to_string(),
        commentary: Some("Pago test 5".to_string()),
        photo: "url5".to_string(),
        state: PaymentStatus::OnRevision,
    };
    insert_payment_helper(&context, &payment);
    let result = futures::executor::block_on(async {
        PaymentMutation::approve_or_reject_payment(
            &context,
            payment.id.clone(),
            "INVALIDO".to_string(),
            "".to_string(),
        ).await
    });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Estado inválido, debe ser ACCEPTED o REJECTED");
}
