use actix_web::web;
use r2d2::Pool;
use redis::{Client, RedisError};

use redis::{from_redis_value, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde_json::from_str;

use crate::{
    models::{
        graphql::{Codeudor, Loan, Pagare, PrestamoDetalles},
        redis::Loan as RedisLoan,
    },
    repos::auth::utils::hashing_composite_key,
};

use super::utils::return_n_dummies;

pub struct LoanRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl LoanRepo {
    pub fn init(pool: web::Data<Pool<Client>>) -> LoanRepo {
        return LoanRepo { pool };
    }

    //TODO: refactor for generalize this kind of methods of get n thing

    // ! NOT FULLY TESTED, BUT IT SHOULD WORK

    //TODO: implent true logic
    pub fn get_user_loans(&self, access_token: String) -> Result<Vec<Loan>, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        match con.scan_match::<String, String>(format!("users:{}:loans:*", db_access_token)) {
            Ok(keys) => {
                let mut loans_list: Vec<Loan> = Vec::new();

                // conn for fetching payments
                let mut con = self.pool.get().expect("Couldn't connect to pool");

                for key in keys {
                    // We first fetch the raw data, first
                    let user_loan_raw = con
                        .json_get::<String, &str, RedisValue>(format!("{}", key), "$")
                        .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                    // for some reason redis gives all the info deserialize, so I have to do the
                    // serializion process my self
                    let nested_data =
                        from_redis_value::<String>(&user_loan_raw).unwrap_or_default(); // first is
                                                                                        // just the path, second is the actual data

                    // ik that I could've made the direct mapping to the GraphQl object, but I
                    // rather using my own name standar for the redis keys and that Bryan manages
                    // the names as however he want's it
                    let user_loan_redis =
                        from_str::<RedisLoan>(nested_data.as_str()).unwrap_or_default();
                    // that
                    // was just for getting the redis object, now I have to do the mapping

                    // now we do the loan mapping

                    loans_list.push(Loan {
                        qoutas: user_loan_redis.total_qouta,
                        payed: user_loan_redis.payed,
                        debt: user_loan_redis.debt,
                        total: user_loan_redis.total,
                        status: user_loan_redis.status,
                        reason: user_loan_redis.reason,
                    });
                }

                Ok(loans_list)
            }
            Err(_) => Err("Couldn't get users payments".to_string()),
        }
    }

    //TODO: add later

    //pub fn add_ill_pay(&self, loan_id: String, ill_pay: Pagare) -> () {}

    //pub fn add_loan(&self, loan: Loan) -> () {}
}
