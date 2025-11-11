use serde_json;
use std::fmt::{format, Debug};

use actix_web::web::Data;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::from_str;

use crate::{models::GraphQLMappable, repos::auth::utils::hashing_composite_key};

use crate::endpoints::handlers::configs::schema::GeneralContext;
use crate::models::graphql::Payment;

/// Inserta un pago en Redis usando el pool del contexto y devuelve la clave Redis creada.
/// Formato de la clave: users:{hash("all")}:payments:{id}
pub fn insert_payment_helper(context: &GeneralContext, payment: &Payment) -> String {
    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
    use crate::models::redis::Payment as RedisPayment;
    use crate::repos::auth::utils::hashing_composite_key;
    // Clave individual por pago, siguiendo el patrón: users:{hash("all")}:payments:{id}
    let composite_key = hashing_composite_key(&[&String::from("all")]);
    let redis_key = format!("users:{}:payments:{}", composite_key, payment.id);
    // aquí mapeamos
    // explícitamente del modelo GraphQL al struct de Redis actual para que la
    // persistencia use los nombres/formatos correctos. Usamos el wrapper JSON
    // (`json_set`) en lugar de HSET/manual serde para que los objetos queden guardados
    // como JSON y sean recuperables con `json_get`
    let redis_payment = RedisPayment {
        date_created: payment.payment_date.clone(),
        account_number: payment.account_num.clone(),
        total_amount: payment.total_amount,
        name: payment.name.clone(),
        comments: payment.commentary.clone(),
        comprobante_bucket: payment.photo_path.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_string(),
        being_payed: vec![], // tests typically don't set this; leave empty default or fill as needed
    };

    // Use redis_json wrapper (JsonCommands) to persist the value as JSON
    let _: redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_payment);
    redis_key
}

///Function for returning n number of any type value, having a function as a generator
//(Ik syntaxis looks scary in the parameters, but it ain't)
pub fn return_n_dummies<Value>(dummy_generator: &dyn Fn() -> Value, n: i32) -> Vec<Value> {
    let mut dummy_list: Vec<Value> = vec![];

    //pretty simple logic
    for _ in 0..n {
        dummy_list.push(dummy_generator());
    }

    dummy_list
}

/// Function that returns only the relative payment key
/// where the raw_key is the string of the value, and the key_type is the type of the redis
/// key
pub fn get_key(raw_key: String, key_type: String) -> String {
    // we format for injecting the key_type
    let re = Regex::new(format!(r"users:[\w]+:{}:(?<key>\w+)", key_type).as_str()).unwrap();

    // let's assume this is correct, cause the only value that will enter here will be payment_keys
    let split_key = re.captures(&raw_key).unwrap();

    split_key["key"].to_string()
}

// this method could be really slow, I'll see a way for optimizing later
pub fn get_db_access_token_with_affiliate_key(
    affiliate_key: String,
    pool: Data<Pool<Client>>,
) -> Result<String, String> {
    let mut con = pool.get().expect("Couldn't connect to pool");

    match con.get::<String, String>(format!("affiliate_key_to_db_access:{}", affiliate_key)) {
        Ok(db_key) => Ok(db_key),
        Err(err) => {
            println!("{err:?}");
            Err("Couldn't Get Db Token".to_owned())
        }
    }
}

/// Function for generalizing the fetching for redis values and turnining them in to GraphQLObject
pub fn get_multiple_models_by_id<GraphQLType, RedisType>(
    access_token: Option<String>,
    db_token: Option<String>,
    pool: Data<Pool<Client>>,
    redis_key_type: String,
) -> Result<Vec<GraphQLType>, String>
where
    RedisType: DeserializeOwned + Clone + GraphQLMappable<GraphQLType> + Debug,
{
    let db_access_token;
    let mut con = pool.get().expect("Couldn't connect to pool");

    if (access_token.is_none()) && (db_token.is_none()) {
        return Err("At leat one of the token most be something".to_owned());
    }

    if let Some(token) = access_token {
        db_access_token = hashing_composite_key(&[&token]);
    } else {
        // cause of them at least has to be something
        db_access_token = db_token.unwrap();
    }

    match con
        .scan_match::<String, String>(format!("users:{}:{}:*", db_access_token, redis_key_type))
    {
        Ok(keys) => {
            let mut graphql_object_list: Vec<GraphQLType> = Vec::new();

            // conn for fetching redis models
            let mut con = pool.get().expect("Couldn't connect to pool");

            for key in keys {
                // We first fetch the raw data, first
                let redis_raw = con
                    .json_get::<String, &str, RedisValue>(key.to_owned(), "$")
                    .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                // for some reason redis gives all the info deserialize, so I have to do the
                // serializion process my self
                let nested_data = from_redis_value::<String>(&redis_raw).unwrap(); // first is

                // ik that I could've made the direct mapping to the GraphQl object, but I
                // rather using my own name standar for the redis keys and that Bryan manages
                // the names as however he want's it
                let redis_object_parsed =
                    from_str::<Vec<RedisType>>(nested_data.as_str()).unwrap()[0].clone();
                // cause
                // of the way  of the way the json library works on redis, the objects follow a
                // list type fetching, but as the db was planned, we where heading for a more
                // key aproach overall, so that's why we need the cast (after all there will
                // always be just one element)

                // now we do the graphql mapping

                graphql_object_list.push(redis_object_parsed.to_graphql_type(key));
            }

            Ok(graphql_object_list)
        }
        Err(_) => Err("Couldn't get users payments".to_string()),
    }
}

/// versión de get_multiple_models_by_id que retorna también las keys
/// sirve para después enriquecer los objetos con presented_by_name u otra info
pub fn get_multiple_models_by_id_with_keys<GraphQLType, RedisType>(
    access_token: Option<String>,
    db_token: Option<String>,
    pool: Data<Pool<Client>>,
    redis_key_type: String,
) -> Result<(Vec<GraphQLType>, Vec<String>), String>
where
    RedisType: DeserializeOwned + Clone + GraphQLMappable<GraphQLType> + Debug,
{
    let mut db_access_token;
    let mut con = pool.get().expect("Couldn't connect to pool");

    if (access_token == None) && (db_token == None) {
        return Err("At leat one of the token most be something".to_owned());
    }

    if let Some(token) = access_token {
        db_access_token = hashing_composite_key(&[&token]);
    } else {
        db_access_token = db_token.unwrap();
    }

    match con
        .scan_match::<String, String>(format!("users:{}:{}:*", db_access_token, redis_key_type))
    {
        Ok(keys) => {
            let mut graphql_object_list: Vec<GraphQLType> = Vec::new();
            let mut key_list: Vec<String> = Vec::new();

            let mut con = pool.get().expect("Couldn't connect to pool");

            for key in keys {
                let redis_raw = con
                    .json_get::<String, &str, RedisValue>(key.to_owned(), "$")
                    .unwrap();

                let nested_data = from_redis_value::<String>(&redis_raw).unwrap();

                let redis_object_parsed =
                    from_str::<Vec<RedisType>>(nested_data.as_str()).unwrap()[0].clone();

                graphql_object_list.push(redis_object_parsed.to_graphql_type(key.clone()));
                key_list.push(key);
            }

            Ok((graphql_object_list, key_list))
        }
        Err(_) => Err("Couldn't get users payments".to_string()),
    }
}

/// Function for generalizing the fetching for redis values and turnining them in to GraphQLObject
pub fn get_multiple_models<GraphQLType, RedisType>(
    access_token: String,
    pool: Data<Pool<Client>>,
    redis_key_type: String,
) -> Result<Vec<GraphQLType>, String>
where
    RedisType: DeserializeOwned + Clone + GraphQLMappable<GraphQLType> + Debug,
{
    let mut con = pool.get().expect("Couldn't connect to pool");
    let db_access_token = hashing_composite_key(&[&access_token]);

    match con
        .scan_match::<String, String>(format!("users:{}:{}:*", db_access_token, redis_key_type))
    {
        Ok(keys) => {
            // Nota para reviewers: después del refactor de pagos podemos encontrarnos
            // con claves en Redis que no sean JSON o que no sigan el shape esperado.
            // En vez de `unwrap()`-ear, este helper es defensivo: intenta `json_get`,
            // intenta parsear y si falla simplemente ignora la clave. Esto responde al
            // comentario del PR sobre robustez y evita panics por datos antiguos/no esperados.
            // (Comentario casual: mejor saltarse una clave rara que romper toda la query.)
            let mut graphql_object_list: Vec<GraphQLType> = Vec::new();

            // conn for fetching redis models
            let mut con = pool.get().expect("Couldn't connect to pool");

            // Collect keys into a Vec so we can log and iterate deterministically for debugging
            let key_vec: Vec<String> = keys.collect();
            println!("DEBUG get_multiple_models - scanned keys: {:?}", key_vec);

            for key in key_vec {
                // We first try to fetch JSON at path "$" for the key. Skip keys that don't
                // have a JSON value or where the response is nil (this can happen if there is
                // a non-id payment key like `users:...:payments` stored as a string or empty).
                let redis_raw_res = con.json_get::<String, &str, RedisValue>(key.to_owned(), "$");
                let redis_raw = match redis_raw_res {
                    Ok(v) => v,
                    Err(e) => {
                        println!(
                            "DEBUG get_multiple_models - json_get failed for key {}: {:?}",
                            key, e
                        );
                        continue; // skip invalid/non-json keys
                    }
                };

                // Try to convert redis value to string, then parse JSON into RedisType
                let nested_data_res = from_redis_value::<String>(&redis_raw);
                let nested_data = match nested_data_res {
                    Ok(s) => s,
                    Err(e) => {
                        println!(
                            "DEBUG get_multiple_models - from_redis_value failed for key {}: {:?}",
                            key, e
                        );
                        continue;
                    }
                };

                // Intentar deserializar como array; si falla, intentar como objeto individual
                let parsed_vec_res = from_str::<Vec<RedisType>>(nested_data.as_str());
                let mut parsed_objects: Vec<RedisType> = match parsed_vec_res {
                    Ok(v) => v,
                    Err(_) => {
                        // Si no es array, intentar como objeto individual
                        match from_str::<RedisType>(nested_data.as_str()) {
                            Ok(obj) => vec![obj],
                            Err(e) => {
                                println!("DEBUG get_multiple_models - JSON parse failed for key {}: {} -> {}", key, nested_data, e);
                                continue;
                            }
                        }
                    }
                };

                if parsed_objects.is_empty() {
                    continue;
                }

                for redis_object_parsed in parsed_objects {
                    println!("DEBUG get_multiple_models - parsed key {} into object", key);
                    // now we do the graphql mapping
                    graphql_object_list.push(redis_object_parsed.to_graphql_type(key.clone()));
                }
            }

            Ok(graphql_object_list)
        }
        Err(_) => Err("Couldn't get users payments".to_string()),
    }
}

/// creado porque necesitamos escanear con patrones arbitrarios como users:*:payments:*
/// los otros helpers construyen el patrón desde un access token y no sirven para queries globales
/// este recibe el patrón ya formado, hace el json_get defensivo y mapea los objetos redis a graphql
/// lo dejo separado pa no tocar lo que ya usa access token y pa consultas que spannean todo
pub fn get_multiple_models_by_pattern<GraphQLType, RedisType>(
    pattern: String,
    pool: Data<Pool<Client>>,
) -> Result<Vec<GraphQLType>, String>
where
    RedisType: DeserializeOwned + Clone + GraphQLMappable<GraphQLType> + Debug,
{
    let mut con = pool.get().expect("Couldn't connect to pool");

    match con.scan_match::<String, String>(pattern) {
        Ok(keys) => {
            let mut graphql_object_list: Vec<GraphQLType> = Vec::new();

            // conn for fetching redis models
            let mut con = pool.get().expect("Couldn't connect to pool");

            // Collect keys into a Vec so we can log and iterate deterministically for debugging
            let key_vec: Vec<String> = keys.collect();
            println!(
                "DEBUG get_multiple_models_by_pattern - scanned keys: {:?}",
                key_vec
            );

            for key in key_vec {
                let redis_raw_res = con.json_get::<String, &str, redis::Value>(key.to_owned(), "$");
                let redis_raw = match redis_raw_res {
                    Ok(v) => v,
                    Err(e) => {
                        println!("DEBUG get_multiple_models_by_pattern - json_get failed for key {}: {:?}", key, e);
                        continue; // skip invalid/non-json keys
                    }
                };

                let nested_data_res = from_redis_value::<String>(&redis_raw);
                let nested_data = match nested_data_res {
                    Ok(s) => s,
                    Err(e) => {
                        println!("DEBUG get_multiple_models_by_pattern - from_redis_value failed for key {}: {:?}", key, e);
                        continue;
                    }
                };

                let parsed_vec_res = from_str::<Vec<RedisType>>(nested_data.as_str());
                let mut parsed_objects: Vec<RedisType> = match parsed_vec_res {
                    Ok(v) => v,
                    Err(_) => match from_str::<RedisType>(nested_data.as_str()) {
                        Ok(obj) => vec![obj],
                        Err(e) => {
                            println!("DEBUG get_multiple_models_by_pattern - JSON parse failed for key {}: {} -> {}", key, nested_data, e);
                            continue;
                        }
                    },
                };

                if parsed_objects.is_empty() {
                    continue;
                }

                for redis_object_parsed in parsed_objects {
                    graphql_object_list.push(redis_object_parsed.to_graphql_type(key.clone()));
                }
            }

            Ok(graphql_object_list)
        }
        Err(_) => Err("Couldn't get users payments".to_string()),
    }
}

/// versión nueva del helper que retorna tanto los objetos como las keys
/// sirve para después poder enriquecer los objetos con info adicional (ej: presented_by_name)
/// no modificamos el helper original para no romper código existente (non-breaking change)
pub fn get_multiple_models_by_pattern_with_keys<GraphQLType, RedisType>(
    pattern: String,
    pool: Data<Pool<Client>>,
) -> Result<(Vec<GraphQLType>, Vec<String>), String>
where
    RedisType: DeserializeOwned + Clone + GraphQLMappable<GraphQLType> + Debug,
{
    let mut con = pool.get().expect("Couldn't connect to pool");

    match con.scan_match::<String, String>(pattern) {
        Ok(keys) => {
            let mut graphql_object_list: Vec<GraphQLType> = Vec::new();
            let mut key_list: Vec<String> = Vec::new();

            // conn for fetching redis models
            let mut con = pool.get().expect("Couldn't connect to pool");

            // Collect keys into a Vec so we can log and iterate deterministically for debugging
            let key_vec: Vec<String> = keys.collect();
            println!(
                "DEBUG get_multiple_models_by_pattern_with_keys - scanned keys: {:?}",
                key_vec
            );

            for key in key_vec {
                let redis_raw_res = con.json_get::<String, &str, redis::Value>(key.to_owned(), "$");
                let redis_raw = match redis_raw_res {
                    Ok(v) => v,
                    Err(e) => {
                        println!("DEBUG get_multiple_models_by_pattern_with_keys - json_get failed for key {}: {:?}", key, e);
                        continue; // skip invalid/non-json keys
                    }
                };

                let nested_data_res = from_redis_value::<String>(&redis_raw);
                let nested_data = match nested_data_res {
                    Ok(s) => s,
                    Err(e) => {
                        println!("DEBUG get_multiple_models_by_pattern_with_keys - from_redis_value failed for key {}: {:?}", key, e);
                        continue;
                    }
                };

                let parsed_vec_res = from_str::<Vec<RedisType>>(nested_data.as_str());
                let mut parsed_objects: Vec<RedisType> = match parsed_vec_res {
                    Ok(v) => v,
                    Err(_) => match from_str::<RedisType>(nested_data.as_str()) {
                        Ok(obj) => vec![obj],
                        Err(e) => {
                            println!("DEBUG get_multiple_models_by_pattern_with_keys - JSON parse failed for key {}: {} -> {}", key, nested_data, e);
                            continue;
                        }
                    },
                };

                if parsed_objects.is_empty() {
                    continue;
                }

                for redis_object_parsed in parsed_objects {
                    graphql_object_list.push(redis_object_parsed.to_graphql_type(key.clone()));
                    // guardamos la key correspondiente para poder usar después
                    key_list.push(key.clone());
                }
            }

            Ok((graphql_object_list, key_list))
        }
        Err(_) => Err("Couldn't get users payments".to_string()),
    }
}

/// helper para extraer el user_hash de una key de redis con el pattern users:{hash}:*
/// sirve para después buscar el complete_name del usuario
/// retorna None si la key no matchea el patrón esperado
pub fn extract_user_hash_from_key(key: &str) -> Option<String> {
    // regex para capturar el hash entre users: y el siguiente :
    let re = Regex::new(r"users:(?<hash>[a-fA-F0-9]+):").ok()?;
    let captures = re.captures(key)?;
    Some(captures["hash"].to_string())
}

/// helper genérico que toma una lista de objetos y les asigna el presented_by_name
/// usando el trait WithPresenterName. funciona con cualquier modelo (Payment, Fine, Loan, etc)
/// que implemente el trait. extrae el user_hash de cada key, fetchea el complete_name desde redis
/// y lo asigna al objeto. si no encuentra el nombre usa DEFAULT_PRESENTER_NAME
pub fn enrich_with_presenter_names<T>(
    mut objects: Vec<T>,
    keys: Vec<String>,
    pool: &Pool<Client>,
) -> Vec<T>
where
    T: crate::models::WithPresenterName,
{
    // aseguramos que tengamos la misma cantidad de objetos y keys
    if objects.len() != keys.len() {
        println!(
            "WARNING: enrich_with_presenter_names - objects.len() ({}) != keys.len() ({})",
            objects.len(),
            keys.len()
        );
        return objects;
    }

    for (obj, key) in objects.iter_mut().zip(keys.iter()) {
        if let Some(user_hash) = extract_user_hash_from_key(key) {
            // intentamos fetchear el complete_name del usuario
            let mut con = pool.get().expect("Couldn't connect to pool");
            let name_result = con.get::<String, String>(format!("users:{}:complete_name", user_hash));

            let name = match name_result {
                Ok(n) => n,
                Err(_) => {
                    println!(
                        "WARNING: enrich_with_presenter_names - no complete_name for user_hash {}",
                        user_hash
                    );
                    crate::models::DEFAULT_PRESENTER_NAME.to_string()
                }
            };

            obj.set_presenter_name(name);
        } else {
            // si no pudimos extraer el hash, usamos el default
            println!(
                "WARNING: enrich_with_presenter_names - couldn't extract user_hash from key: {}",
                key
            );
            obj.set_presenter_name(crate::models::DEFAULT_PRESENTER_NAME.to_string());
        }
    }

    objects
}
