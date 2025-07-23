use std::collections::HashMap;

use actix_web::web;
use r2d2::Pool;
use redis::{Client, Commands, RedisError};
use regex::Regex;

use crate::{
    models::graphql::{Affiliate, Aporte, Cuota, Payment, PaymentHistory, PrestamoDetalles},
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

    //TODO: keep with this later
    pub fn get_user_payments(&self, access_token: String) -> Result<Vec<Payment>, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        println!("db_key {}", db_access_token);
        match con.scan_match::<String, String>(format!("users:{}:payments:*", db_access_token)) {
            // Could be easier, but noo... Pedro wanted to do keys for everything
            Ok(keys) => {
                // Map to store all keys from the same payment
                Ok(Vec::new())
            }
            Err(_) => Err("Couldn't get users payments".to_string()),
        }
    }

    // This goes in the payment repo, only cause is an utililty endpoint for the Payments
    pub fn get_all_users_for_affiliates(&self) -> Result<Vec<Affiliate>, String> {
        let con = &mut self.pool.get().expect("Couldn't connect to pool");

        match con.scan_match::<&str, String>("affiliate_ids:*") {
            Ok(keys) => {
                let mut affiliates: Vec<Affiliate> = Vec::new();
                // TODO: see to refactor and generelize the regex part
                let regex = Regex::new(r"^(affiliate_ids+):([\w]+)*").unwrap();

                // TODO: see to refactor and generalize this
                for key in keys {
                    println!("{}", key);
                    let parsed_key = regex.captures(key.as_str()).unwrap();

                    // Why borrow checker, WHY?!?!?
                    // The equivalent of cloning
                    let name_con = &mut self.pool.get().expect("Couldn't connect to pool");

                    affiliates.push(Affiliate {
                        usuario_id: parsed_key[2].parse::<i32>().unwrap_or(0),
                        name: name_con
                            .get::<String, String>(format!(
                                "affiliate_ids:{}",
                                parsed_key[2].to_string()
                            ))
                            .unwrap_or("Not A Name".to_string()),
                    })
                }

                Ok(affiliates)
            }
            Err(_) => Err("Couldn't get users".to_string()),
        }
    }
}
