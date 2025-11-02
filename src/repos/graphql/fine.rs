use actix_web::web::Data;
use r2d2::Pool;
use redis::{cmd, from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde_json::from_str;

use crate::{
    models::{
        graphql::{Fine, FineStatus, UsersWithFines},
        redis::Fine as RedisFine,
    },
    repos::{
        auth::utils::hashing_composite_key,
        graphql::utils::{get_db_access_token_with_affiliate_key, get_multiple_models_by_id},
    },
};

pub struct FineRepo {
    pub pool: Data<Pool<Client>>,
}

impl FineRepo {
    pub fn get_user_fines(&self, access_token: String) -> Result<Vec<Fine>, String> {
        get_multiple_models_by_id::<Fine, RedisFine>(
            Some(access_token),
            None,
            self.pool.clone(),
            "fines".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

    pub fn create_fine(
        &self,
        affiliate_key: String,
        amount: f32,
        motive: String,
    ) -> Result<String, String> {
        let mut con = &mut self.pool.get().expect("Couldn't connect to pool");

        // check if the loan exist in the first place

        let db_access_token =
            get_db_access_token_with_affiliate_key(affiliate_key, self.pool.clone())?;

        if let Ok(keys) =
            con.scan_match::<String, String>(format!("users:{}:fines:*", db_access_token))
        {
            let keys_parsed: Vec<String> = keys.collect();

            // for creating the fine and not having collissions
            let fine_hash_key =
                hashing_composite_key(&[&keys_parsed.len().to_string(), &db_access_token]);

            let con = &mut self.pool.get().expect("Couldn't connect to pool");

            let _: () = con
                .json_set(
                    format!("users:{}:fines:{}", db_access_token, fine_hash_key),
                    "$",
                    &RedisFine {
                        amount,
                        motive,
                        status: "UNPAID".to_owned(),
                    },
                )
                .expect("FINE CREATION: Couldn't Create Fine");
            return Ok("Fine Createad".to_owned());
        }

        Err("FINE CREATION: Couldn't Create Fine".to_owned())
    }

    pub fn edit_fine(
        &self,
        fine_key: String,
        new_amount: Option<f64>,
        new_motive: Option<String>,
        new_status: Option<FineStatus>,
    ) -> Result<String, String> {
        let mut con = &mut self.pool.get().expect("Couldn't connect to pool");

        // we search the specific fine

        match con.scan_match::<String, String>(format!("users:*:fines:{}", fine_key)) {
            // there should be only one key
            Ok(mut keys) => {
                // we grab the only needed key
                let key = keys.next().unwrap();

                // the old boilerplate for getting the json value in rust

                // we get the latest fine
                let old_fine_raw = con
                    .json_get::<String, &str, RedisValue>(key.clone(), "$")
                    .unwrap();

                let nested_data = from_redis_value::<String>(&old_fine_raw).unwrap();

                let old_fine_parsed =
                    from_str::<Vec<RedisFine>>(nested_data.as_str()).unwrap()[0].clone();

                let new_amount = if new_amount.is_some() {
                    new_amount
                } else {
                    Some(old_fine_parsed.amount as f64)
                };

                let new_motive = if new_motive.is_some() {
                    new_motive
                } else {
                    Some(old_fine_parsed.motive)
                };

                let new_status = if new_status.is_some() {
                    new_status
                } else {
                    Some(FineStatus::from_string(old_fine_parsed.status))
                };

                let _: () = con
                    .json_set(
                        &key,
                        "$",
                        &RedisFine {
                            amount: new_amount.unwrap() as f32,
                            motive: new_motive.unwrap(),
                            status: new_status.unwrap().to_string(),
                        },
                    )
                    .expect("FINE CREATION: Couldn't Create Fine");
                return Ok("Fine updated".to_owned());
            }
            Err(_) => {
                return Err("Couldn't update fine".to_owned());
            }
        }
    }

    pub fn get_users_with_there_fines(&self) -> Result<Vec<Fine>, String> {
        let mut con_for_users_key = &mut self.pool.get().expect("Couldn't connect to pool");

        // we get first all the user db id
        match con_for_users_key.scan_match::<&str, String>("users:*:complete_name") {
            Ok(users_keys) => {
                let mut users_with_fines: Vec<UsersWithFines> = Vec::new();
                let regex = Regex::new(r"(users):(\w+):(complete_name)").unwrap();

                for user_key in users_keys {
                    let parsed_key = regex.captures(user_key.as_str()).unwrap();

                    let name_con = &mut self.pool.get().expect("Couldn't connect to pool");

                    let affiliate_con = &mut self.pool.get().expect("Couldn't connect to pool");
                }

                todo!()
            }
            Err(_) => {
                return Err("Couldn't update fine".to_owned());
            }
        }
    }
}
