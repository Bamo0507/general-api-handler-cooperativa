use crate::models::graphql::Payment;

use super::utils::utils::return_n_dummies;

//it is use, but by referencing it, not directly
fn dummy_payment() -> Payment {
    return Payment {
        usuario_id: 000000000,
        monto_total: 00.00,
        fecha_pago: "0-00-0000".to_string(),
        num_boleta: "0a0a0a0a0a0a".to_string(),
        banco_deposito: "0a0a0a0a0a".to_string(),
        comentarios: "None Comment LMAO".to_string(),
        foto: "Bucket URL".to_string(),
        estado: "None".to_string(),
    };
}

//TODO: Add respective pools
pub struct PaymentRepo {}

impl PaymentRepo {
    pub fn init() -> PaymentRepo {
        return PaymentRepo {};
    }

    //TODO: implent true logic

    pub fn get_all(&self) -> Vec<Payment> {
        return return_n_dummies::<Payment>(&dummy_payment, 10);
    }
}
