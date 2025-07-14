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

    /// giving the acess token, this returns the an Object of PaymentHistory of that "user"
    pub fn get_user_history(&self, access_token: String) -> Result<PaymentHistory, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        let payed_to_capital = match con
            .get::<String, String>(format!("users:{}:payed_to_capital", db_access_token))
        {
            Ok(val) => val.parse::<f64>().unwrap_or(0.0),
            Err(_) => return Err("Couldnt Get Payed To Capital".to_string()),
        };

        let owed_capital =
            match con.get::<String, String>(format!("users:{}:owed_capital", db_access_token)) {
                Ok(val) => val.parse::<f64>().unwrap_or(0.0),
                Err(_) => return Err("Couldnt Get Owed Capital".to_string()),
            };

        Ok(PaymentHistory {
            payed_to_capital,
            owed_capital,
        })
    }

    pub fn get_user_payments(&self, access_token: String) -> Result<Vec<Payment>, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        // get all payments keys
        unimplemented!();
    }
}
