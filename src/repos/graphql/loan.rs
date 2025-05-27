use actix_web::web;
use r2d2::Pool;
use redis::{Client, RedisError};

use crate::models::graphql::{Codeudor, Loan, Pagare, PrestamoDetalles};

use super::utils::utils::return_n_dummies;

//it is use, but by referencing it, not directly
fn dummy_loan() -> Loan {
    return Loan {
        solicitante_id: 0000000,
        nombre: "John".to_string(),
        monto_total: 00.00,
        monto_cancelado: 00.00,
        motivo: "None".to_string(),
        tasa_interes: 00.00,
        fecha_solicitud: "0-00-0000".to_string(), //For parsing purposes
        plazo_meses: 0,
        meses_cancelados: 0,
        pagares: vec![Pagare {
            pagare: "Bucket String".to_string(),
            estado: "None".to_string(),
            comentarios_rechazo: "None".to_string(),
        }],
        codeudores: vec![Codeudor {
            nombre: "John".to_string(),
        }],
        mensualidad_prestamo: vec![PrestamoDetalles {
            numero_cuota: 0,
            monto_cuota: 00.00,
            fecha_vencimiento: "0-00-0000".to_string(),
            monto_pagado: 00.00,
            multa: 00.00,
        }],
    };
}

pub struct LoanRepo {
    pub pool: web::Data<Pool<Client>>,
}

//TODO: add error managment for redis
impl LoanRepo {
    pub fn init(pool: web::Data<Pool<Client>>) -> LoanRepo {
        return LoanRepo { pool };
    }

    //TODO: implent true logic
    pub fn get_user_loans(&self, user_id: String) -> Result<Vec<Loan>, String> {
        return Ok(return_n_dummies::<Loan>(&dummy_loan, 10));
    }

    //TODO: add later

    //pub fn add_ill_pay(&self, loan_id: String, ill_pay: Pagare) -> () {}

    //pub fn add_loan(&self, loan: Loan) -> () {}
}
