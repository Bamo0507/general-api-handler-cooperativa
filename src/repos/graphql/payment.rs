use actix_web::web;
use chrono::{Local, NaiveDateTime, Utc};
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde_json::from_str;

use crate::{
    models::{
        graphql::{Affiliate, Payment, PaymentHistory, PaymentStatus},
        redis::Payment as RedisPayment,
    },
    repos::{auth::utils::hashing_composite_key, graphql::utils::get_multiple_models},
};
pub struct PaymentRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl PaymentRepo {
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
        get_multiple_models::<Payment, RedisPayment>(
            access_token,
            self.pool.clone(),
            "payments".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

        /// Obtiene todos los pagos de todos los socios
        pub fn get_all_payments(&self) -> Result<Vec<Payment>, String> {
            // Se fundamenta en el patrón de get_multiple_models usado en get_user_payments
            // Para obtener todos los pagos, se puede usar una clave global o escanear todas las claves de pagos
            // Aquí se usa una clave global "all" para mantener el patrón
            get_multiple_models::<Payment, RedisPayment>(
                "all".to_owned(),
                self.pool.clone(),
                "payments".to_owned(),
            )
        }

    // TODO: implement payment creation
    pub fn create_payment(
        &self,
        access_token: String,
        comments: String,
        amount: f64,
        ticket_number: String,
        account_number: String,
        // TODO: refactor for being schema compliant
        //quotas: Vec<(String, f32)>,
        //loans: Vec<(String, f32)>,
        //fines: Vec<(String, f32)>,
        //affiliates_owed_capitals: Vec<(String, f32)>,
    ) -> Result<String, String> {
        // for the moment I'll just implement it as for creating a payment without the relation
        // wich the other fields

        let con = &mut self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        // we check how many payments we have

        if let Ok(keys) =
            con.scan_match::<String, String>(format!("users:{}:payments:*", db_access_token))
        {
            let keys_parsed: Vec<String> = keys.collect();

            // for creating the payment and not having collissions
            let payment_hash_key = hashing_composite_key(&[&keys_parsed.len().to_string()]);

            let con = &mut self.pool.get().expect("Couldn't connect to pool");

            let date = Utc::now().date_naive().to_string();

            //TODO: implement relation for fines, quootas, etc

            let _: () = con
                .json_set(
                    format!("users:{db_access_token}:payments:{payment_hash_key}"),
                    "$",
                    &RedisPayment {
                        quantity: amount,
                        ticket_number,
                        date_created: date,
                        comprobante_bucket: String::new(),
                        account_number,
                        comments,
                        status: "ON_REVISION".to_owned(),
                    },
                )
                .expect("PAYMENT CREATION: Couldn't Create payment");
            return Ok("Payment Created".to_owned());
        }

        Err("PAYMENT CREATION: Couldn't Create payment".to_owned())
    }

    // This goes in the payment repo, only cause is an utililty endpoint for the Payments
    pub fn get_all_users_for_affiliates(&self) -> Result<Vec<Affiliate>, String> {
        let con = &mut self.pool.get().expect("Couldn't connect to pool");

        match con.scan_match::<&str, String>("users:*:affiliate_key") {
            Ok(keys) => {
                let mut affiliates: Vec<Affiliate> = Vec::new();
                let regex = Regex::new(r"(users):(\w+):(affiliate_key)").unwrap();

                for key in keys {
                    let parsed_key = regex.captures(key.as_str()).unwrap();

                    // Why borrow checker, WHY?!?!?
                    // The equivalent of cloning
                    let name_con = &mut self.pool.get().expect("Couldn't connect to pool");

                    affiliates.push(Affiliate {
                        // user db_id
                        user_id: parsed_key[2].to_owned(),
                        name: name_con
                            .get::<String, String>(format!(
                                "users:{}:complete_name",
                                parsed_key[2].to_owned()
                            ))
                            .unwrap_or("Not Name Found".to_owned()),
                    })
                }

                Ok(affiliates)
            }
            Err(_) => Err("Couldn't get users".to_string()),
        }
    }
}
