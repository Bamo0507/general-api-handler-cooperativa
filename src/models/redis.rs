use serde::{Deserialize, Serialize};

use crate::{
    models::{
        graphql::{
            Fine as GraphQLFine, FineStatus, Loan as GraphQLLoan, LoanStatus,
            Payment as GraphQLPayment, PaymentStatus,
        },
        GraphQLMappable, PayedTo,
    },
    repos::graphql::utils::get_key,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub date_created: String,
    pub account_number: String,
    pub total_amount: f64,
    pub name: String,
    pub comments: Option<String>, // it will be added if the directive sends it
    pub comprobante_bucket: String,
    pub ticket_number: String,
    pub status: String,
    pub being_payed: Vec<PayedTo>,
}

impl Default for Payment {
    fn default() -> Self {
        Payment {
            name: "none".to_owned(),
            date_created: "0-00-0000".to_owned(),
            comprobante_bucket: "/".to_owned(),
            account_number: "000000000".to_owned(),
            ticket_number: "000000000".to_owned(),
            status: "NOT_PROCESS".to_owned(),
            total_amount: 0.00,
            comments: Some("".to_owned()),
            being_payed: vec![PayedTo::default()],
        }
    }
}

impl GraphQLMappable<GraphQLPayment> for Payment {
    fn to_graphql_type(&self, key: String) -> GraphQLPayment {
        GraphQLPayment {
            id: get_key(key, "payments".to_owned()),
            name: (*self.name).to_owned(),
            total_amount: self.total_amount,
            account_num: (*self.account_number).to_string(),
            payment_date: (*self.date_created).to_string(),
            ticket_num: (*self.ticket_number).to_string(),
            commentary: self.comments.clone(), // f*** options, can't do low level stuff some times
            photo: (*self.comprobante_bucket).to_string(),
            state: PaymentStatus::from_string((*self.status).to_string()),
            // clonamos el array de being_payed del redis model
            being_payed: self.being_payed.clone(),
            // el nombre real se fetchea después en el repo con el helper genérico enrich_with_presenter_names
            // acá ponemos el default porque este trait no tiene acceso al pool de redis
            presented_by_name: crate::models::DEFAULT_PRESENTER_NAME.to_string(),
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

impl GraphQLMappable<GraphQLLoan> for Loan {
    fn to_graphql_type(&self, key: String) -> GraphQLLoan {
        GraphQLLoan {
            id: get_key(key, "loans".to_owned()),
            quotas: self.total_quota,
            payed: self.payed,
            debt: self.debt,
            total: self.total,
            status: LoanStatus::from_string((*self.status).to_string()),
            reason: (*self.reason).to_string(),
            // campo que requiere contexto adicional se llena con default aquí
            // solo get_all_loans lo llena correctamente con datos de redis
            presented_by_name: "N/A".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fine {
    pub amount: f32,
    pub motive: String,
    pub status: String,
}

impl Default for Fine {
    fn default() -> Self {
        Fine {
            amount: 0.,
            status: "UNPAID".to_owned(),
            motive: "nu uh".to_owned(),
        }
    }
}

impl GraphQLMappable<GraphQLFine> for Fine {
    fn to_graphql_type(&self, key: String) -> GraphQLFine {
        GraphQLFine {
            id: get_key(key, "fines".to_owned()),
            status: FineStatus::from_string((*self.status).to_string()),
            amount: self.amount as f64,
            reason: (*self.motive).to_string(),
            // el nombre real se fetchea después en el repo con el helper genérico
            presented_by_name: crate::models::DEFAULT_PRESENTER_NAME.to_string(),
        }
    }
}
