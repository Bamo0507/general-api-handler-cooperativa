use actix_web::web;
use r2d2::Pool;
use redis::{Client, RedisError};

use crate::models::graphql::{Aporte, Cuota, Payment, PrestamoDetalles};

use super::utils::return_n_dummies;

pub struct PaymentRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl PaymentRepo {
    pub fn init(pool: web::Data<Pool<Client>>) -> PaymentRepo {
        return PaymentRepo { pool };
    }

    //TODO: implent true logic

    pub fn get_user_payments(&self, acess_token: String) -> Result<Vec<Payment>, String> {
        let con = self.pool.get().expect("Couldn't connect to pool");

        unimplemented!();
    }
}
