use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

use redis::{from_redis_value, Commands, JsonCommands, Value as RedisValue};
use serde_json::from_str;

use crate::repos::graphql::utils::get_multiple_models;
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
        get_multiple_models::<Loan, RedisLoan>(
            access_token,
            self.pool.clone(),
            "loans".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

    //pub fn add_ill_pay(&self, loan_id: String, ill_pay: Pagare) -> () {}

    //pub fn add_loan(&self, loan: Loan) -> () {}
}
