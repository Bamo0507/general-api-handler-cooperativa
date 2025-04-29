use super::utils::utils::return_n_dummies;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

//Fields are in spanish, for easier parsing in bryan's side
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Payment {
    usuario_id: i32,
    monto_total: f64,
    fecha_pago: String, //I'll pass it as a string, for not having parsing difficulties
    num_boleta: String,
    banco_deposito: String,
    //cuotas: Vector<Cuotas> TODO: make later
    //prestamos: Vector<PrestamoDetalle> TODO: make later
    //aporte_capital: Vector<Aporte> TODO: make it later
    comentarios: String,
    foto: String,   //For bucket use
    estado: String, //Following bryan's enums
}

//it is use, but by referencing it, not directly
fn DummyPayment() -> Payment {
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
    pub fn get_history(&self) -> Vec<Payment> {
        return return_n_dummies::<Payment>(&DummyPayment, 10);
    }
}
