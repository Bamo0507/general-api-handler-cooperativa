use serde::{Deserialize, Serialize};

//TODO: refactor for different files

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub date_created: String,
    pub comprobante_bucket: String,
    pub ticket_number: String,
    pub status: String,
    pub quantity: f64,
    pub comments: String,
}

impl Default for Payment {
    fn default() -> Self {
        Payment {
            date_created: "0-00-0000".to_string(),
            comprobante_bucket: "/".to_string(),
            ticket_number: "000000000".to_string(),
            status: "NOT_PROCESS".to_string(),
            quantity: 0.00,
            comments: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub total_quota: i32,         // total couta needed
    pub base_needed_payment: f64, //initial value of the lone without fines
    pub payed: f64,
    pub debt: f64,
    pub total: f64,
    pub status: String, //TODO: ASk bryan how to do this
    pub reason: String,
}

impl Default for Loan {
    fn default() -> Self {
        Loan {
            total_quota: 0,
            base_needed_payment: 0.,
            payed: 0.,
            debt: 0.,
            total: 0.,
            status: "Not Done".to_owned(),
            reason: "None".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fine {
    pub amount: f32,
    pub motive: String,
}

impl Default for Fine {
    fn default() -> Self {
        Fine {
            amount: 0.,
            motive: "nu uh".to_owned(),
        }
    }
}
