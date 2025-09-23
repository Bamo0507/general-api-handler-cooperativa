use std::fmt::Debug;

use actix_web::web::Data;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::from_str;

use crate::{models::GraphQLMappable, repos::auth::utils::hashing_composite_key};

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
