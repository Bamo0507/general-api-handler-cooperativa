use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

use redis::{from_redis_value, Commands, JsonCommands, Value as RedisValue};
use serde_json::from_str;

use crate::repos::graphql::utils::{get_multiple_models_by_id, get_multiple_models_by_pattern};
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
            Some(access_token),
            None,
            self.pool.clone(),
            "loans".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

    /// obtiene todos los préstamos de todos los socios
    pub fn get_all_loans(&self) -> Result<Vec<Loan>, String> {
        // usamos el helper que acepta un patrón porque necesitamos spannear todos los users
        // los otros helpers construyen patrón desde access token y no sirven para esto
        get_multiple_models_by_pattern::<Loan, RedisLoan>(
            "users:*:loans:*".to_string(),
            self.pool.clone(),
        )
    }

    pub fn create_loan(
        &self,
        affiliate_key: String,
        total_quota: i32,
        base_needed_payment: f64,
        interest_rate: f64,
        reason: String,
    ) -> Result<String, String> {
        let mut con = &mut self.pool.get().expect("Couldn't connect to pool");

        // obtenemos el db_affiliate_key desde el affiliate_key

        let db_affiliate_key = hashing_composite_key(&[&affiliate_key]);

        if let Ok(keys) =
            con.scan_match::<String, String>(format!("users:{}:loans:*", db_affiliate_key))
        {
            let keys_parsed: Vec<String> = keys.collect();

            // para crear el loan y evitar colisiones
            let loan_hash_key =
                hashing_composite_key(&[&keys_parsed.len().to_string(), &db_affiliate_key]);

            let con = &mut self.pool.get().expect("Couldn't connect to pool");

            let _: () = con
                .json_set(
                    format!("users:{}:loans:{}", db_affiliate_key, loan_hash_key),
                    "$",
                    &RedisLoan {
                        total_quota,
                        base_needed_payment,
                        payed: 0.0,
                        debt: base_needed_payment,
                        total: base_needed_payment,
                        status: "PENDING".to_owned(),
                        reason,
                        interest_rate,
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
