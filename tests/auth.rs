use dotenv::dotenv;
// This tests assume that the redis db is running and the env variables are declare on the cli
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

use general_api::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    repos::{
        auth::{
            create_user_with_access_token, get_user_access_token, utils::hashing_composite_key,
        },
        graphql::payment::PaymentRepo,
    },
};
use redis::Commands;

// Helper function para limpiar datos de usuario de prueba
fn cleanup_test_user(username: &str) {
    let mut con = get_pool_connection().into_inner().get().unwrap();

    // Generar las claves que usa este usuario específico
    let access_token = hashing_composite_key(&[&username.to_string(), &"ElTestoPaga".to_string()]);
    let db_access_token = hashing_composite_key(&[&access_token]);
    let affiliate_key = hashing_composite_key(&[&username.to_string()]);

    // Eliminar todas las claves relacionadas
    let claves = vec![
        format!("users_on_used:{}", username),
        format!("users:{}:complete_name", db_access_token),
        format!("users:{}:affiliate_key", db_access_token),
        format!("affiliate_keys:{}", affiliate_key),
        format!("users:{}:payed_to_capital", db_access_token),
        format!("users:{}:owed_capital", db_access_token),
        format!("users:{}:is_directive", db_access_token),
        format!("users:{}:payments", db_access_token),
        format!("users:{}:loans", db_access_token),
    ];
    for clave in claves {
        let del_result: Result<(), _> = con.del(&clave);
        println!("Eliminando {} => {:?}", clave, del_result);
    }

    // Esperar un poco para asegurar que Redis procese la eliminación
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// Tests for checking login function integrity
/// For for getting acess_token with the given credentials in Redis DB (Login)
#[test]
fn from_credentials_to_acess_token() {
    let _ = dotenv();

    let username = "El_Mago_Pero_Del_Test";
    let password = "ElTestoPaga";

    // Limpiar datos previos del usuario de prueba
    cleanup_test_user(username);

    // Primero crear el usuario para el test
    let creation_result = create_user_with_access_token(
        username.to_string(),
        password.to_string(),
        "Test User Complete Name".to_string(),
    );

    assert!(
        creation_result.is_ok(),
        "Should create test user: {:?}",
        creation_result.err()
    );

    // Ahora obtener el token de acceso
    let access_token = get_user_access_token(username.to_string(), password.to_string());

    assert_eq!(
        access_token.unwrap().access_token.to_uppercase(),
        "c50329f3e834e2d6a27d0e1a81fc12579aa4570fa889eb302ca192f82961edb0"
            .to_string()
            .to_uppercase()
    );

    // Limpiar después del test
    cleanup_test_user(username);
}

/// For checking if credentials cretead an user instance in Redis DB (Signup)
#[test]
fn from_credentials_to_data() {
    let _ = dotenv();
    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    // random string
    let mut random_string = Alphanumeric.sample_string(&mut rng(), 16);

    // Loop for getting new access token
    let mut access_token = String::new();
    loop {
        access_token = match create_user_with_access_token(
            random_string.clone(),
            random_string.clone(),
            random_string.clone(),
        ) {
            Ok(token_info) => {
                let mut con = get_pool_connection().into_inner().get().unwrap();

                let db_acess_token = hashing_composite_key(&[&token_info.access_token]);
                let _: () = con
                    .set(format!("users:{}:owed_capital", db_acess_token), 10101.0)
                    .expect("Should set owed_capital");
                let _: () = con
                    .set(format!("users:{}:payed_to_capital", db_acess_token), 1010.0)
                    .expect("Should set payed_to_capital");

                token_info.access_token
            }
            Err(_) => {
                // for retrying a new string
                random_string = Alphanumeric.sample_string(&mut rng(), 13);
                "Error Token".to_string()
            }
        };

        // Just breaks the loop until it could create a signup
        if access_token != "Error Token".to_string() {
            break;
        };
    }

    // Pretty similar as the Login test
    assert_eq!(
        10101.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .owed_capital
    );

    assert_eq!(
        1010.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .payed_to_capital
    );
}

//*! 2 Tests go here
/// For getting data from User in Redis DB (Login)
#[test]
fn check_if_can_acess_data() {
    let _ = dotenv();
    // Username aleatorio para evitar colisiones
    let username = format!("testuser_{}", Alphanumeric.sample_string(&mut rng(), 12));
    let passcode = "ElTestoPaga".to_string();

    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    // Limpiar datos previos del usuario de prueba
    cleanup_test_user(&username);

    // Crear el usuario para el test
    let token = create_user_with_access_token(
        username.clone(),
        passcode.clone(),
        "EL Pedro Del Testo".to_string(),
    )
    .expect("Should create test user");

    let access_token = token.access_token;

    // Configurar los valores de capital para el test
    let mut con = get_pool_connection().into_inner().get().unwrap();
    let db_acess_token = hashing_composite_key(&[&access_token]);

    let _: () = con
        .set(format!("users:{}:owed_capital", db_acess_token), 10101.0)
        .expect("Should set owed_capital");
    let _: () = con
        .set(format!("users:{}:payed_to_capital", db_acess_token), 1010.0)
        .expect("Should set payed_to_capital");

    assert_eq!(
        10101.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .owed_capital
    );

    assert_eq!(
        1010.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .payed_to_capital
    );

    // Limpiar después del test
    cleanup_test_user(&username);
}
