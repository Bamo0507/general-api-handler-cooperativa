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
            configure_all_security_answers, validate_security_answer, reset_password,
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
        format!("users:{}:security_answer_0", db_access_token),
        format!("users:{}:security_answer_1", db_access_token),
        format!("users:{}:security_answer_2", db_access_token),
    ];
    for clave in claves {
        let del_result: Result<(), _> = con.del(&clave);
        println!("Eliminando {} => {:?}", clave, del_result);
    }

    // Esperar un poco para asegurar que Redis procese la eliminación
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// Helper function to seed security questions for test users
/// Creates 5 test users with 3 security questions/answers each
fn seed_security_questions() -> Vec<(String, String, [String; 3])> {
    let mut test_users = Vec::new();
    for i in 1..=5 {
        let username = format!("security_test_user_{}", i);
        let password = "ElTestoPaga".to_string();
        let answers = [
            format!("test_answer_{}_0", i),
            format!("test_answer_{}_1", i),
            format!("test_answer_{}_2", i),
        ];
        cleanup_test_user(&username);
        let creation_result = create_user_with_access_token(
            username.clone(),
            password.clone(),
            format!("Test User {}", i),
        );
        if let Ok(token_info) = creation_result {
            let config_result = configure_all_security_answers(
                token_info.access_token.clone(),
                answers.clone(),
            );
            if let Err(e) = &config_result {
                println!("Failed to configure security answers for {}: {:?}", username, e);
            }
            test_users.push((username, token_info.access_token, answers));
        } else {
            println!("Failed to create user {}: {:?}", username, creation_result.err());
        }
    }
    test_users
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
    
    assert!(creation_result.is_ok(), "Should create test user: {:?}", creation_result.err());

    // Ahora obtener el token de acceso
    let access_token = get_user_access_token(
        username.to_string(),
        password.to_string(),
    );

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
    
    let _: () = con.set(format!("users:{}:owed_capital", db_acess_token), 10101.0)
        .expect("Should set owed_capital");
    let _: () = con.set(format!("users:{}:payed_to_capital", db_acess_token), 1010.0)
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

/// Test for validating security answer - correct answer path
#[test]
fn test_validate_security_answer_correct() {
    let _ = dotenv();
    
    let test_users = seed_security_questions();
    assert!(!test_users.is_empty(), "Should have seeded test users");
    
    let (username, _access_token, answers) = test_users.first().unwrap();
    // Validate with correct answer from index 0
    let result = validate_security_answer(username.clone(), 0, answers[0].clone());
    assert!(result.is_ok(), "Should validate correct answer: {:?}", result.err());
    
    // The result should be the db_composite_key
    let _db_composite_key = result.unwrap();
    assert!(!_db_composite_key.is_empty(), "db_composite_key should not be empty");
    
    // Cleanup
    cleanup_test_user(username);
}

/// Test for validating security answer - incorrect answer path
#[test]
fn test_validate_security_answer_incorrect() {
    let _ = dotenv();
    
    let test_users = seed_security_questions();
    assert!(!test_users.is_empty(), "Should have seeded test users");
    
    let (username, _access_token, _answers) = test_users.first().unwrap();
    let wrong_answer = "completely_wrong_answer";
    
    // Validate with incorrect answer
    let result = validate_security_answer(username.clone(), 0, wrong_answer.to_string());
    assert!(result.is_err(), "Should reject incorrect answer");
    
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.message.contains("Respuesta incorrecta") || error_msg.message.contains("incorrecta"),
        "Should return incorrect answer message, got: {}",
        error_msg.message
    );
    
    // Cleanup
    cleanup_test_user(username);
}

/// Test for reset password - successful reset path
#[test]
fn test_reset_password_success() {
    let _ = dotenv();
    
    let test_users = seed_security_questions();
    assert!(!test_users.is_empty(), "Should have seeded test users");
    
    let (username, _access_token, answers) = test_users.first().unwrap();
    let new_password = "NewPassword123";
    // Get original token for comparison
    let original_token = get_user_access_token(
        username.clone(),
        "ElTestoPaga".to_string(),
    ).expect("Should get original token");
    // Reset password with question_index 0
    let result = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        new_password.to_string(),
    );
    assert!(result.is_ok(), "Should reset password successfully: {:?}", result.err());
    let new_token_info = result.unwrap();
    assert!(!new_token_info.access_token.is_empty(), "Should return new access_token");
    // Verify new token is different from old token
    assert_ne!(
        original_token.access_token,
        new_token_info.access_token,
        "New token should be different from old token"
    );
    // Verify user can login with new password
    let new_login_result = get_user_access_token(
        username.clone(),
        new_password.to_string(),
    );
    assert!(new_login_result.is_ok(), "Should be able to login with new password");
    let new_login_token = new_login_result.unwrap();
    assert_eq!(
        new_login_token.access_token,
        new_token_info.access_token,
        "New login token should match reset password token"
    );
    // Cleanup
    cleanup_test_user(username);
}

/// Test for reset password without configured security question
#[test]
fn test_reset_password_without_question() {
    let _ = dotenv();
    
    // Create a user without security question
    let username = format!("no_security_user_{}", Alphanumeric.sample_string(&mut rng(), 8));
    let password = "ElTestoPaga".to_string();
    
    cleanup_test_user(&username);
    
    let creation_result = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Test User No Question".to_string(),
    );
    
    assert!(creation_result.is_ok(), "Should create test user");
    
    // Try to reset password without configuring security question
    let result = reset_password(
        username.clone(),
        0,
        "some_answer".to_string(),
        "NewPassword".to_string(),
    );
    
    assert!(result.is_err(), "Should fail when security question not configured");
    
    // Cleanup
    cleanup_test_user(&username);
}
