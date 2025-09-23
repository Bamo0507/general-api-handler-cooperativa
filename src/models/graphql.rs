// TODO: Refactor all of them with default imp

use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

// Fields are in spanish, for easier parsing in bryan's side
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Loan {
    pub id: String,
    pub quotas: i32, // total couta needed
    pub payed: f64,
    pub debt: f64,
    pub total: f64,
    pub status: String, //TODO: ASk bryan how to do this
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Fine {
    pub id: String,
    pub quantity: f64,
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Payment {
    pub id: String,
    pub total_amount: f64,
    pub payment_date: String, // I'll pass it as a string, for not having parsing difficulties
    pub ticket_num: String,
    pub account_num: String,
    //pub banco_deposito: String, //Like this the same as the as ticker_num
    pub commentary: String,
    pub photo: String, // For bucket use
    pub state: String, // Following bryan's enums
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Affiliate {
    pub user_id: String,
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
    pub numero_quota: i32,
    pub monto_quota: f64,
    pub fecha_vencimiento: String, // I'll pass it as a string, for not having parsing difficulties
    pub monto_pagado: f64,
    pub multa: f64,
}

// ! As bryan sent me the model, it left for room for tons of overfetching, restructuring
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Aporte {
    pub monto: f64,
}


// --- SCRUM-255: Modelo de Quota de préstamo ---
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Quota {
    pub user_id: String,
    pub monto: f64,
    pub fecha_vencimiento: Option<String>,
    pub monto_pagado: f64,
    pub multa: f64,
    pub pagada_por: Option<String>, // bryan lo dijo porque en caso de que la Quota la pague otro usuario
    pub tipo: TipoQuota,
    pub loan_id: Option<String>, // de acá se debería sacar el nombre del prestamo, pero todavía no está implementado (así lo pidió bryan)
    pub extraordinaria: Option<bool>, // esto al crear, por logica de negocio va cambiar el monto si es extraordinaria o no
    pub pagada: Option<bool>, // SCRUM-255: campo para estado de pago
    pub numero_quota: Option<i32>, // Solo para préstamo
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct QuotaAfiliadoMensualResponse {
    pub identifier: String,
    pub user_id: String,
    pub monto: f64,
    pub nombre: String,
    pub fecha_vencimiento: String,
    pub extraordinaria: bool,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct QuotaPrestamoResponse {
    pub user_id: String,
    pub monto: f64,
    pub fecha_vencimiento: String,
    pub monto_pagado: f64,
    pub multa: f64,
    pub pagada_por: Option<String>,
    pub tipo: String,
    pub loan_id: Option<String>,
    pub pagada: bool,
    pub numero_quota: Option<i32>,
    pub nombre_prestamo: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, juniper::GraphQLEnum, PartialEq)]
pub enum TipoQuota {
    Prestamo,
    Afiliado,
}
