use actix_web::web::Data;
use r2d2::Pool;
use redis::{cmd, Client, Commands, JsonCommands};

use crate::{
    models::{
        graphql::{Fine, FineStatus},
        redis::Fine as RedisFine,
    },
    repos::{
        auth::utils::hashing_composite_key,
        graphql::utils::{get_db_access_token_with_affiliate_key, get_multiple_models},
    },
};

pub struct FineRepo {
    pub pool: Data<Pool<Client>>,
}

impl FineRepo {
    pub fn get_user_fines(&self, access_token: String) -> Result<Vec<Fine>, String> {
        get_multiple_models::<Fine, RedisFine>(
            access_token,
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
}
