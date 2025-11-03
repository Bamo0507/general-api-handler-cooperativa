// Pruebas unitarias para la query get_user_fines
// Verifica que devuelva el nombre de la persona (presented_by_name)

use general_api::repos::graphql::fine::FineRepo;
use super::common::{create_test_context, TestRedisGuard};
use general_api::test_sync::REDIS_TEST_LOCK;
use general_api::repos::auth::utils::hashing_composite_key;
use redis::{Commands, JsonCommands};

/// Helper para insertar una multa directamente en Redis y retornar la key
fn insert_fine_helper_and_return(
    context: &general_api::endpoints::handlers::configs::schema::GeneralContext,
    fine: &general_api::models::redis::Fine,
    access_token: &str,
) -> String {
    let mut con = context.pool.get().expect("Couldn't connect to pool");
    let db_access_token = hashing_composite_key(&[&access_token.to_string()]);
    
    // Crear la key de la multa
    let fine_key = format!("users:{}:fines:{}", db_access_token, fine.status);
    
    // Insertar la multa en Redis como JSON
    let _: () = con.json_set(&fine_key, "$", fine).expect("Failed to insert fine");
    
    fine_key
}

#[test]
fn test_get_user_fines_includes_presented_by_name() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto y guard para limpieza
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());
    
    // Crear usuario de prueba con nombre completo
    let test_access_token = "test_user_fine_1";
    let db_access_token = hashing_composite_key(&[&test_access_token.to_string()]);
    let user_name = "Juan Pérez López";
    
    // Insertar nombre del usuario en Redis
    {
        let mut con = context.pool.get().expect("Couldn't connect to pool");
        let name_key = format!("users:{}:complete_name", db_access_token);
        let _: () = con.set(&name_key, user_name).expect("Failed to set user name");
        guard.register_key(name_key);
    }
    
    // Crear multas de prueba
    use general_api::models::redis::Fine as RedisFine;
    
    let fine1 = RedisFine {
        amount: 50.0,
        motive: "Llegó tarde a reunión".to_string(),
        status: "UNPAID".to_string(),
    };
    
    let fine2 = RedisFine {
        amount: 100.0,
        motive: "Falta injustificada".to_string(),
        status: "PAID".to_string(),
    };
    
    // Insertar las multas
    let key1 = insert_fine_helper_and_return(&context, &fine1, test_access_token);
    let key2 = insert_fine_helper_and_return(&context, &fine2, test_access_token);
    guard.register_key(key1.clone());
    guard.register_key(key2.clone());
    
    // Verificar que las claves existen
    {
        let mut con = context.pool.get().expect("Couldn't connect to pool");
        assert!(con.exists::<_, bool>(&key1).unwrap(), "Fine 1 no existe en Redis");
        assert!(con.exists::<_, bool>(&key2).unwrap(), "Fine 2 no existe en Redis");
    }
    
    // Ejecutar get_user_fines
    let repo = FineRepo {
        pool: context.pool.clone(),
    };
    
    let result = repo.get_user_fines(test_access_token.to_string())
        .expect("get_user_fines failed");
    
    // Validaciones
    assert_eq!(result.len(), 2, "Deberían haber 2 multas");
    
    // Verificar que todas las multas tienen el presented_by_name correcto
    for fine in result.iter() {
        assert_eq!(
            fine.presented_by_name, 
            user_name,
            "El presented_by_name debería ser '{}'", 
            user_name
        );
        
        // Verificar que los demás campos existen y tienen sentido
        assert!(!fine.id.is_empty(), "El ID no debería estar vacío");
        assert!(fine.amount > 0.0, "El amount debería ser mayor a 0");
        assert!(!fine.reason.is_empty(), "El reason no debería estar vacío");
    }
}

#[test]
fn test_get_user_fines_with_missing_name_returns_default() {
    // Serializar pruebas que tocan Redis
    let _guard = REDIS_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap();
    
    // Crear contexto y guard para limpieza
    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());
    
    // Crear usuario SIN nombre completo en Redis
    let test_access_token = "test_user_fine_no_name";
    
    // Crear multa de prueba
    use general_api::models::redis::Fine as RedisFine;
    
    let fine = RedisFine {
        amount: 75.0,
        motive: "Multa de prueba".to_string(),
        status: "UNPAID".to_string(),
    };
    
    // Insertar la multa (pero NO el nombre del usuario)
    let key = insert_fine_helper_and_return(&context, &fine, test_access_token);
    guard.register_key(key);
    
    // Ejecutar get_user_fines
    let repo = FineRepo {
        pool: context.pool.clone(),
    };
    
    let result = repo.get_user_fines(test_access_token.to_string())
        .expect("get_user_fines failed");
    
    // Validar que devuelve el nombre por defecto
    assert_eq!(result.len(), 1, "Debería haber 1 multa");
    assert_eq!(
        result[0].presented_by_name,
        "Nombre no encontrado",
        "Debería devolver el nombre por defecto cuando no existe"
    );
}
