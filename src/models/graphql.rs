// TODO: Refactor all of them with default imp

use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum PaymentType {
    Loan,
    Quota,
    Fine,
    ParsedError,
}

impl PaymentType {
    pub fn from_string(raw_status: String) -> PaymentType {
        match raw_status.to_uppercase().as_str() {
            "LOAN" => PaymentType::Loan,
            "QUOTA" => PaymentType::Quota,
            "FINE" => PaymentType::Fine,
            _ => PaymentType::ParsedError,
        }
    }
}

impl ToString for PaymentType {
    fn to_string(&self) -> String {
        match self {
            PaymentType::Loan => "LOAN".to_owned(),
            PaymentType::Quota => "QUOTA".to_owned(),
            PaymentType::Fine => "FINE".to_owned(),
            PaymentType::ParsedError => "PARSED_ERROR".to_owned(),
        }
    }
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

    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::OnRevision => "ON_REVISION",
            PaymentStatus::Rejected => "REJECTED",
            PaymentStatus::Accepted => "ACCEPTED",
            PaymentStatus::ParsedError => "PARSED_ERROR",
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

#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum FineStatus {
    Paid,
    Unpaid,
    ParsedError,
}

impl FineStatus {
    pub fn from_string(raw_status: String) -> FineStatus {
        match raw_status.to_uppercase().as_str() {
            "PAID" => FineStatus::Paid,
            "UNPAID" => FineStatus::Unpaid,
            "UPAID" => FineStatus::Unpaid,
            _ => FineStatus::ParsedError,
        }
    }
}

impl ToString for FineStatus {
    fn to_string(&self) -> String {
        match self {
            FineStatus::Paid => "PAID".to_owned(),
            FineStatus::Unpaid => "UNPAID".to_owned(),
            _ => "ERROR".to_owned(),
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
    pub amount: f64,
    pub status: FineStatus,
    pub reason: String,
    // nombre de quien presentó la multa (viene del complete_name del usuario)
    pub presented_by_name: String,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct UsersWithFines {
    pub complete_name: String,
    pub user_id: String,
    pub fines: Vec<Fine>,
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Payment {
    pub id: String,
    pub name: String,
    pub total_amount: f64,
    pub payment_date: String, // I'll pass it as a string, for not having parsing difficulties
    pub ticket_num: String,
    pub account_num: String,
    pub commentary: Option<String>,
    pub photo: String,        // For bucket use
    pub state: PaymentStatus, // Following bryan's enums
    // lo que el usuario está pagando con este pago (préstamos, cuotas, multas, etc)
    // viene del array being_payed que manda el usuario al crear el pago
    pub being_payed: Vec<crate::models::PayedTo>,
    // nombre de quien presentó/creó el pago (viene del complete_name del usuario con el access_token)
    pub presented_by_name: String,
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

// implementaciones del trait WithPresenterName para que el helper genérico
// pueda asignar el nombre del presentador a estos modelos
use crate::models::WithPresenterName;

impl WithPresenterName for Payment {
    fn set_presenter_name(&mut self, name: String) {
        self.presented_by_name = name;
    }
}

impl WithPresenterName for Fine {
    fn set_presenter_name(&mut self, name: String) {
        self.presented_by_name = name;
    }
}
