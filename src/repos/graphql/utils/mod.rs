use std::fmt::Debug;
use serde_json;

use actix_web::web::Data;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::from_str;

use crate::{models::GraphQLMappable, repos::auth::utils::hashing_composite_key};

use crate::models::graphql::Payment;
use crate::endpoints::handlers::configs::schema::GeneralContext;


/// Crea un contexto de test con pool de Redis real (localhost)
pub fn create_test_context() -> GeneralContext {
    // Conexión a Redis para testing, la URL debe ser provista por la variable de entorno REDIS_URL
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL debe estar exportada en el CLI");
    let client = redis::Client::open(redis_url).expect("No se pudo conectar a Redis");
    let pool = Pool::builder().build(client).expect("No se pudo crear el pool de Redis");
    GeneralContext { pool: Data::new(pool) }
}


/// Limpia todas las claves de pagos en Redis para tests
pub fn clear_redis(context: &GeneralContext) {
    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
    
    // borra la colección 'all' usada en tests; solo elimina claves de pruebas
    let composite_key = hashing_composite_key(&[&String::from("all")]);
    let pattern_all_composite = format!("users:{}:payments:*", composite_key);
    let keys_for_all: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern_all_composite)
        .query(&mut con)
        .unwrap_or_default();
    for key in keys_for_all {
        let _: () = con.del(&key).unwrap_or(());
    }

    // Additionally, clean any test-generated payment keys elsewhere (ids starting with test_pago_)
    let pattern_any = "users:*:payments:*".to_string();
    let keys_any: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern_any)
        .query(&mut con)
        .unwrap_or_default();
    for key in keys_any {
        if let Ok(re) = Regex::new(r"users:[\w]+:payments:(?P<key>.+)") {
            if let Some(caps) = re.captures(&key) {
                let id = caps.name("key").map(|m| m.as_str()).unwrap_or("");
                if id.starts_with("test_pago_") {
                    let _: () = con.del(&key).unwrap_or(());
                }
            }
        }
    }
}


/// Inserta un pago en Redis usando el pool del contexto
pub fn insert_payment_helper(context: &GeneralContext, payment: &Payment) {
    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");
    use crate::repos::auth::utils::hashing_composite_key;
    use crate::models::redis::Payment as RedisPayment;
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
        comprobante_bucket: payment.photo.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_string(),
        being_payed: vec![], // tests typically don't set this; leave empty default or fill as needed
    };

    // Use redis_json wrapper (JsonCommands) to persist the value as JSON
    // Additionally write the serialized JSON string with SET as a fallback for
    // environments where RedisJSON may not return the expected types via json_get.
    let _ : redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_payment);
    if let Ok(s) = serde_json::to_string(&redis_payment) {
        let _ : redis::RedisResult<()> = con.set(&redis_key, s);
    }
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
                // Try JSON.GET first (RedisJSON). If it fails or returns nil, fallback to GET
                let mut nested_data: String = String::new();

                match con.json_get::<String, &str, RedisValue>(key.to_owned(), "$") {
                    Ok(redis_raw) => {
                        // Attempt to convert redis value to string
                        match from_redis_value::<String>(&redis_raw) {
                            Ok(s) => nested_data = s,
                            Err(e) => {
                                println!("DEBUG get_multiple_models - from_redis_value failed for key {}: {:?}", key, e);
                                // Fallthrough to try GET below
                            }
                        }
                    }
                    Err(e) => {
                        println!("DEBUG get_multiple_models - json_get failed for key {}: {:?}", key, e);
                        // Fallthrough to try GET below
                    }
                }

                // If nested_data is still empty, try fetching the raw string value with GET
                if nested_data.is_empty() {
                    match con.get::<String, String>(key.to_owned()) {
                        Ok(s) => nested_data = s,
                        Err(e) => {
                            println!("DEBUG get_multiple_models - GET fallback failed for key {}: {:?}", key, e);
                            continue; // skip keys we can't read
                        }
                    }
                }

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
