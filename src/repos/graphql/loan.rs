use super::utils::utils::return_n_dummies;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

//Fields are in spanish, for easier parsing in bryan's side
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Loan {
    solicitante_id: i32,
    nombre: String,
    monto_total: f64,
    monto_cancelado: f64,
    motivo: String,
    tasa_interes: f64,
    fecha_solicitud: String, //For parsing purposes
    plazo_meses: i32,
    meses_cancelados: i32,
    //codeudores: Vec<Codeudor>> TODO: add when codeudores
    //mensualidad_prestamo: Vec<PrestamoDetalles> TODO: add PrestamoDetalles
    //pagare: Vec<Pagare> TODO: add Pagares
}

//it is use, but by referencing it, not directly
fn DummyLoan() -> Loan {
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
        //codeudores: Vec<Codeudor>> TODO: add when codeudores
        //mensualidad_prestamo: Vec<PrestamoDetalles> TODO: add PrestamoDetalles
        //pagare: Vec<Pagare> TODO: add Pagares
    };
}

//TODO:Add respective pools
pub struct LoanRepo {}

impl LoanRepo {
    pub fn init() -> LoanRepo {
        return LoanRepo {};
    }

    //TODO: implent true logic
    pub fn get_history(&self) -> Vec<Loan> {
        return return_n_dummies::<Loan>(&DummyLoan, 10);
    }
}
