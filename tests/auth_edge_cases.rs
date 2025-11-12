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
        hashing_composite_key(&[&username.to_string(), &"ElTestoPaga".to_string()]);
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
        println!("Eliminando {} => {:?}", clave, del_result);
    }

    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// TEST EDGE CASE 1: Validate with out-of-bounds question_index
#[test]
fn test_validate_with_out_of_bounds_index() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation_result = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case Test User".to_string(),
    );
    assert!(creation_result.is_ok(), "Should create test user");

    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let config_result = configure_all_security_answers(access_token, answers.clone());
    assert!(config_result.is_ok(), "Should configure security answers");

    let result = validate_security_answer(username.clone(), 3, "answer_0".to_string());
    println!("Out-of-bounds result: {:?}", result);
    assert!(result.is_err(), "Should fail with out-of-bounds index");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 2: Validate with question_index = 255 (max u8)
#[test]
fn test_validate_with_max_u8_index() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation_result = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case Test User".to_string(),
    );
    assert!(creation_result.is_ok());

    let answers = ["answer_0".to_string(), "answer_1".to_string(), "answer_2".to_string()];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers.clone());

    let result = validate_security_answer(username.clone(), 255, "answer_0".to_string());
    println!("Max u8 result: {:?}", result);
    assert!(result.is_err(), "Should fail with index 255");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 3: Multiple consecutive password resets
#[test]
fn test_multiple_consecutive_resets() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password_1 = "ElTestoPaga";
    let password_2 = "NewPassword2";
    let password_3 = "NewPassword3";

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password_1.to_string(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password_1.to_string()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers.clone());

    let reset_1 =
        reset_password(username.clone(), 0, answers[0].clone(), password_2.to_string());
    assert!(reset_1.is_ok(), "First reset should succeed");

    let login_1 = get_user_access_token(username.clone(), password_2.to_string());
    assert!(login_1.is_ok(), "Should login with password_2");

    let reset_2 =
        reset_password(username.clone(), 1, answers[1].clone(), password_3.to_string());
    assert!(reset_2.is_ok(), "Second reset should succeed");

    let login_2 = get_user_access_token(username.clone(), password_3.to_string());
    assert!(login_2.is_ok(), "Should login with password_3");

    let old_login = get_user_access_token(username.clone(), password_1.to_string());
    assert!(old_login.is_err(), "Should NOT login with old password_1");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 4: Validate with different indices for same user
#[test]
fn test_validate_all_three_indices() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = ["answer_0".to_string(), "answer_1".to_string(), "answer_2".to_string()];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers.clone());

    let result_0 = validate_security_answer(username.clone(), 0, answers[0].clone());
    println!("Index 0 validation: {:?}", result_0);
    assert!(result_0.is_ok(), "Should validate index 0");

    let result_1 = validate_security_answer(username.clone(), 1, answers[1].clone());
    println!("Index 1 validation: {:?}", result_1);
    assert!(result_1.is_ok(), "Should validate index 1");

    let result_2 = validate_security_answer(username.clone(), 2, answers[2].clone());
    println!("Index 2 validation: {:?}", result_2);
    assert!(result_2.is_ok(), "Should validate index 2");

    let key_0 = result_0.unwrap();
    let key_1 = result_1.unwrap();
    let key_2 = result_2.unwrap();
    assert_eq!(key_0, key_1, "Index 0 and 1 should return same key");
    assert_eq!(key_1, key_2, "Index 1 and 2 should return same key");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 5: Wrong answer with correct index
#[test]
fn test_wrong_answer_correct_index() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = [
        "correct_answer_0".to_string(),
        "correct_answer_1".to_string(),
        "correct_answer_2".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers);

    let result = validate_security_answer(username.clone(), 0, "wrong_answer".to_string());
    println!("Wrong answer with correct index: {:?}", result);
    assert!(result.is_err(), "Should fail with wrong answer");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 6: Correct answer with wrong index
#[test]
fn test_correct_answer_wrong_index() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers.clone());

    let result = validate_security_answer(username.clone(), 1, answers[0].clone());
    println!("Correct answer with wrong index: {:?}", result);
    assert!(result.is_err(), "Should fail when answer is at wrong index");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 7: Case sensitivity and normalization
#[test]
fn test_case_sensitivity_normalization() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = [
        "lowercase_answer".to_string(),
        "another_answer".to_string(),
        "third_answer".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers);

    let result_upper =
        validate_security_answer(username.clone(), 0, "LOWERCASE_ANSWER".to_string());
    println!("Uppercase validation: {:?}", result_upper);
    assert!(result_upper.is_ok(), "Should normalize case and match");

    let result_mixed =
        validate_security_answer(username.clone(), 0, "LowerCase_Answer".to_string());
    println!("Mixed case validation: {:?}", result_mixed);
    assert!(result_mixed.is_ok(), "Should normalize and match");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 8: Reset preserves all 3 answers
#[test]
fn test_reset_preserves_all_answers() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let original_password = "OriginalPassword";
    let new_password = "NewPassword";

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        original_password.to_string(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let original_answers = [
        "original_answer_0".to_string(),
        "original_answer_1".to_string(),
        "original_answer_2".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), original_password.to_string())
            .unwrap()
            .access_token;
    let _ = configure_all_security_answers(access_token, original_answers.clone());

    let reset = reset_password(
        username.clone(),
        0,
        original_answers[0].clone(),
        new_password.to_string(),
    );
    assert!(reset.is_ok(), "Reset should succeed");

    let validate_0 =
        validate_security_answer(username.clone(), 0, original_answers[0].clone());
    println!("Post-reset validation with answer 0: {:?}", validate_0);
    assert!(validate_0.is_ok(), "Answer 0 should still work after reset");

    let validate_1 =
        validate_security_answer(username.clone(), 1, original_answers[1].clone());
    println!("Post-reset validation with answer 1: {:?}", validate_1);
    assert!(validate_1.is_ok(), "Answer 1 should still work after reset");

    let validate_2 =
        validate_security_answer(username.clone(), 2, original_answers[2].clone());
    println!("Post-reset validation with answer 2: {:?}", validate_2);
    assert!(validate_2.is_ok(), "Answer 2 should still work after reset");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 9: Answer with spaces - normalization
#[test]
fn test_answer_with_spaces() {
    let _ = dotenv();

    let username =
        format!("edge_case_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let password = "ElTestoPaga".to_string();

    cleanup_test_user(&username);

    let creation = create_user_with_access_token(
        username.clone(),
        password.clone(),
        "Edge Case User".to_string(),
    );
    assert!(creation.is_ok());

    let answers = [
        "  answer with spaces  ".to_string(),
        "another answer".to_string(),
        "third".to_string(),
    ];
    let access_token =
        get_user_access_token(username.clone(), password.clone()).unwrap().access_token;
    let _ = configure_all_security_answers(access_token, answers);

    let result =
        validate_security_answer(username.clone(), 0, "   answer with spaces   ".to_string());
    println!("Answer with spaces validation: {:?}", result);
    assert!(result.is_ok(), "Should normalize spaces and match");

    cleanup_test_user(&username);
}

/// TEST EDGE CASE 10: Non-existent user
#[test]
fn test_validate_nonexistent_user() {
    let _ = dotenv();

    let nonexistent_user = "this_user_definitely_does_not_exist_12345";

    let result =
        validate_security_answer(nonexistent_user.to_string(), 0, "some_answer".to_string());
    println!("Non-existent user validation: {:?}", result);
    assert!(result.is_err(), "Should fail for non-existent user");
}
