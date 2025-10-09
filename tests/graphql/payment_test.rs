// Pruebas unitarias para la query get_all_payments
// No se usa dotenv, las variables se cargan directamente

use general_api::models::graphql::Payment;
use general_api::endpoints::handlers::graphql::payment::PaymentQuery;
use general_api::repos::graphql::utils::{create_test_context, clear_redis, insert_payment_helper};

#[tokio::test]
async fn test_get_all_payments_returns_all_inserted_payments() {
    // Crear contexto y limpiar Redis SOLO UNA VEZ
    let context = create_test_context();
    clear_redis(&context);

    // Insertar pagos de prueba
    use general_api::models::graphql::PaymentStatus;
    let now = chrono::Utc::now().timestamp();
    let payments = vec![
        Payment {
            id: format!("test_pago_{}_1", now),
            total_amount: 100.0,
            payment_date: "2025-10-09".to_string(),
            ticket_num: "A123".to_string(),
            account_num: "ACC1".to_string(),
            commentary: "Pago test 1".to_string(),
            photo: "url1".to_string(),
            state: PaymentStatus::from_string("aprobado".to_string()),
        },
        Payment {
            id: format!("test_pago_{}_2", now),
            total_amount: 200.0,
            payment_date: "2025-10-10".to_string(),
            ticket_num: "B456".to_string(),
            account_num: "ACC2".to_string(),
            commentary: "Pago test 2".to_string(),
            photo: "url2".to_string(),
            state: PaymentStatus::from_string("pendiente".to_string()),
        },
    ];

    for payment in &payments {
        insert_payment_helper(&context, payment);
    }

    // Debug: mostrar claves en Redis después de insertar pagos
    {
        let pool = context.pool.clone();
        let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("*")
            .query(&mut con)
            .unwrap_or_default();
        println!("Claves en Redis tras inserción: {:?}", keys);
        // Verificar que existen ambas claves de pago
        let all_str = String::from("all");
        let composite_key = general_api::repos::auth::utils::hashing_composite_key(&[&all_str]);
    let key1 = format!("users:{}:payments:{}", composite_key, payments[0].id);
    let key2 = format!("users:{}:payments:{}", composite_key, payments[1].id);
    assert!(keys.contains(&key1), "No se encontró la clave del pago 1 en Redis");
    assert!(keys.contains(&key2), "No se encontró la clave del pago 2 en Redis");
    }

    // Ejecutar la query
    let result = PaymentQuery::get_all_payments(&context).await.unwrap();

    // Validar que retorna todos los pagos insertados
    assert_eq!(result.len(), payments.len());
    for (expected, actual) in payments.iter().zip(result.iter()) {
        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.total_amount, actual.total_amount);
        assert_eq!(expected.payment_date, actual.payment_date);
        assert_eq!(expected.ticket_num, actual.ticket_num);
        assert_eq!(expected.account_num, actual.account_num);
        assert_eq!(expected.commentary, actual.commentary);
        assert_eq!(expected.photo, actual.photo);
        assert_eq!(expected.state, actual.state);
    }
}

// Helpers ahora importados desde tests/utils/redis_helpers.rs
