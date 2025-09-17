use actix_web::web;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands, Value as RedisValue};
use regex::Regex;
use serde_json::from_str;

use crate::{
    models::{
        graphql::{Affiliate, Payment, PaymentHistory},
        redis::Payment as RedisPayment,
    },
    repos::{auth::utils::hashing_composite_key, graphql::utils::get_payment_key},
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

    //TODO: keep with this later
    pub fn get_user_payments(&self, access_token: String) -> Result<Vec<Payment>, String> {
        let mut con = self.pool.get().expect("Couldn't connect to pool");

        let db_access_token = hashing_composite_key(&[&access_token]);

        match con.scan_match::<String, String>(format!("users:{}:payments:*", db_access_token)) {
            Ok(keys) => {
                let mut payment_list: Vec<Payment> = Vec::new();

                // conn for fetching payments
                let mut con = self.pool.get().expect("Couldn't connect to pool");

                for key in keys {
                    // We first fetch the raw data, first
                    let user_payment_raw = con
                        .json_get::<String, &str, RedisValue>(format!("{}", key), "$")
                        .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                    // for some reason redis gives all the info deserialize, so I have to do the
                    // serializion process my self
                    let nested_data =
                        from_redis_value::<String>(&user_payment_raw).unwrap_or_default(); // first is
                                                                                           // just the path, second is the actual data

                    // ik that I could've made the direct mapping to the GraphQl object, but I
                    // rather using my own name standar for the redis keys and that Bryan manages
                    // the names as however he want's it
                    let user_payment_redis =
                        from_str::<RedisPayment>(nested_data.as_str()).unwrap_or_default();
                    // that
                    // was just for getting the redis object, now I have to do the mapping

                    // now we do the payment mapping

                    payment_list.push(Payment {
                        payment_id: get_payment_key(key),
                        monto_total: user_payment_redis.quantity,
                        fecha_pago: user_payment_redis.date_created,
                        num_boleta: user_payment_redis.ticket_number,
                        comentarios: user_payment_redis.comments,
                        foto: user_payment_redis.comprobante_bucket,
                        estado: user_payment_redis.status,
                    });
                }

                Ok(payment_list)
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
                                parsed_key[2].to_owned()
                            ))
                            .unwrap_or("Not A Name".to_owned()),
                    })
                }

                Ok(affiliates)
            }
            Err(_) => Err("Couldn't get users".to_string()),
        }
    }
}
