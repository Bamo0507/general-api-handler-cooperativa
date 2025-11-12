use redis::{cmd, Commands, JsonCommands};
use utils::{
    hashing_composite_key, get_db_key_from_username, copy_redis_field, 
    copy_security_answers, delete_keys_by_pattern
};

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

    // Get db_composite_key from username using helper
    let db_composite_key = get_db_key_from_username(&user_name, &mut con)?;

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

/// guarda las 3 respuestas de seguridad para un usuario usando access_token
pub fn configure_all_security_answers(
    access_token: String,
    answers: [String; 3],
) -> Result<(), StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // obtiene db_composite_key del access_token
    let db_composite_key = hashing_composite_key(&[&access_token]);

    // verifica que el usuario exista
    let exists: bool = con
        .exists(format!("users:{}:complete_name", &db_composite_key))
        .unwrap_or(false);
    if !exists {
        return Err(StatusMessage {
            message: "Usuario no encontrado o token inválido".to_string(),
        });
    }

    // guarda las 3 respuestas hasheadas con su índice
    for (index, answer) in answers.iter().enumerate() {
        let normalized_answer = answer.trim().to_lowercase();
        let answer_hash = hashing_composite_key(&[&normalized_answer]);

        let _: () = con
            .set(
                format!("users:{}:security_answer_{}", &db_composite_key, index),
                answer_hash,
            )
            .map_err(|_| StatusMessage {
                message: format!("No se pudo guardar la respuesta {} de seguridad", index),
            })?;
    }

    Ok(())
}

/// valida la respuesta de seguridad para recuperación de contraseña
pub fn validate_security_answer(
    user_name: String,
    question_index: u8,
    security_answer: String,
) -> Result<String, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // obtiene db_composite_key del username usando helper
    let db_composite_key = get_db_key_from_username(&user_name, &mut con)?;

    // obtiene la respuesta hasheada guardada en ese índice
    let stored_answer_hash: String = con
        .get(format!("users:{}:security_answer_{}", &db_composite_key, question_index))
        .map_err(|_| StatusMessage {
            message: "Usuario sin pregunta de seguridad configurada".to_string(),
        })?;

    // normaliza y hashea la respuesta que el usuario ingresó
    let normalized_answer = security_answer.trim().to_lowercase();
    let provided_answer_hash = hashing_composite_key(&[&normalized_answer]);

    // compara los hashes
    if provided_answer_hash == stored_answer_hash {
        Ok(db_composite_key)
    } else {
        Err(StatusMessage {
            message: "Respuesta incorrecta".to_string(),
        })
    }
}

/// resetea la contraseña validando respuesta de seguridad
pub fn reset_password(
    user_name: String,
    question_index: u8,
    security_answer: String,
    new_pass: String,
) -> Result<TokenInfo, StatusMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    // valida la respuesta y obtiene el db_composite_key anterior
    let old_db_composite_key = validate_security_answer(user_name.clone(), question_index, security_answer)?;

    // genera los nuevos hashes con la nueva contraseña
    let new_access_token = hashing_composite_key(&[&user_name, &new_pass]);
    let new_db_composite_key = hashing_composite_key(&[&new_access_token]);
    let affiliate_key = hashing_composite_key(&[&user_name]);

    // obtiene el nombre del usuario (lo necesita para la nueva entrada)
    let real_name: String = con
        .get(format!("users:{}:complete_name", &old_db_composite_key))
        .map_err(|_| StatusMessage {
            message: "No se pudo obtener datos del usuario".to_string(),
        })?;

    // copia todos los datos del usuario a las nuevas claves (con el nuevo db_composite_key)
    
    // datos base del usuario
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

    // copia las 3 respuestas de seguridad usando helper
    copy_security_answers(&mut con, &old_db_composite_key, &new_db_composite_key)?;

    // copia campos financieros usando helper
    copy_redis_field(&mut con, "payed_to_capital", &old_db_composite_key, &new_db_composite_key, 0.0)?;
    copy_redis_field(&mut con, "owed_capital", &old_db_composite_key, &new_db_composite_key, 0.0)?;
    
    // copia tipo de usuario usando helper
    copy_redis_field(&mut con, "is_directive", &old_db_composite_key, &new_db_composite_key, false)?;

    // Copiar datos de payments, loans y fines del usuario (todas las claves individuales)
    // Estos datos se almacenan como JSON, los copiamos directamente preservando su contenido
    for data_type in &["payments", "loans", "fines"] {
        let pattern = format!("users:{}:{}:*", &old_db_composite_key, data_type);
        if let Ok(keys_iter) = con.scan_match::<String, String>(pattern) {
            let old_keys: Vec<String> = keys_iter.collect();
            
            for old_key in old_keys {
                // Extract the hash/id part after the data_type (último componente después del último :)
                if let Some(hash_id) = old_key.split(':').last() {
                    let new_key = format!("users:{}:{}:{}", &new_db_composite_key, data_type, hash_id);
                    
                    // Copiar datos JSON preservando la estructura exacta
                    // Usar cmd("JSON.GET") directamente para obtener el JSON raw
                    if let Ok(json_str) = cmd("JSON.GET")
                        .arg(&old_key)
                        .query::<String>(&mut con)
                    {
                        let _: Result<(), _> = cmd("JSON.SET")
                            .arg(&new_key)
                            .arg("$")
                            .arg(&json_str)
                            .query(&mut con)
                            .map_err(|e| println!("Warning: couldn't copy {} data: {:?}", data_type, e));
                    }
                }
            }
        }
    }

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

    // crea el nuevo mapping con las nuevas claves
    let _: () = con
        .set(
            format!("affiliate_key_to_db_access:{}", &affiliate_key),
            &new_db_composite_key,
        )
        .map_err(|_| StatusMessage {
            message: "No se pudo actualizar mapeo de usuario".to_string(),
        })?;

    // elimina todas las claves viejas del usuario anterior (seguridad: invalida el token anterior)
    let old_keys_to_delete = vec![
        format!("users:{}:complete_name", &old_db_composite_key),
        format!("users:{}:affiliate_key", &old_db_composite_key),
        format!("users:{}:payed_to_capital", &old_db_composite_key),
        format!("users:{}:owed_capital", &old_db_composite_key),
        format!("users:{}:is_directive", &old_db_composite_key),
        format!("users:{}:payments", &old_db_composite_key),
        format!("users:{}:loans", &old_db_composite_key),
        format!("users:{}:fines", &old_db_composite_key),
        format!("users:{}:security_answer_0", &old_db_composite_key),
        format!("users:{}:security_answer_1", &old_db_composite_key),
        format!("users:{}:security_answer_2", &old_db_composite_key),
    ];
    
    for key in old_keys_to_delete {
        let _: Result<(), _> = con.del(&key);
    }

    // Eliminar también todas las claves individuales de payments/loans/fines del db_key anterior usando helper
    for data_type in &["payments", "loans", "fines"] {
        let pattern = format!("users:{}:{}:*", &old_db_composite_key, data_type);
        let _ = delete_keys_by_pattern(&mut con, pattern);
    }

    // retorna el nuevo token con datos actualizados
    let is_directive: bool = con
        .get(format!("users:{}:is_directive", &new_db_composite_key))
        .unwrap_or(false);
    
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
