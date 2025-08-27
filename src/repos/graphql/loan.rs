use actix_web::web;
use r2d2::Pool;
use redis::{Client, RedisError};

use crate::models::graphql::{Codeudor, Loan, Pagare, PrestamoDetalles};

use super::utils::return_n_dummies;

pub struct LoanRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl LoanRepo {
    pub fn init(pool: web::Data<Pool<Client>>) -> LoanRepo {
        return LoanRepo { pool };
    }

    //TODO: implent true logic
    pub fn get_user_loans(&self, user_id: String) -> Result<Vec<Loan>, String> {
        todo!();
    }

    //TODO: add later

    //pub fn add_ill_pay(&self, loan_id: String, ill_pay: Pagare) -> () {}

    //pub fn add_loan(&self, loan: Loan) -> () {}
}
