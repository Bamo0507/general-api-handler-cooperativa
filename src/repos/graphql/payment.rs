use crate::models::graphql::PaymentStatus;
use crate::models::GraphQLMappable;
use crate::repos::graphql::utils::get_multiple_models;
use crate::{
    models::{
        graphql::{Affiliate, Payment, PaymentHistory},
        redis::Payment as RedisPayment,
        PayedTo,
    },
    repos::{auth::utils::hashing_composite_key, graphql::utils::get_multiple_models_by_id},
};
use actix_web::web;
use chrono::Utc;
use r2d2::Pool;
use redis::{from_redis_value, Client, Commands, JsonCommands};
use regex::Regex;
use serde_json::from_str;
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
        get_multiple_models_by_id::<Payment, RedisPayment>(
            access_token,
            self.pool.clone(),
            "payments".to_owned(), // TODO: see a way to don't burn the keys
        )
    }

    /// Obtiene todos los pagos de todos los socios
    pub fn get_all_payments(&self) -> Result<Vec<Payment>, String> {
        // usamos el helper que acepta un patrón porque necesitamos spannear todos los users
        // los otros helpers construyen patrón desde access token y no sirven para esto
        crate::repos::graphql::utils::get_multiple_models_by_pattern::<Payment, RedisPayment>(
            "users:*:payments:*".to_string(),
            self.pool.clone(),
        )
    }

    // TODO: implement payment creation
    pub fn create_payment(
        &self,
        access_token: String,
        name: String,
        total_amount: f64,
        ticket_number: String,
        account_number: String,
        being_payed: Vec<PayedTo>,
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
            let payment_hash_key =
                hashing_composite_key(&[&keys_parsed.len().to_string(), &db_access_token]);

            let con = &mut self.pool.get().expect("Couldn't connect to pool");

            let date = Utc::now().date_naive().to_string();

            let _: () = con
                .json_set(
                    format!("users:{db_access_token}:payments:{payment_hash_key}"),
                    "$",
                    &RedisPayment {
                        name,
                        total_amount,
                        ticket_number,
                        date_created: date,
                        //TODO: add impl for bucket paths
                        comprobante_bucket: String::new(),
                        account_number,
                        comments: None,
                        status: "ON_REVISION".to_owned(),
                        being_payed,
                    },
                )
                .expect("PAYMENT CREATION: Couldn't Create Payment");
            return Ok("Payment Created".to_owned());
        }

        Err("PAYMENT CREATION: Couldn't Create Payment".to_owned())
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

                    let affiliate_con = &mut self.pool.get().expect("Couldn't connect to pool");

                    affiliates.push(Affiliate {
                        // user db_id
                        user_id: affiliate_con
                            .get::<String, String>(format!(
                                "users:{}:affiliate_key",
                                parsed_key[2].to_owned()
                            ))
                            .unwrap_or("Not Name Found".to_owned()),
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

    /// Aprueba o rechaza un pago por id, actualizando estado y comentario (si es REJECTED)
    pub async fn approve_or_reject_payment(
        &self,
        id: String,
        new_state: String,
        commentary: String,
    ) -> Result<Payment, String> {
        let mut con = self.pool.get().map_err(|_| "Couldn't connect to pool")?;

        // Buscamos todas las keys que correspondan al id: users:*:payments:{id}
        let pattern = format!("users:*:payments:{}", id);

        match con.scan_match::<String, String>(pattern.clone()) {
            Ok(keys) => {
                let key_vec: Vec<String> = keys.collect();

                if key_vec.is_empty() {
                    // Fallback: try the 'all' key behavior from before
                    let all_key = hashing_composite_key(&[&String::from("all")]);
                    let key = format!("users:{}:payments:{}", all_key, id);

                    // Obtener JSON del pago
                    let raw = con
                        .json_get::<String, &str, redis::Value>(key.clone(), "$")
                        .map_err(|_| "Error fetching payment")?;
                    let nested = from_redis_value::<String>(&raw).map_err(|_| "Error decoding redis value")?;
                    let mut parsed: Vec<RedisPayment> =
                        from_str(&nested).map_err(|_| "Error deserializing payment")?;
                    let mut redis_payment = parsed
                        .pop()
                        .ok_or_else(|| "Payment not found".to_string())?;

                    // Validar estado actual
                    let current_status = PaymentStatus::from_string(redis_payment.status.clone());
                    if current_status == PaymentStatus::Accepted || current_status == PaymentStatus::Rejected {
                        return Err("El pago ya está finalizado".to_string());
                    }

                    // Validar nuevo estado
                    let new_status = PaymentStatus::from_string(new_state.clone());
                    match new_status {
                        PaymentStatus::Accepted => {}
                        PaymentStatus::Rejected => {
                            if commentary.trim().is_empty() {
                                return Err("Se requiere comentario al rechazar el pago".to_string());
                            }
                        }
                        _ => return Err("Estado inválido, debe ser ACCEPTED o REJECTED".to_string()),
                    }

                    // Actualizar y persistir
                    redis_payment.status = new_status.as_str().to_owned();
                    if new_status == PaymentStatus::Rejected {
                        redis_payment.comments = Some(commentary);
                    }

                    con.json_set::<String, &str, _, ()>(key.clone(), "$", &redis_payment)
                        .map_err(|_| "Error updating payment")?;

                    // Mapear a GraphQL
                    let payment = redis_payment.to_graphql_type(key);
                    Ok(payment)
                } else {
                    // Tenemos una o más keys; actualizamos todas para mantenerlas sincronizadas
                    let mut con2 = self.pool.get().map_err(|_| "Couldn't connect to pool")?;

                    // Primero validamos que ninguna copia ya esté finalizada
                    for key in &key_vec {
                        let raw_res = con2.json_get::<String, &str, redis::Value>(key.clone(), "$");
                        let raw = match raw_res {
                            Ok(v) => v,
                            Err(_) => continue, // skip invalid
                        };
                        let nested = match from_redis_value::<String>(&raw) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let parsed_vec_res = from_str::<Vec<RedisPayment>>(nested.as_str());
                        let parsed_objects: Vec<RedisPayment> = match parsed_vec_res {
                            Ok(v) => v,
                            Err(_) => match from_str::<RedisPayment>(nested.as_str()) {
                                Ok(obj) => vec![obj],
                                Err(_) => continue,
                            },
                        };

                        for p in parsed_objects {
                            let current_status = PaymentStatus::from_string(p.status.clone());
                            if current_status == PaymentStatus::Accepted || current_status == PaymentStatus::Rejected {
                                return Err("El pago ya está finalizado".to_string());
                            }
                        }
                    }

                    // Validar nuevo estado
                    let new_status = PaymentStatus::from_string(new_state.clone());
                    match new_status {
                        PaymentStatus::Accepted => {}
                        PaymentStatus::Rejected => {
                            if commentary.trim().is_empty() {
                                return Err("Se requiere comentario al rechazar el pago".to_string());
                            }
                        }
                        _ => return Err("Estado inválido, debe ser ACCEPTED o REJECTED".to_string()),
                    }

                    // Actualizamos todas las copias y guardamos la primera mapeada para devolverla
                    let mut mapped_payment: Option<Payment> = None;
                    for key in key_vec {
                        // volver a leer y parsear (para obtener el objeto)
                        let raw = match con2.json_get::<String, &str, redis::Value>(key.clone(), "$") {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let nested = match from_redis_value::<String>(&raw) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let mut parsed: Vec<RedisPayment> = match from_str(&nested) {
                            Ok(v) => v,
                            Err(_) => match from_str::<RedisPayment>(&nested) {
                                Ok(obj) => vec![obj],
                                Err(_) => continue,
                            },
                        };
                        if parsed.is_empty() {
                            continue;
                        }

                        let mut redis_payment = parsed.pop().unwrap();
                        redis_payment.status = new_status.as_str().to_owned();
                        if new_status == PaymentStatus::Rejected {
                            redis_payment.comments = Some(commentary.clone());
                        }

                        con2.json_set::<String, &str, _, ()>(key.clone(), "$", &redis_payment)
                            .map_err(|_| "Error updating payment")?;

                        if mapped_payment.is_none() {
                            mapped_payment = Some(redis_payment.to_graphql_type(key.clone()));
                        }
                    }

                    mapped_payment.ok_or_else(|| "Payment not found".to_string())
                }
            }
            Err(_) => Err("Couldn't scan for payment keys".to_string()),
        }
    }
}
