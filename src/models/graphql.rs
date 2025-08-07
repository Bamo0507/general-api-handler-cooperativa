// TODO: Refactor all of them with default imp

use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

// Fields are in spanish, for easier parsing in bryan's side
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Loan {
    pub solicitante_id: i32,
    pub nombre: String,
    pub monto_total: f64,
    pub monto_cancelado: f64,
    pub motivo: String,
    pub tasa_interes: f64,
    pub fecha_solicitud: String, // For parsing purposes
    pub plazo_meses: i32,
    pub meses_cancelados: i32,
    pub codeudores: Vec<Codeudor>,
    pub mensualidad_prestamo: Vec<PrestamoDetalles>,
    pub pagares: Vec<Pagare>,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Payment {
    pub payment_id: String,
    pub monto_total: f64,
    pub fecha_pago: String, // I'll pass it as a string, for not having parsing difficulties
    pub num_boleta: String,
    //pub banco_deposito: String, //Like this the same as the as ticker_num
    pub comentarios: String,
    pub foto: String,   // For bucket use
    pub estado: String, // Following bryan's enums
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Affiliate {
    pub usuario_id: i32,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct PaymentHistory {
    /// the value that brings
    pub payed_to_capital: f64,
    /// The capital that the user owes in total
    pub owed_capital: f64,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Pagare {
    //pub prestamo_id i32 //! Redundant, not adding it
    pub pagare: String,              //For the bucket
    pub estado: String,              //Following Bryan's rules
    pub comentarios_rechazo: String, //empty string for not a value
}

// ! As bryan sent me the model, it left for room for tons of overfetching, restructuring
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Codeudor {
    pub nombre: String,
}

// ! As bryan sent me the model, it left for room for tons of overfetching, restructuring
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct PrestamoDetalles {
    pub numero_cuota: i32,
    pub monto_cuota: f64,
    pub fecha_vencimiento: String, // I'll pass it as a string, for not having parsing difficulties
    pub monto_pagado: f64,
    pub multa: f64,
}

// ! As bryan sent me the model, it left for room for tons of overfetching, restructuring
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Aporte {
    pub monto: f64,
}

// ! As bryan sent me the model, it left for room for tons of overfetching, restructuring
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Cuota {
    pub monto_couta: f64,
    pub fecha_vencimiento: String,
    pub monto_pagado: f64,
    pub multa: f64,
}
