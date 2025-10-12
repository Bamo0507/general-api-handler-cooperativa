use actix_web::web::Data;
use r2d2::Pool;
use redis::{cmd, Client, Commands};

use crate::{
    models::{
        graphql::{Fine, FineStatus},
        redis::Fine as RedisFine,
    },
    repos::{auth::utils::hashing_composite_key, graphql::utils::get_multiple_models},
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
        amount: f64,
        status: FineStatus,
        loan_key: String,
    ) -> Result<String, String> {
        let mut con = &mut self.pool.get().expect("Couldn't connect to pool");

        //let db_access_token = hashing_composite_key(&[&access_token]);

        // check if the loan exist in the first place

        //if cmd("EXISTS")
        //    .arg(format!("users:{db_access_token}:fines:{}")) //Closests key-value we have at hand
        //    .query::<bool>(&mut con)
        //    .unwrap()
        //{}

        //if let Ok(keys) =
        //    con.scan_match::<String, String>(format!("users:{}:fines:*", db_access_token))
        //{}

        todo!()
    }
}
