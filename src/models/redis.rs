use serde::{Deserialize, Serialize};

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
