use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

use redis::{from_redis_value, Commands, JsonCommands, Value as RedisValue};
use serde_json::from_str;

use crate::{
    models::{
        graphql::{Loan},
        redis::Loan as RedisLoan,
    },
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
                        .json_get::<String, &str, RedisValue>(key.to_owned(), "$")
                        .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                    println!("{user_loan_raw:?}");
                    // for some reason redis gives all the info deserialize, so I have to do the
                    // serializion process my self
                    let nested_data = from_redis_value::<String>(&user_loan_raw).unwrap(); // first is
                                                                                           // just the path, second is the actual data

                    // ik that I could've made the direct mapping to the GraphQl object, but I
                    // rather using my own name standar for the redis keys and that Bryan manages
                    // the names as however he want's it
                    let user_loan_redis =
                        from_str::<Vec<RedisLoan>>(nested_data.as_str()).unwrap()[0].clone(); // cause
                                                                                              // of the way  of the way the json library works on redis, the objects follow a
                                                                                              // list type fetching, but as the db was planned, we where heading for a more
                                                                                              // key aproach overall, so that's why we need the cast (after all there will
                                                                                              // always be just one element)

                    // now we do the loan mapping

                    loans_list.push(Loan {
                        quotas: user_loan_redis.total_quota,
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
