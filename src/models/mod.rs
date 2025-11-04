use juniper::{GraphQLInputObject, GraphQLObject};
use serde::{Deserialize, Serialize};

pub mod auth;
pub mod graphql;
pub mod redis;

// valor por default cuando no se puede fetchear el nombre del usuario que presentó algo
// (multa, pago, etc). se usa en vez de hardcodear "N/A" por todo el código
pub const DEFAULT_PRESENTER_NAME: &str = "N/A";

//my Own error message
#[derive(Debug, Clone, Serialize)]
pub struct StatusMessage {
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralInfo {
    pub api_version: String,
}

// PayedTo: necesitamos dos versiones separadas debido a restricciones de Juniper
// - PayedTo (GraphQLObject): para queries/output - retornar datos al frontend
// - PayedToInput (GraphQLInputObject): para mutations/input - recibir datos del frontend
// Redis usa PayedTo directamente. La conversión PayedToInput -> PayedTo es automática via From trait
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[derive(PartialEq)]
pub struct PayedTo {
    pub model_type: String,
    pub amount: f64,
    pub model_key: String,
}

// versión input para usar en mutations (create_user_payment, etc)
#[derive(Debug, Clone, Serialize, Deserialize, GraphQLInputObject)]
pub struct PayedToInput {
    pub model_type: String,
    pub amount: f64,
    pub model_key: String,
}

impl Default for PayedToInput {
    fn default() -> Self {
        PayedToInput {
            model_type: "LOAN".to_owned(),
            amount: 0.00,
            model_key: "000000000000".to_owned(),
        }
    }
}

// conversión fácil de input a output
impl From<PayedToInput> for PayedTo {
    fn from(input: PayedToInput) -> Self {
        PayedTo {
            model_type: input.model_type,
            amount: input.amount,
            model_key: input.model_key,
        }
    }
}

impl Default for PayedTo {
    fn default() -> Self {
        PayedTo {
            model_type: "LOAN".to_owned(),
            amount: 0.00,
            model_key: "000000000000".to_owned(),
        }
    }
}

/// trait for mapping redis values to graphql ones
pub trait GraphQLMappable<GraphQLType> {
    /// method for mapping any object with this trait in to a graphQLModel.
    fn to_graphql_type(&self, key: String) -> GraphQLType; // adding the key
                                                           // argument for not doing multiple traits jus for those which don't hae
}

/// trait for enum mapping
pub trait FromString {
    fn from_string(raw_status: String) -> Self;
}

/// trait para modelos que tienen el campo presented_by_name
/// permite que el helper genérico pueda asignar el nombre del presentador
/// a cualquier modelo sin importar su tipo específico (Payment, Fine, Loan, etc)
pub trait WithPresenterName {
    fn set_presenter_name(&mut self, name: String);
}
