use serde::{Deserialize, Serialize};

use crate::{
    models::{
        graphql::{
            Fine as GraphQLFine, Loan as GraphQLLoan, LoanStatus, Payment as GraphQLPayment,
            PaymentStatus,
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
            total_amount: self.total_amount,
            account_num: (*self.account_number).to_string(),
            payment_date: (*self.date_created).to_string(),
            ticket_num: (*self.ticket_number).to_string(),
            commentary: self.comments.clone(), // f*** options, can't do low level stuff some times
            photo: (*self.comprobante_bucket).to_string(),
            state: PaymentStatus::from_string((*self.status).to_string()),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fine {
    pub amount: f32,
    pub motive: String,
}

impl Default for Fine {
    fn default() -> Self {
        Fine {
            amount: 0.,
            motive: "nu uh".to_owned(),
        }
    }
}

impl GraphQLMappable<GraphQLFine> for Fine {
    fn to_graphql_type(&self, key: String) -> GraphQLFine {
        GraphQLFine {
            id: get_key(key, "fines".to_owned()),
            quantity: self.amount as f64,
            reason: (*self.motive).to_string(),
        }
    }
}
