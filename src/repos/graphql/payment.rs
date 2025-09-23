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
    repos::auth::utils::hashing_composite_key,
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
                        .json_get::<String, &str, RedisValue>(key.to_owned(), "$")
                        .unwrap(); // I will do it in one line, but nu uh, it would be unreadable

                    // for some reason redis gives all the info deserialize, so I have to do the
                    // serializion process my self
                    let nested_data = from_redis_value::<String>(&user_payment_raw).unwrap(); // first is

                    // ik that I could've made the direct mapping to the GraphQl object, but I
                    // rather using my own name standar for the redis keys and that Bryan manages
                    // the names as however he want's it
                    let user_payment_redis =
                        from_str::<Vec<RedisPayment>>(nested_data.as_str()).unwrap()[0].clone();
                    // cause
                    // of the way  of the way the json library works on redis, the objects follow a
                    // list type fetching, but as the db was planned, we where heading for a more
                    // key aproach overall, so that's why we need the cast (after all there will
                    // always be just one element)

                    // now we do the payment mapping

                    //payment_list.push(Payment {
                    //    payment_id: get_payment_key(key),
                    //    total_amount: user_payment_redis.quantity,
                    //    payment_date: user_payment_redis.date_created,
                    //    ticket_num: user_payment_redis.ticket_number,
                    //    commentary: user_payment_redis.comments,
                    //    photo: user_payment_redis.comprobante_bucket,
                    //    state: user_payment_redis.status,
                    //});
                }

                Ok(payment_list)
            }
            Err(_) => Err("Couldn't get users payments".to_string()),
        }
    }

    //TODO: refactor for the affiliate_key jus to be a simple array

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
