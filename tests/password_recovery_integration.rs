use dotenv::dotenv;
use rand::{
    distr::Alphanumeric,
    Rng,
};
use rand::rng;

use general_api::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    repos::auth::{
        configure_all_security_answers, create_user_with_access_token,
        get_user_access_token, reset_password, utils::hashing_composite_key,
        validate_security_answer,
    },
};
use redis::Commands;

fn cleanup_test_user(username: &str) {
    let mut con = get_pool_connection().into_inner().get().unwrap();

    let access_token =
        hashing_composite_key(&[&username.to_string(), &"InitialPassword123".to_string()]);
    let db_access_token = hashing_composite_key(&[&access_token]);
    let affiliate_key = hashing_composite_key(&[&username.to_string()]);

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
        println!("Cleaning up {} => {:?}", clave, del_result);
    }

    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// INTEGRATION TEST: Complete Password Recovery Flow
/// 
/// This test covers the entire password recovery workflow:
/// 1. Create a new user
/// 2. Configure security answers
/// 3. Validate security answers
/// 4. Reset password using security answer
/// 5. Verify login with new password works
/// 6. Verify login with old password fails
#[test]
fn test_complete_password_recovery_flow() {
    let _ = dotenv();

    let username = format!("recovery_test_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let initial_password = "InitialPassword123".to_string();
    let new_password = "NewPassword456".to_string();
    let full_name = "Test User Recovery".to_string();

    println!("\n=== STARTING PASSWORD RECOVERY INTEGRATION TEST ===");
    println!("Username: {}", username);

    // Cleanup before test
    cleanup_test_user(&username);

    // STEP 1: Create user
    println!("\n[STEP 1] Creating user...");
    let creation = create_user_with_access_token(
        username.clone(),
        initial_password.clone(),
        full_name.clone(),
    );
    assert!(creation.is_ok(), "User creation should succeed");
    println!("✓ User created successfully");
    if let Ok(ref token_info) = creation {
        println!("  Username: {}, User type: {}", token_info.user_name, token_info.user_type);
    }

    // STEP 2: Get access token for configuration
    println!("\n[STEP 2] Getting access token for security configuration...");
    let token_result = get_user_access_token(username.clone(), initial_password.clone());
    assert!(token_result.is_ok(), "Should get access token with initial password");
    let initial_token = token_result.unwrap().access_token;
    println!("✓ Access token obtained: {}", &initial_token[..32]);

    // STEP 3: Configure security answers
    println!("\n[STEP 3] Configuring security answers...");
    let answers = [
        "My favorite color is blue".to_string(),
        "I was born in the city".to_string(),
        "My first pet was a dog".to_string(),
    ];
    let config_result = configure_all_security_answers(initial_token.clone(), answers.clone());
    assert!(config_result.is_ok(), "Security answers configuration should succeed");
    println!("✓ Security answers configured successfully");

    // STEP 4: Validate all security answers
    println!("\n[STEP 4] Validating security answers...");
    for (index, answer) in answers.iter().enumerate() {
        println!("  Validating answer at index {}...", index);
        let validate_result =
            validate_security_answer(username.clone(), index as u8, answer.clone());
        assert!(validate_result.is_ok(), "Validation of answer {} should succeed", index);
        println!("  ✓ Answer {} validated successfully", index);
    }
    println!("✓ All security answers validated");

    // STEP 5: Reset password using security answer
    println!("\n[STEP 5] Resetting password using security answer...");
    let reset_result = reset_password(
        username.clone(),
        0, // Using first security question
        answers[0].clone(),
        new_password.clone(),
    );
    assert!(reset_result.is_ok(), "Password reset should succeed");
    let token_after_reset = reset_result.unwrap().access_token;
    println!("✓ Password reset successfully");
    println!("  New access token: {}", &token_after_reset[..32]);

    // STEP 6: Verify new password works for login
    println!("\n[STEP 6] Verifying login with new password...");
    let new_login = get_user_access_token(username.clone(), new_password.clone());
    assert!(new_login.is_ok(), "Login with new password should succeed");
    println!("✓ Login with new password succeeded");
    println!("  New access token obtained: {}", &new_login.unwrap().access_token[..32]);

    // STEP 7: Verify old password no longer works
    println!("\n[STEP 7] Verifying old password no longer works...");
    let old_login = get_user_access_token(username.clone(), initial_password.clone());
    assert!(old_login.is_err(), "Login with old password should fail");
    println!("✓ Old password correctly rejected");
    if let Err(err) = old_login {
        println!("  Error: {}", err.message);
    }

    // STEP 8: Verify security answers still work after reset
    println!("\n[STEP 8] Verifying security answers still work after reset...");
    for (index, answer) in answers.iter().enumerate() {
        let validate_result =
            validate_security_answer(username.clone(), index as u8, answer.clone());
        assert!(validate_result.is_ok(), "Validation of answer {} should still work", index);
        println!("  ✓ Answer {} still valid after password reset", index);
    }
    println!("✓ All security answers still valid");

    // Cleanup after test
    cleanup_test_user(&username);

    println!("\n=== PASSWORD RECOVERY INTEGRATION TEST PASSED ✓ ===\n");
}

/// INTEGRATION TEST: Multiple password resets in sequence
/// 
/// Tests that a user can perform multiple password resets using different security questions
#[test]
fn test_multiple_sequential_password_resets() {
    let _ = dotenv();

    let username = format!("multi_reset_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password_1 = "Password001".to_string();
    let password_2 = "Password002".to_string();
    let password_3 = "Password003".to_string();

    println!("\n=== MULTIPLE PASSWORD RESETS TEST ===");
    println!("Username: {}", username);

    cleanup_test_user(&username);

    // Create user
    println!("\n[1] Creating user with password: {}", password_1);
    let creation = create_user_with_access_token(
        username.clone(),
        password_1.clone(),
        "Multi Reset User".to_string(),
    );
    assert!(creation.is_ok());
    println!("✓ User created");

    // Configure security answers
    let answers = [
        "Answer to question 1".to_string(),
        "Answer to question 2".to_string(),
        "Answer to question 3".to_string(),
    ];
    let token_1 = get_user_access_token(username.clone(), password_1.clone())
        .unwrap()
        .access_token;
    let _ = configure_all_security_answers(token_1, answers.clone());
    println!("✓ Security answers configured");

    // First reset using question 0
    println!("\n[2] First password reset: {} -> {}", password_1, password_2);
    let reset_1 = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        password_2.clone(),
    );
    assert!(reset_1.is_ok());
    println!("✓ First reset successful");

    // Verify password_1 doesn't work
    let old_login = get_user_access_token(username.clone(), password_1.clone());
    assert!(old_login.is_err());
    println!("✓ Old password (1) correctly rejected");

    // Verify password_2 works
    let new_login = get_user_access_token(username.clone(), password_2.clone());
    assert!(new_login.is_ok());
    println!("✓ New password (2) works");

    // Second reset using question 1
    println!("\n[3] Second password reset: {} -> {}", password_2, password_3);
    let reset_2 = reset_password(
        username.clone(),
        1,
        answers[1].clone(),
        password_3.clone(),
    );
    assert!(reset_2.is_ok());
    println!("✓ Second reset successful");

    // Verify password_2 doesn't work
    let old_login_2 = get_user_access_token(username.clone(), password_2.clone());
    assert!(old_login_2.is_err());
    println!("✓ Previous password (2) correctly rejected");

    // Verify password_3 works
    let new_login_2 = get_user_access_token(username.clone(), password_3.clone());
    assert!(new_login_2.is_ok());
    println!("✓ Latest password (3) works");

    cleanup_test_user(&username);

    println!("\n=== MULTIPLE RESETS TEST PASSED ✓ ===\n");
}

/// INTEGRATION TEST: Error handling in password recovery
/// 
/// Tests that the system properly rejects invalid attempts
#[test]
fn test_password_recovery_error_handling() {
    let _ = dotenv();

    let username = format!("error_test_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "TestPassword789".to_string();

    println!("\n=== ERROR HANDLING TEST ===");
    println!("Username: {}", username);

    cleanup_test_user(&username);

    // Create user
    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Error Test User".to_string(),
    );
    assert!(creation.is_ok());
    println!("✓ User created");

    // Configure answers
    let answers = [
        "Correct answer 1".to_string(),
        "Correct answer 2".to_string(),
        "Correct answer 3".to_string(),
    ];
    let token = get_user_access_token(username.clone(), password.clone())
        .unwrap()
        .access_token;
    let _ = configure_all_security_answers(token, answers.clone());
    println!("✓ Security answers configured");

    // TEST 1: Wrong answer should fail
    println!("\n[TEST 1] Attempting reset with wrong answer...");
    let wrong_answer_reset = reset_password(
        username.clone(),
        0,
        "Wrong answer".to_string(),
        "NewPassword".to_string(),
    );
    assert!(wrong_answer_reset.is_err(), "Reset with wrong answer should fail");
    println!("✓ Wrong answer correctly rejected");

    // TEST 2: Valid answer should still work after failed attempt
    println!("\n[TEST 2] Verifying valid answer still works after failed attempt...");
    let valid_reset = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        "ValidNewPassword".to_string(),
    );
    assert!(valid_reset.is_ok(), "Valid answer should work after failed attempt");
    println!("✓ Valid answer still works");

    // TEST 3: Old password doesn't work anymore
    println!("\n[TEST 3] Verifying old password no longer works...");
    let old_password_login = get_user_access_token(username.clone(), password.clone());
    assert!(old_password_login.is_err());
    println!("✓ Old password correctly rejected");

    // TEST 4: New password works
    println!("\n[TEST 4] Verifying new password works...");
    let new_password_login = get_user_access_token(username.clone(), "ValidNewPassword".to_string());
    assert!(new_password_login.is_ok());
    println!("✓ New password works");

    cleanup_test_user(&username);

    println!("\n=== ERROR HANDLING TEST PASSED ✓ ===\n");
}
