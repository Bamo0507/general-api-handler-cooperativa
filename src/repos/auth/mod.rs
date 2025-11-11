use redis::{cmd, Commands};
use utils::hashing_composite_key;

use crate::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    models::{
        auth::{TokenInfo, UserType},
        StatusMessage,
    },
};

pub mod utils;

//TODO: ~Set for ALC (ALC is out of scoope)~
pub fn create_user_with_access_token(
    user_name: String,
    pass: String,
    real_name: String,
) -> Result<TokenInfo, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool"); //Can't abstracted to a struct, :C

    // This will be the token that the user will use for loging
    let access_token = hashing_composite_key(&[&user_name, &pass]);

    // The reference on the db
    let db_composite_key = hashing_composite_key(&[&access_token]);

    // getting username and the las character for getting the affiliate key
    let affiliate_key = hashing_composite_key(&[&user_name]);

    //For checking the existance of the field
    match cmd("GET")
        .arg(format!("users_on_used:{}", &user_name))
        .query::<String>(&mut con)
    {
        //This will look weird, but we are looking here in the case it fails
        Err(_) => {
            //For creating fields
            // IK is pretty repetitive, but is the best way for being the most explicit neccesary

            //Want to have the resource the closest to key level, cause is just for checking if it exists
            let _: () = con
                .set(format!("users_on_used:{}", &user_name), "")
                .expect("USERNAME CREATION : Couldn't filled username");

            let _: () = con
                .set(
                    format!("users:{}:complete_name", &db_composite_key),
                    &real_name,
                )
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            let _: () = con
                .set(
                    format!("users:{}:affiliate_key", &db_composite_key),
                    &affiliate_key,
                )
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            let _: () = con
                .set(format!("affiliate_keys:{}", &affiliate_key), &user_name)
                .expect("USERNAME CREATION : Couldn't filled username");

            let _: () = con
                .set(
                    format!("affiliate_key_to_db_access:{}", &affiliate_key),
                    &db_composite_key,
                )
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:payed_to_capital", &db_composite_key), 0.0)
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:owed_capital", &db_composite_key), 0.0)
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            // For default any new user won't be
            let _: () = con
                .set(format!("users:{}:is_directive", &db_composite_key), false)
                .expect("ACCESS TOKEN CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:payments", &db_composite_key), false)
                .expect("BASE PAYMENTS CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:loans", &db_composite_key), false)
                .expect("BASE LOANS CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:fines", &db_composite_key), false)
                .expect("BASE FINES CREATION: Couldn't create field");

            Ok(TokenInfo {
                user_name,
                access_token,
                user_type: UserType::General.to_string(),
            })
        }

        Ok(_) => Err(StatusMessage {
            message: "Couldn't Create User".to_string(),
        }),
    }
}

//TODO: Refactor this for recieving the access token
pub fn get_user_access_token(user_name: String, pass: String) -> Result<TokenInfo, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool"); //Can't abstracted to a struct, :C

    // THe token derived from the user and pass
    let access_token = hashing_composite_key(&[&user_name, &pass]);

    // How is registered on the db
    let db_access_token = hashing_composite_key(&[&access_token]);

    //Passing an String for recieving an nil
    match cmd("EXISTS")
        .arg(format!("users:{db_access_token}:complete_name")) //Closests key-value we have at hand
        .query::<bool>(&mut con)
    {
        Ok(it_exists) => {
            if it_exists {
                let mut con = get_pool_connection()
                    .get()
                    .expect("Couldn't connect to pool");

                // get the the user type

                let user_type = match con
                    .get::<String, bool>(format!("users:{db_access_token}:is_directive"))
                    .unwrap_or_default()
                {
                    true => UserType::Directive.to_string(),
                    false => UserType::General.to_string(),
                };

                // cause this is an "earlier" return, can't use the other syntax
                return Ok(TokenInfo {
                    user_name,
                    access_token,
                    user_type,
                });
            }

            Err(StatusMessage {
                message: "User Might Not Exist or User/Password is wrong".to_string(),
            })
        }
        Err(e) => Err(StatusMessage {
            message: format!("Error: {e}"),
        }),
    }
}

/// Configure security answer for a user
/// 
/// # Arguments
/// * `user_name` - Username to configure
/// * `security_question_index` - Index (0, 1, or 2) of the question
/// * `security_answer` - User's answer to the security question (will be hashed)
pub fn configure_security_answer(
    user_name: String,
    security_question_index: u8,
    security_answer: String,
) -> Result<(), StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // Get affiliate_key from username
    let affiliate_key = hashing_composite_key(&[&user_name]);

    // Get db_composite_key from Redis mapping
    let db_composite_key: String = con
        .get(format!("affiliate_key_to_db_access:{}", &affiliate_key))
        .map_err(|_| StatusMessage {
            message: "Usuario no encontrado".to_string(),
        })?;

    // Normalize answer: lowercase + trim
    let normalized_answer = security_answer.trim().to_lowercase();
    let answer_hash = hashing_composite_key(&[&normalized_answer]);

    // Save question index
    let _: () = con
        .set(
            format!("users:{}:security_question_index", &db_composite_key),
            security_question_index.to_string(),
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo guardar la pregunta de seguridad".to_string(),
        })?;

    // Save answer hash
    let _: () = con
        .set(
            format!("users:{}:security_answer", &db_composite_key),
            answer_hash,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo guardar la respuesta de seguridad".to_string(),
        })?;

    Ok(())
}

/// Validate security answer for password recovery
/// 
/// # Arguments
/// * `user_name` - Username attempting to recover password
/// * `security_answer` - Answer provided by user
/// 
/// # Returns
/// Ok(db_composite_key) if answer is correct, Err(StatusMessage) if incorrect or user not found
pub fn validate_security_answer(
    user_name: String,
    security_answer: String,
) -> Result<String, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // Get affiliate_key from username
    let affiliate_key = hashing_composite_key(&[&user_name]);

    // Get db_composite_key from Redis mapping
    let db_composite_key: String = con
        .get(format!("affiliate_key_to_db_access:{}", &affiliate_key))
        .map_err(|_| StatusMessage {
            message: "Usuario no encontrado".to_string(),
        })?;

    // Check if user has security question configured
    let stored_answer_hash: String = con
        .get(format!("users:{}:security_answer", &db_composite_key))
        .map_err(|_| StatusMessage {
            message: "Usuario sin pregunta de seguridad configurada".to_string(),
        })?;

    // Normalize and hash the provided answer
    let normalized_answer = security_answer.trim().to_lowercase();
    let provided_answer_hash = hashing_composite_key(&[&normalized_answer]);

    // Compare hashes
    if provided_answer_hash == stored_answer_hash {
        Ok(db_composite_key)
    } else {
        Err(StatusMessage {
            message: "Respuesta incorrecta".to_string(),
        })
    }
}

/// Reset password using security answer validation
/// 
/// # Arguments
/// * `user_name` - Username
/// * `security_answer` - Answer to security question (for validation)
/// * `new_pass` - New password
/// 
/// # Returns
/// Ok(TokenInfo) with new access_token if successful, Err(StatusMessage) otherwise
pub fn reset_password(
    user_name: String,
    security_answer: String,
    new_pass: String,
) -> Result<TokenInfo, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // Step 1: Validate security answer (this also gets old db_composite_key)
    let old_db_composite_key = validate_security_answer(user_name.clone(), security_answer)?;

    // Step 2: Generate new hashes with new password
    let new_access_token = hashing_composite_key(&[&user_name, &new_pass]);
    let new_db_composite_key = hashing_composite_key(&[&new_access_token]);
    let affiliate_key = hashing_composite_key(&[&user_name]);

    // Step 3: Get user's complete_name (needed for new account creation)
    let real_name: String = con
        .get(format!("users:{}:complete_name", &old_db_composite_key))
        .map_err(|_| StatusMessage {
            message: "No se pudo obtener datos del usuario".to_string(),
        })?;

    // Step 4: Get security question index and answer (to copy them)
    let security_question_index: String = con
        .get(format!("users:{}:security_question_index", &old_db_composite_key))
        .map_err(|_| StatusMessage {
            message: "No se pudo obtener pregunta de seguridad".to_string(),
        })?;

    let security_answer_hash: String = con
        .get(format!("users:{}:security_answer", &old_db_composite_key))
        .map_err(|_| StatusMessage {
            message: "No se pudo obtener respuesta de seguridad".to_string(),
        })?;

    // Step 5: Copy all user data to new keys (with new db_composite_key)
    // Base user info
    let _: () = con
        .set(
            format!("users:{}:complete_name", &new_db_composite_key),
            &real_name,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo crear nuevo usuario".to_string(),
        })?;

    let _: () = con
        .set(
            format!("users:{}:affiliate_key", &new_db_composite_key),
            &affiliate_key,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo crear nuevo usuario".to_string(),
        })?;

    // Security question data
    let _: () = con
        .set(
            format!("users:{}:security_question_index", &new_db_composite_key),
            &security_question_index,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo guardar pregunta de seguridad".to_string(),
        })?;

    let _: () = con
        .set(
            format!("users:{}:security_answer", &new_db_composite_key),
            &security_answer_hash,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo guardar respuesta de seguridad".to_string(),
        })?;

    // Copy financial fields
    let payed_to_capital: f64 = con
        .get(format!("users:{}:payed_to_capital", &old_db_composite_key))
        .unwrap_or(0.0);
    let _: () = con
        .set(
            format!("users:{}:payed_to_capital", &new_db_composite_key),
            payed_to_capital,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo copiar datos financieros".to_string(),
        })?;

    let owed_capital: f64 = con
        .get(format!("users:{}:owed_capital", &old_db_composite_key))
        .unwrap_or(0.0);
    let _: () = con
        .set(
            format!("users:{}:owed_capital", &new_db_composite_key),
            owed_capital,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo copiar datos financieros".to_string(),
        })?;

    // Copy user type
    let is_directive: bool = con
        .get(format!("users:{}:is_directive", &old_db_composite_key))
        .unwrap_or(false);
    let _: () = con
        .set(
            format!("users:{}:is_directive", &new_db_composite_key),
            is_directive,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo copiar tipo de usuario".to_string(),
        })?;

    // Copy counters
    let _: () = con
        .set(
            format!("users:{}:payments", &new_db_composite_key),
            false,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo inicializar contadores".to_string(),
        })?;

    let _: () = con
        .set(format!("users:{}:loans", &new_db_composite_key), false)
        .map_err(|_| StatusMessage {
            message: "No se pudo inicializar contadores".to_string(),
        })?;

    let _: () = con
        .set(format!("users:{}:fines", &new_db_composite_key), false)
        .map_err(|_| StatusMessage {
            message: "No se pudo inicializar contadores".to_string(),
        })?;

    // Step 6: Update affiliate_key_to_db_access mapping to new db_composite_key
    let _: () = con
        .set(
            format!("affiliate_key_to_db_access:{}", &affiliate_key),
            &new_db_composite_key,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo actualizar mapeo de usuario".to_string(),
        })?;

    // Return new token
    Ok(TokenInfo {
        user_name,
        access_token: new_access_token,
        user_type: if is_directive {
            UserType::Directive.to_string()
        } else {
            UserType::General.to_string()
        },
    })
}
