use serde::Serialize;

pub(crate) mod auth;
pub(crate) mod general;
pub(crate) mod graphql;

//My Own error message
#[derive(Clone, Serialize)]
pub struct ErrorMessage {
    pub message: String,
}
