// Pruebas unitarias para la query get_user_fines
// Valida que el campo presented_by_name se enriquezca correctamente

use general_api::models::graphql::{Fine, FineStatus};
use general_api::repos::graphql::fine::FineRepo;
use super::common::{create_test_context, TestRedisGuard};
use general_api::test_sync::REDIS_TEST_LOCK;
use general_api::repos::auth::utils::hashing_composite_key;
use redis::{Commands, JsonCommands};
use actix_web::web::Data;

/// Helper para insertar una multa de prueba en Redis y retornar su key
fn insert_fine_helper_and_return(
    pool: &r2d2::Pool<redis::Client>,
    user_hash: &str,
    fine_id: &str,
    amount: f32,
    motive: &str,
) -> String {
    use general_api::models::redis::Fine as RedisFine;

    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
    let redis_key = format!("users:{}:fines:{}", user_hash, fine_id);

    let redis_fine = RedisFine {
        amount,
        motive: motive.to_string(),
        status: "UNPAID".to_string(),
    };

    let _: redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_fine);
    redis_key
}

#[test]
fn test_get_user_fines_returns_with_presented_by_name() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto y guard para limpieza
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Crear un user_hash único para este test
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let unique_str = format!("test_fine_{}", now);
    let user_hash = hashing_composite_key(&[&unique_str]);

    // Insertar un nombre de usuario completo en Redis para poder validar presented_by_name
    let complete_name = "Juan Pérez Test";
    let user_name_key = format!("users:{}:complete_name", user_hash);
    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        let _: redis::RedisResult<()> = con.set(&user_name_key, complete_name);
        guard.register_key(user_name_key.clone());
    }

    // Insertar multas de prueba
    let fine_id_1 = format!("fine_{}_1", now);
    let fine_id_2 = format!("fine_{}_2", now);

    let key1 = insert_fine_helper_and_return(
        &context.pool,
        &user_hash,
        &fine_id_1,
        100.0,
        "Multa test 1",
    );
    guard.register_key(key1);

    let key2 = insert_fine_helper_and_return(
        &context.pool,
        &user_hash,
        &fine_id_2,
        200.0,
        "Multa test 2",
    );
    guard.register_key(key2);

    // Crear el FineRepo y llamar a get_user_fines
    // Necesitamos simular el access_token que sería el user_hash
    let fine_repo = FineRepo {
        pool: context.pool.clone(),
    };

    // get_user_fines espera un access_token, pero internamente usa get_db_access_token_with_affiliate_key
    // Para simplificar, podemos usar el user_hash directamente si modificamos temporalmente,
    // o mejor aún, insertar una clave de affiliate para simular el flujo completo.
    // Por simplicidad en este test, vamos a insertar la estructura esperada:
    
    // Insertar affiliate key que mapea a nuestro user_hash
    let affiliate_key = format!("test_affiliate_{}", now);
    let affiliate_redis_key = format!("users:{}:affiliate_key", user_hash);
    let affiliate_to_db_key = format!("affiliate_key_to_db_access:{}", affiliate_key);
    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        // Guardar users:{hash}:affiliate_key
        let _: redis::RedisResult<()> = con.set(&affiliate_redis_key, &affiliate_key);
        guard.register_key(affiliate_redis_key.clone());
        // Guardar affiliate_key_to_db_access:{affiliate_key} -> user_hash (esto es lo que faltaba)
        let _: redis::RedisResult<()> = con.set(&affiliate_to_db_key, &user_hash);
        guard.register_key(affiliate_to_db_key.clone());
    }

    // Ahora get_user_fines debería poder encontrar las multas usando affiliate_key como access_token
    let result = fine_repo.get_user_fines(affiliate_key);

    assert!(result.is_ok(), "get_user_fines should succeed");
    let fines = result.unwrap();

    // Validar que las multas se recuperaron
    assert_eq!(fines.len(), 2, "Should have 2 fines");

    // Validar que presented_by_name se enriqueció correctamente
    for fine in fines.iter() {
        assert_eq!(
            fine.presented_by_name, complete_name,
            "presented_by_name should be enriched with the complete name: {}",
            complete_name
        );
        assert!(fine.amount > 0.0, "Fine amount should be positive");
        assert!(!fine.reason.is_empty(), "Fine reason should not be empty");
    }
}

#[test]
fn test_get_user_fines_defaults_to_na_when_no_user_name() {
    // Test que valida que cuando no existe users:{hash}:complete_name, 
    // el presented_by_name sea "N/A"
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());

    // Crear un user_hash único
    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let unique_str = format!("test_fine_na_{}", now);
    let user_hash = hashing_composite_key(&[&unique_str]);

    // NO insertar users:{hash}:complete_name - esto debe resultar en "N/A"

    // Insertar una multa
    let fine_id = format!("fine_{}_na", now);
    let key = insert_fine_helper_and_return(
        &context.pool,
        &user_hash,
        &fine_id,
        150.0,
        "Multa sin nombre",
    );
    guard.register_key(key);

    // Insertar affiliate key
    let affiliate_key = format!("test_affiliate_na_{}", now);
    let affiliate_redis_key = format!("users:{}:affiliate_key", user_hash);
    let affiliate_to_db_key = format!("affiliate_key_to_db_access:{}", affiliate_key);
    {
        let mut con = context.pool.get().expect("No se pudo obtener conexión de Redis");
        // Guardar users:{hash}:affiliate_key
        let _: redis::RedisResult<()> = con.set(&affiliate_redis_key, &affiliate_key);
        guard.register_key(affiliate_redis_key.clone());
        // Guardar affiliate_key_to_db_access:{affiliate_key} -> user_hash
        let _: redis::RedisResult<()> = con.set(&affiliate_to_db_key, &user_hash);
        guard.register_key(affiliate_to_db_key.clone());
    }

    let fine_repo = FineRepo {
        pool: context.pool.clone(),
    };

    let result = fine_repo.get_user_fines(affiliate_key);
    
    assert!(result.is_ok(), "get_user_fines should succeed even without complete_name");
    let fines = result.unwrap();

    assert_eq!(fines.len(), 1, "Should have 1 fine");
    assert_eq!(
        fines[0].presented_by_name, "N/A",
        "presented_by_name should default to N/A when no complete_name exists"
    );
}
