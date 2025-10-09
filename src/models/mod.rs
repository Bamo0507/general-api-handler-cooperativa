use serde::{Deserialize, Serialize};

pub mod auth;
pub mod graphql;
pub mod redis;

//my Own error message
#[derive(Debug, Clone, Serialize)]
pub struct StatusMessage {
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralInfo {
    pub api_version: String,
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
