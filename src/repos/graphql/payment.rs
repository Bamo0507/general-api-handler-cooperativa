use r2d2::Pool;
use redis::{Client, RedisError};

use crate::models::graphql::{Aporte, Cuota, Payment, PrestamoDetalles};

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
        aporte_capital: vec![Aporte { monto: 00.00 }],
        cuotas: vec![Cuota {
            monto_couta: 00.00,
            fecha_vencimiento: "0-00-000".to_string(),
            monto_pagado: 00.00,
            multa: 00.00,
        }],
        prestamos: vec![PrestamoDetalles {
            numero_cuota: 0,
            monto_cuota: 00.00,
            fecha_vencimiento: "0-00-0000".to_string(),
            monto_pagado: 00.00,
            multa: 00.00,
        }],
    };
}

pub struct PaymentRepo {
    pub pool: Pool<Client>,
}

//TODO: add error managment for redis
impl PaymentRepo {
    pub fn init(pool: Pool<Client>) -> PaymentRepo {
        return PaymentRepo { pool };
    }

    //TODO: implent true logic

    pub fn get_user_payments(&self, user_id: String) -> Result<Vec<Payment>, String> {
        return Ok(return_n_dummies::<Payment>(&dummy_payment, 10));
    }
}
