use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

use redis::{from_redis_value, Commands, JsonCommands, Value as RedisValue};
use serde_json::from_str;

use crate::repos::graphql::utils::get_multiple_models_by_id;
use crate::{
    models::{graphql::Loan, redis::Loan as RedisLoan},
    repos::auth::utils::hashing_composite_key,
};

pub struct LoanRepo {
    pub pool: Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl LoanRepo {
    //TODO: refactor for generalize this kind of methods of get n thing

    // ! NOT FULLY TESTED, BUT IT SHOULD WORK

    //TODO: implent true logic
    pub fn get_user_loans(&self, access_token: String) -> Result<Vec<Loan>, String> {
        get_multiple_models_by_id::<Loan, RedisLoan>(
            access_token,
            self.pool.clone(),
            "loans".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

    pub fn create_loan(
        &self,
        access_token: String,
        total_quota: i32,
        base_needed_payment: f64,
        reason: String,
    ) -> Result<String, String> {
        let mut con = &mut self.pool.get().expect("Couldn't connect to pool");

        // obtenemos el db_access_token desde el access_token

        let db_access_token = hashing_composite_key(&[&access_token]);

        if let Ok(keys) =
            con.scan_match::<String, String>(format!("users:{}:loans:*", db_access_token))
        {
            let keys_parsed: Vec<String> = keys.collect();

            // para crear el loan y evitar colisiones
            let loan_hash_key =
                hashing_composite_key(&[&keys_parsed.len().to_string(), &db_access_token]);

            let con = &mut self.pool.get().expect("Couldn't connect to pool");

            let _: () = con
                .json_set(
                    format!("users:{}:loans:{}", db_access_token, loan_hash_key),
                    "$",
                    &RedisLoan {
                        total_quota,
                        base_needed_payment,
                        payed: 0.0,
                        debt: base_needed_payment,
                        total: base_needed_payment,
                        status: "PENDING".to_owned(),
                        reason,
                    },
                )
                .expect("LOAN CREATION: Couldn't Create Loan");
            return Ok("Loan Created".to_owned());
        }

        Err("LOAN CREATION: Couldn't Create Loan".to_owned())
    }

    //pub fn add_ill_pay(&self, loan_id: String, ill_pay: Pagare) -> () {}

    //pub fn add_loan(&self, loan: Loan) -> () {}
}
