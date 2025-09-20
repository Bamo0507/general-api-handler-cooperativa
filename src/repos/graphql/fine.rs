use actix_web::web::Data;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use serde_json::from_str;

use crate::{
    models::{graphql::Fine, redis::Fine as RedisFine},
    repos::auth::utils::hashing_composite_key,
};

pub struct FineRepo {
    pub pool: Data<Pool<Client>>,
}

impl FineRepo {
    pub fn get_user_fines(&self, access_token: String) -> Result<Vec<Fine>, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        match con.scan_match::<String, String>(format!("users:{}:fines:*", db_access_token)) {
            Ok(keys) => {
                let mut fine_list: Vec<Fine> = Vec::new();

                let mut con = self.pool.get().expect("Couldn't connect to pool");

                for key in keys {
                    // We first fetch the raw data, first
                    let user_payment_raw = con
                        .json_get::<String, &str, RedisValue>(key.to_owned(), "$")
                        .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                    // for some reason redis gives all the info deserialize, so I have to do the
                    // serializion process my self
                    let nested_data =
                        from_redis_value::<String>(&user_payment_raw).unwrap_or_default(); // cause
                                                                                           // of the way  of the way the json library works on redis, the objects follow a
                                                                                           // list type fetching, but as the db was planned, we where heading for a more
                                                                                           // key aproach overall, so that's why we need the cast (after all there will
                                                                                           // always be just one element)

                    // ik that I could've made the direct mapping to the GraphQl object, but I
                    // rather using my own name standar for the redis keys and that Bryan manages
                    // the names as however he want's it
                    let user_fines_redis =
                        from_str::<Vec<RedisFine>>(nested_data.as_str()).unwrap()[0].clone();
                    // that
                    // was just for getting the redis object, now I have to do the mapping

                    // now we do the payment mapping
                    fine_list.push(Fine {
                        quantity: user_fines_redis.amount as f64,
                        reason: user_fines_redis.motive,
                    });
                }

                Ok(fine_list)
            }
            Err(_) => Err("Couldn't get users fines".to_owned()),
        }
    }
}
