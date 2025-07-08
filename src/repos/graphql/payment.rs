use actix_web::web;
use r2d2::Pool;
use redis::{Client, Commands, RedisError};

use crate::{
    models::graphql::{Aporte, Cuota, Payment, PaymentHistory, PrestamoDetalles},
    repos::auth::utils::hashing_composite_key,
};

pub struct PaymentRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl PaymentRepo {
    pub fn init(pool: web::Data<Pool<Client>>) -> PaymentRepo {
        PaymentRepo { pool }
    }

    pub fn get_user_history(&self, acess_token: String) -> Result<PaymentHistory, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&acess_token]);

        //TODO: see a way of reducing boiler plate for error handling

        let payed_to_capital =
            match con.get::<String, f64>(format!("users:{}:payed_to_capital", db_access_token)) {
                Ok(val) => val,
                Err(_) => return Err("Couldnt Get Payed To Capital".to_string()),
            };

        let owed_capital =
            match con.get::<String, f64>(format!("users:{}:payed_to_capital", db_access_token)) {
                Ok(val) => val,
                Err(_) => return Err("Couldnt Get Payed To Capital".to_string()),
            };

        Ok(PaymentHistory {
            payed_to_capital,
            owed_capital,
        })
    }
}
