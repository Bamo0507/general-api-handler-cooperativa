use serde::{Deserialize, Serialize};

use super::graphql::{Aporte, Cuota, PrestamoDetalles};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Payment {
    pub usuario_id: i32,
    pub monto_total: f64,
    pub fecha_pago: String, // I'll pass it as a string, for not having parsing difficulties
    pub num_boleta: String,
    pub banco_deposito: String,
    pub cuotas: Vec<Cuota>,
    pub prestamos: Vec<PrestamoDetalles>,
    pub aporte_capital: Vec<Aporte>,
    pub comentarios: String,
    pub foto: String,   // For bucket use
    pub estado: String, // Following bryan's enums
}
