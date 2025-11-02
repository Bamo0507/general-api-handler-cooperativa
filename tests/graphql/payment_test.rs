// Pruebas unitarias para la query get_all_payments
// No se usa dotenv, las variables se cargan directamente

use general_api::models::graphql::Payment;
use general_api::endpoints::handlers::graphql::payment::PaymentQuery;
use super::common::{create_test_context, insert_payment_helper_and_return, TestRedisGuard};
use general_api::test_sync::REDIS_TEST_LOCK;
use general_api::repos::auth::utils::hashing_composite_key;
use redis::Commands;

#[test]
fn test_get_all_payments_returns_all_inserted_payments() {
    // Serializar pruebas que tocan Redis sin dependencias externas
    // Acquire a blocking mutex guard to serialize access across tests
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    // Crear contexto y guard para limpieza de claves de test
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Insertar pagos de prueba
    use general_api::models::graphql::PaymentStatus;
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let payments = vec![
        Payment {
            id: format!("test_pago_{}_1", now),
            name: "Test".to_string(),
            total_amount: 100.0,
            payment_date: "2025-10-09".to_string(),
            ticket_num: "A123".to_string(),
            account_num: "ACC1".to_string(),
            commentary: Some("Pago test 1".to_string()),
            photo: "url1".to_string(),
            state: PaymentStatus::from_string("ACCEPTED".to_string()),
        },
        Payment {
            id: format!("test_pago_{}_2", now),
            name: "Test".to_string(),
            total_amount: 200.0,
            payment_date: "2025-10-10".to_string(),
            ticket_num: "B456".to_string(),
            account_num: "ACC2".to_string(),
            commentary: Some("Pago test 2".to_string()),
            photo: "url2".to_string(),
            state: PaymentStatus::from_string("ON_REVISION".to_string()),
        },
    ];

    let mut inserted_keys: Vec<String> = Vec::new();
    for payment in &payments {
        let k = insert_payment_helper_and_return(&context, payment);
        inserted_keys.push(k.clone());
        guard.register_key(k);
    }

    // Debug: verificar que las claves insertadas existen en Redis
    {
        let pool = context.pool.clone();
        let mut con = pool.get().expect("No se pudo obtener conexión de Redis");

        assert!(con.exists::<_, bool>(&inserted_keys[0]).unwrap(), "No se encontró la clave del pago 1 en Redis");
        assert!(con.exists::<_, bool>(&inserted_keys[1]).unwrap(), "No se encontró la clave del pago 2 en Redis");
    }

    // Ejecutar la query
    let mut result = futures::executor::block_on(async {
        PaymentQuery::get_all_payments(&context).await.unwrap()
    });

    // Ordenar ambos vectores por id para evitar dependencia del orden de Redis
    let mut expected_sorted = payments.clone();
    expected_sorted.sort_by(|a, b| a.id.cmp(&b.id));
    result.sort_by(|a, b| a.id.cmp(&b.id));

    // Validar que los pagos insertados están presentes en el resultado (no exigimos exclusividad
    // porque el entorno de pruebas puede tener otros elementos). Buscamos por id y comparamos campos.
    for expected in expected_sorted.iter() {
        let found = result.iter().find(|r| r.id == expected.id);
        assert!(found.is_some(), "Expected payment with id {} not found", expected.id);
        let actual = found.unwrap();
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
