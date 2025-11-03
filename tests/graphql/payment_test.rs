// Pruebas unitarias para queries de payments
// No se usa dotenv, las variables se cargan directamente

use general_api::models::graphql::{Payment, PaymentHistory};
use general_api::endpoints::handlers::graphql::payment::PaymentQuery;
use general_api::repos::graphql::payment::PaymentRepo;
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

#[test]
fn test_get_user_payments_returns_only_user_payments() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto y guard para limpieza
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());
    
    // Crear un usuario específico con access_token
    let test_access_token = "test_user_payments_001";
    let db_access_token = hashing_composite_key(&[&test_access_token.to_string()]);
    
    // Insertar nombre del usuario en Redis
    {
        let mut con = context.pool.get().expect("Couldn't connect to pool");
        let name_key = format!("users:{}:complete_name", db_access_token);
        let _: () = con.set(&name_key, "Test User Payments").expect("Failed to set user name");
        guard.register_key(name_key);
    }
    
    // Crear pagos del usuario
    use general_api::models::graphql::PaymentStatus;
    use general_api::models::redis::Payment as RedisPayment;
    use general_api::models::PayedTo;
    use redis::JsonCommands;
    
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    
    let redis_payment1 = RedisPayment {
        date_created: "2025-10-15".to_string(),
        account_number: "ACC001".to_string(),
        total_amount: 150.0,
        name: "Pago usuario 1".to_string(),
        comments: Some("Comentario 1".to_string()),
        comprobante_bucket: "url1".to_string(),
        ticket_number: format!("TICKET_{}_1", now),
        status: "ACCEPTED".to_string(),
        being_payed: vec![PayedTo::default()],
    };
    
    let redis_payment2 = RedisPayment {
        date_created: "2025-10-16".to_string(),
        account_number: "ACC002".to_string(),
        total_amount: 250.0,
        name: "Pago usuario 2".to_string(),
        comments: Some("Comentario 2".to_string()),
        comprobante_bucket: "url2".to_string(),
        ticket_number: format!("TICKET_{}_2", now),
        status: "ON_REVISION".to_string(),
        being_payed: vec![PayedTo::default()],
    };
    
    // Insertar pagos en Redis
    {
        let mut con = context.pool.get().expect("Couldn't connect to pool");
        
        let key1 = format!("users:{}:payments:{}", db_access_token, redis_payment1.ticket_number);
        let key2 = format!("users:{}:payments:{}", db_access_token, redis_payment2.ticket_number);
        
        let _: () = con.json_set(&key1, "$", &redis_payment1).expect("Failed to insert payment 1");
        let _: () = con.json_set(&key2, "$", &redis_payment2).expect("Failed to insert payment 2");
        
        guard.register_key(key1);
        guard.register_key(key2);
    }
    
    // Ejecutar get_user_payments
    let repo = PaymentRepo {
        pool: context.pool.clone(),
    };
    
    let result = repo.get_user_payments(test_access_token.to_string())
        .expect("get_user_payments failed");
    
    // Validaciones
    assert_eq!(result.len(), 2, "Deberían haber 2 pagos del usuario");
    
    // Verificar que los pagos son los correctos
    let payment1 = result.iter().find(|p| p.ticket_num == redis_payment1.ticket_number);
    let payment2 = result.iter().find(|p| p.ticket_num == redis_payment2.ticket_number);
    
    assert!(payment1.is_some(), "Pago 1 no encontrado");
    assert!(payment2.is_some(), "Pago 2 no encontrado");
    
    let p1 = payment1.unwrap();
    assert_eq!(p1.total_amount, 150.0);
    assert_eq!(p1.state, PaymentStatus::Accepted);
    
    let p2 = payment2.unwrap();
    assert_eq!(p2.total_amount, 250.0);
    assert_eq!(p2.state, PaymentStatus::OnRevision);
}

#[test]
fn test_get_user_history_returns_correct_values() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto y guard para limpieza
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());
    
    // Crear un usuario con historial
    let test_access_token = "test_user_history_001";
    let db_access_token = hashing_composite_key(&[&test_access_token.to_string()]);
    
    let payed_to_capital_value = 1500.50;
    let owed_capital_value = 3200.75;
    
    // Insertar valores de historial en Redis
    {
        let mut con = context.pool.get().expect("Couldn't connect to pool");
        
        let payed_key = format!("users:{}:payed_to_capital", db_access_token);
        let owed_key = format!("users:{}:owed_capital", db_access_token);
        
        let _: () = con.set(&payed_key, payed_to_capital_value.to_string())
            .expect("Failed to set payed_to_capital");
        let _: () = con.set(&owed_key, owed_capital_value.to_string())
            .expect("Failed to set owed_capital");
        
        guard.register_key(payed_key);
        guard.register_key(owed_key);
    }
    
    // Ejecutar get_user_history
    let repo = PaymentRepo {
        pool: context.pool.clone(),
    };
    
    let result = repo.get_user_history(test_access_token.to_string())
        .expect("get_user_history failed");
    
    // Validaciones
    assert_eq!(result.payed_to_capital, payed_to_capital_value, 
        "payed_to_capital debería ser {}", payed_to_capital_value);
    assert_eq!(result.owed_capital, owed_capital_value,
        "owed_capital debería ser {}", owed_capital_value);
}

#[test]
fn test_get_user_history_with_no_data_returns_error() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto
    let context = create_test_context();
    
    // Usuario sin datos de historial
    let test_access_token = "test_user_no_history";
    
    // Ejecutar get_user_history
    let repo = PaymentRepo {
        pool: context.pool.clone(),
    };
    
    let result = repo.get_user_history(test_access_token.to_string());
    
    // Validación: debería fallar porque no hay datos
    assert!(result.is_err(), "Debería retornar error cuando no hay datos de historial");
    assert_eq!(result.unwrap_err(), "Couldnt Get Payed To Capital");
}

// Helpers ahora importados desde tests/utils/redis_helpers.rs
