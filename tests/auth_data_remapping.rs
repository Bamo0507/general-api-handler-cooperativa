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
    },
    repos::graphql::payment::PaymentRepo,
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
        format!("affiliate_key_to_db_access:{}", affiliate_key),
        format!("users:{}:payed_to_capital", db_access_token),
        format!("users:{}:owed_capital", db_access_token),
        format!("users:{}:is_directive", db_access_token),
        format!("users:{}:payments", db_access_token),
        format!("users:{}:loans", db_access_token),
        format!("users:{}:fines", db_access_token),
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

/// TEST: Verificar que datos financieros (owed_capital, payed_to_capital) se remapiean correctamente
#[test]
fn test_financial_data_remapped_after_reset() {
    let _ = dotenv();

    let username =
        format!("remapping_user_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let original_password = "OriginalPassword";
    let new_password = "NewPassword";

    cleanup_test_user(&username);

    // 1. Crear usuario
    let creation = create_user_with_access_token(
        username.clone(),
        original_password.to_string(),
        "Remapping Test User".to_string(),
    );
    assert!(creation.is_ok(), "Should create user");

    let original_token_info = creation.unwrap();
    let original_access_token = original_token_info.access_token.clone();
    let original_db_key = hashing_composite_key(&[&original_access_token]);

    // 2. Setear datos financieros con el token original
    let mut con = get_pool_connection().into_inner().get().unwrap();
    let original_owed = 5000.0;
    let original_payed = 2000.0;

    let _: () = con
        .set(
            format!("users:{}:owed_capital", &original_db_key),
            original_owed,
        )
        .expect("Should set owed_capital");
    let _: () = con
        .set(
            format!("users:{}:payed_to_capital", &original_db_key),
            original_payed,
        )
        .expect("Should set payed_to_capital");

    // 3. Verificar datos con token original
    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };
    let history_before = repo
        .get_user_history(original_access_token.clone())
        .expect("Should get user history");
    assert_eq!(history_before.owed_capital, original_owed, "Original owed_capital should match");
    assert_eq!(history_before.payed_to_capital, original_payed, "Original payed_to_capital should match");

    // 4. Configurar respuestas de seguridad
    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let config_result = configure_all_security_answers(original_access_token.clone(), answers.clone());
    assert!(config_result.is_ok(), "Should configure security answers");

    // 5. Resetear contraseña
    let reset_result = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        new_password.to_string(),
    );
    assert!(reset_result.is_ok(), "Should reset password");
    
    let new_token_info = reset_result.unwrap();
    let new_access_token = new_token_info.access_token.clone();
    assert_ne!(
        original_access_token, new_access_token,
        "New token should be different from original"
    );

    // 6. Verificar que los datos financieros están disponibles con el nuevo token
    let history_after = repo
        .get_user_history(new_access_token.clone())
        .expect("Should get user history with new token");
    assert_eq!(
        history_after.owed_capital, original_owed,
        "Owed capital should be preserved after reset"
    );
    assert_eq!(
        history_after.payed_to_capital, original_payed,
        "Payed to capital should be preserved after reset"
    );

    // 7. Verificar que el token original NO funciona más
    let old_token_result = repo.get_user_history(original_access_token.clone());
    assert!(
        old_token_result.is_err() || (old_token_result.is_ok() && old_token_result.unwrap().owed_capital == 0.0),
        "Old token should not return the original data (either error or empty)"
    );

    // Cleanup
    cleanup_test_user(&username);
}

/// TEST: Verificar que el mapeo affiliate_key → db_composite_key se actualiza correctamente
#[test]
fn test_affiliate_key_mapping_updated_after_reset() {
    let _ = dotenv();

    let username =
        format!("affiliate_test_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let original_password = "OriginalPassword";
    let new_password = "NewPassword";

    cleanup_test_user(&username);

    // 1. Crear usuario
    let creation = create_user_with_access_token(
        username.clone(),
        original_password.to_string(),
        "Affiliate Mapping Test".to_string(),
    );
    assert!(creation.is_ok());

    let original_token_info = creation.unwrap();
    let original_access_token = original_token_info.access_token.clone();
    let original_db_key = hashing_composite_key(&[&original_access_token]);
    let affiliate_key = hashing_composite_key(&[&username]);

    // 2. Verificar mapeo inicial
    let mut con = get_pool_connection().into_inner().get().unwrap();
    let mapped_db_key: String = con
        .get(format!("affiliate_key_to_db_access:{}", affiliate_key))
        .expect("Should get mapping");
    assert_eq!(
        mapped_db_key, original_db_key,
        "Initial mapping should point to original db_key"
    );

    // 3. Configurar respuestas de seguridad y resetear
    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let _ = configure_all_security_answers(original_access_token.clone(), answers.clone());
    let reset_result = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        new_password.to_string(),
    );
    assert!(reset_result.is_ok());

    let new_token_info = reset_result.unwrap();
    let new_access_token = new_token_info.access_token.clone();
    let new_db_key = hashing_composite_key(&[&new_access_token]);

    // 4. Verificar que el mapeo se actualizó
    let updated_mapped_db_key: String = con
        .get(format!("affiliate_key_to_db_access:{}", affiliate_key))
        .expect("Should get updated mapping");
    assert_eq!(
        updated_mapped_db_key, new_db_key,
        "Mapping should now point to new db_key"
    );
    assert_ne!(
        mapped_db_key, updated_mapped_db_key,
        "Mapping should have changed after reset"
    );

    // 5. Verificar que el nuevo db_key tiene los datos de usuario
    let complete_name: String = con
        .get(format!("users:{}:complete_name", new_db_key))
        .expect("Should get complete name from new db_key");
    assert_eq!(complete_name, "Affiliate Mapping Test");

    // 6. Verificar que el OLD db_key ya no existe (fue eliminado)
    let old_key_exists: bool = con
        .exists(format!("users:{}:complete_name", original_db_key))
        .unwrap_or(false);
    assert!(
        !old_key_exists,
        "Old db_key should be deleted after reset (security: invalidate old token)"
    );

    // Cleanup
    cleanup_test_user(&username);
}

/// TEST: Verificar que las 3 respuestas de seguridad se remapiean y siguen siendo válidas
#[test]
fn test_security_answers_remapped_after_reset() {
    let _ = dotenv();

    let username =
        format!("security_remapping_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let original_password = "OriginalPassword";
    let new_password = "NewPassword";

    cleanup_test_user(&username);

    // 1. Crear usuario
    let creation = create_user_with_access_token(
        username.clone(),
        original_password.to_string(),
        "Security Answers Remapping Test".to_string(),
    );
    assert!(creation.is_ok());

    let original_token_info = creation.unwrap();
    let original_access_token = original_token_info.access_token.clone();

    // 2. Configurar respuestas de seguridad
    let answers = [
        "first_answer".to_string(),
        "second_answer".to_string(),
        "third_answer".to_string(),
    ];
    let config_result = configure_all_security_answers(original_access_token.clone(), answers.clone());
    assert!(config_result.is_ok());

    // 3. Resetear contraseña
    let reset_result = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        new_password.to_string(),
    );
    assert!(reset_result.is_ok());

    let new_token_info = reset_result.unwrap();
    let new_access_token = new_token_info.access_token.clone();

    // 4. Verificar que TODAS las 3 respuestas sigan siendo válidas después del reset
    // (esto hace login con el username y nueva contraseña, luego valida respuestas)
    for (index, answer) in answers.iter().enumerate() {
        let validate_result =
            general_api::repos::auth::validate_security_answer(
                username.clone(),
                index as u8,
                answer.clone(),
            );
        assert!(
            validate_result.is_ok(),
            "Answer {} should still be valid after reset",
            index
        );
    }

    // Cleanup
    cleanup_test_user(&username);
}

/// TEST: Verificar que los contadores de datos (payments, loans, fines flags) se remapiean
#[test]
fn test_data_flags_remapped_after_reset() {
    let _ = dotenv();

    let username =
        format!("flags_test_{}", rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect::<String>());
    let original_password = "OriginalPassword";
    let new_password = "NewPassword";

    cleanup_test_user(&username);

    // 1. Crear usuario
    let creation = create_user_with_access_token(
        username.clone(),
        original_password.to_string(),
        "Flags Remapping Test".to_string(),
    );
    assert!(creation.is_ok());

    let original_token_info = creation.unwrap();
    let original_access_token = original_token_info.access_token.clone();
    let original_db_key = hashing_composite_key(&[&original_access_token]);

    // 2. Configurar respuestas y resetear
    let answers = [
        "answer_0".to_string(),
        "answer_1".to_string(),
        "answer_2".to_string(),
    ];
    let _ = configure_all_security_answers(original_access_token.clone(), answers.clone());
    let reset_result = reset_password(
        username.clone(),
        0,
        answers[0].clone(),
        new_password.to_string(),
    );
    assert!(reset_result.is_ok());

    let new_token_info = reset_result.unwrap();
    let new_access_token = new_token_info.access_token.clone();
    let new_db_key = hashing_composite_key(&[&new_access_token]);

    // 3. Verificar que los flags existan en el nuevo db_key
    let mut con = get_pool_connection().into_inner().get().unwrap();
    
    let payments_flag: bool = con
        .get(format!("users:{}:payments", new_db_key))
        .unwrap_or(false);
    let loans_flag: bool = con
        .get(format!("users:{}:loans", new_db_key))
        .unwrap_or(false);
    let fines_flag: bool = con
        .get(format!("users:{}:fines", new_db_key))
        .unwrap_or(false);

    // Los flags deberían ser false (como se inicializan en create_user)
    assert_eq!(payments_flag, false, "payments flag should be false");
    assert_eq!(loans_flag, false, "loans flag should be false");
    assert_eq!(fines_flag, false, "fines flag should be false");

    // 4. Verificar que los OLD flags ya no existan
    let old_payments_exists: bool = con
        .exists(format!("users:{}:payments", original_db_key))
        .unwrap_or(false);
    let old_loans_exists: bool = con
        .exists(format!("users:{}:loans", original_db_key))
        .unwrap_or(false);
    let old_fines_exists: bool = con
        .exists(format!("users:{}:fines", original_db_key))
        .unwrap_or(false);

    assert!(
        !old_payments_exists && !old_loans_exists && !old_fines_exists,
        "Old flags should be deleted after reset"
    );

    // Cleanup
    cleanup_test_user(&username);
}
