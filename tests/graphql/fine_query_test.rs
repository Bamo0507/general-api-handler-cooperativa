// Tests for SCRUM-253: fines query integrity and mutation flow

use general_api::endpoints::handlers::graphql::fine::{FineMutation, FineQuery};
use general_api::models::graphql::{Fine, FineStatus};
// Shared helpers and guard utilities
include!("common/mod.rs");
use general_api::repos::auth::utils::hashing_composite_key;

// Local lock to serialize Redis operations across test runs
use std::sync::{Mutex, OnceLock};
fn redis_test_lock() -> &'static Mutex<()> {
    static REDIS_TEST_LOCAL: OnceLock<Mutex<()>> = OnceLock::new();
    REDIS_TEST_LOCAL.get_or_init(|| Mutex::new(()))
}

#[test]
fn test_query_returns_inserted_fines_and_preserves_fields() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let access_token = "scrum253_user";
    let fine1 = Fine {
        id: "fine_scrum253_1".to_string(),
        amount: 150.25,
        status: FineStatus::Unpaid,
        reason: "Pago atrasado".to_string(),
    };
    let fine2 = Fine {
        id: "fine_scrum253_2".to_string(),
        amount: 87.0,
        status: FineStatus::Paid,
        reason: "Amonestacion".to_string(),
    };

    let key1 = insert_fine_helper_and_return(&context, access_token, &fine1);
    let key2 = insert_fine_helper_and_return(&context, access_token, &fine2);
    guard.register_key(key1);
    guard.register_key(key2);

    let result = futures::executor::block_on(async {
        FineQuery::get_fines_by_id(&context, access_token.to_string()).await
    })
    .expect("Fine query failed");

    assert!(result.len() >= 2, "Expected at least two fines in query result");

    let fetched1 = result.iter().find(|f| f.reason == fine1.reason).expect("fine1 not found");
    assert_eq!(fetched1.amount, fine1.amount);
    assert_eq!(fetched1.status, fine1.status);

    let fetched2 = result.iter().find(|f| f.reason == fine2.reason).expect("fine2 not found");
    assert_eq!(fetched2.amount, fine2.amount);
    assert_eq!(fetched2.status, fine2.status);
}

#[test]
fn test_query_after_create_fine_mutation_returns_new_fine() {
    let _guard = redis_test_lock().lock().unwrap();
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    let access_token = "scrum253_mut_user".to_string();
    let affiliate_key = "AFF253".to_string();
    let db_access_token = hashing_composite_key(&[&access_token]);
    let mapping_key = format!("affiliate_key_to_db_access:{}", affiliate_key);

    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        let _: () = con
            .set(mapping_key.as_str(), db_access_token.clone())
            .expect("No se pudo insertar el mapping affiliate->db token");
    }
    guard.register_key(mapping_key);

    let motive = "SCRUM253 Fine".to_string();
    let amount = 240.5;
    let mutation_result = futures::executor::block_on(async {
        FineMutation::create_fine(&context, affiliate_key.clone(), amount, motive.clone()).await
    })
    .expect("Fine mutation failed");
    assert_eq!(mutation_result, "Fine Createad");

    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        let pattern = format!("users:{}:fines:*", db_access_token);
        let keys: Vec<String> = con.scan_match(pattern).unwrap().collect();
        assert!(!keys.is_empty(), "Mutation did not create fine keys");
        for key in keys {
            guard.register_key(key);
        }
    }

    let result = futures::executor::block_on(async {
        FineQuery::get_fines_by_id(&context, access_token.clone()).await
    })
    .expect("Fine query tras mutation falló");

    assert!(
        result.iter().any(|f| f.reason == motive && (f.amount - amount).abs() < 1e-6),
        "Query no devolvió la multa creada"
    );
    let inserted = result
        .iter()
        .find(|f| f.reason == motive)
        .expect("No se encontró la multa insertada");
    assert_eq!(inserted.status, FineStatus::Unpaid);
}
