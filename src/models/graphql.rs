use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

//Fields are in spanish, for easier parsing in bryan's side
#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Loan {
    pub solicitante_id: i32,
    pub nombre: String,
    pub monto_total: f64,
    pub monto_cancelado: f64,
    pub motivo: String,
    pub tasa_interes: f64,
    pub fecha_solicitud: String, //For parsing purposes
    pub plazo_meses: i32,
    pub meses_cancelados: i32,
    // pub codeudores: Vec<Codeudor>> TODO: add when codeudores
    // pub mensualidad_prestamo: Vec<PrestamoDetalles> TODO: add PrestamoDetalles
    // pub pagare: Vec<Pagare> TODO: add Pagares
}

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Payment {
    pub usuario_id: i32,
    pub monto_total: f64,
    pub fecha_pago: String, //I'll pass it as a string, for not having parsing difficulties
    pub num_boleta: String,
    pub banco_deposito: String,
    // pub cuotas: Vector<Cuotas> TODO: make later
    // pub prestamos: Vector<PrestamoDetalle> TODO: make later
    // pub aporte_capital: Vector<Aporte> TODO: make it later
    pub comentarios: String,
    pub foto: String,   //For bucket use
    pub estado: String, //Following bryan's enums
}
