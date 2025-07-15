use serde::Serialize;

pub mod auth;
pub mod general;
pub mod graphql;
pub mod redis;

//My Own error message
#[derive(Clone, Serialize)]
pub struct ErrorMessage {
    pub message: String,
}
