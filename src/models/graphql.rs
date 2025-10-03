// TODO: Refactor all of them with default imp

use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

use crate::models::FromString;

#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum QuotaType {
    Prestamo,
    Afiliado,
}

#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum PaymentStatus {
    OnRevision,
    Rejected,
    Accepted,
    ParsedError,
}

impl PaymentStatus {
    pub fn from_string(raw_status: String) -> PaymentStatus {
        match raw_status.to_uppercase().as_str() {
            "ON_REVISION" => PaymentStatus::OnRevision,
            "REJECTED" => PaymentStatus::Rejected,
            "ACCEPTED" => PaymentStatus::Accepted,
            _ => PaymentStatus::ParsedError,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum LoanStatus {
    Overdue,
    Active,
    Pending,
    Payed,
    ParsedError,
}

impl LoanStatus {
    pub fn from_string(raw_status: String) -> LoanStatus {
        match raw_status.to_uppercase().as_str() {
            "OVERDUE" => LoanStatus::Overdue,
            "PENDING" => LoanStatus::Pending,
            "ACTIVE" => LoanStatus::Active,
            "PAYED" => LoanStatus::Payed,
            _ => LoanStatus::ParsedError,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Loan {
    pub id: String,
    pub quotas: i32, // total couta needed
    pub payed: f64,
    pub debt: f64,
    pub total: f64,
    pub status: LoanStatus, //TODO: ASk bryan how to do this
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
    pub commentary: String,
    pub photo: String,        // For bucket use
    pub state: PaymentStatus, // Following bryan's enums
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

/// Modelo unificado de Quota para manejar tanto cuotas de afiliado como de préstamo
/// Compatible con GraphQL y todas las operaciones del sistema
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Quota {
    /// ID del usuario (debe coincidir con access_token para dummy data)
    pub user_id: String,
    /// Monto de la cuota
    pub amount: f64,
    /// Fecha de vencimiento en formato YYYY-MM-DD
    pub exp_date: Option<String>,
    /// Monto ya pagado de la cuota (0.0 para nuevas cuotas)
    pub monto_pagado: Option<f64>,
    /// Multa aplicada a la cuota (0.0 para nuevas cuotas)
    pub multa: Option<f64>,
    /// Usuario que pagó la cuota (para pagos por terceros)
    pub pay_by: Option<String>,
    /// Tipo de cuota: Prestamo o Afiliado
    pub quota_type: QuotaType,
    /// ID del préstamo (solo para cuotas de préstamo, SHA256 para dummy data)
    pub loan_id: Option<String>,
    /// Indica si es una cuota extraordinaria
    pub is_extraordinary: Option<bool>,
    /// Estado de pago de la cuota
    pub payed: Option<bool>,
    /// Número de la cuota dentro del préstamo (solo para préstamos)
    pub quota_number: Option<i32>,
    /// Nombre del préstamo para mostrar en frontend
    pub nombre_prestamo: Option<String>,
    /// Nombre del usuario para mostrar en frontend
    pub nombre_usuario: Option<String>,
    /// Identificador único para frontend (formato: "Nombre - Mes Año")
    pub identifier: Option<String>,
}

