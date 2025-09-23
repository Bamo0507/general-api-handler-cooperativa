use serde::Serialize;

pub mod auth;
pub mod general;
pub mod graphql;
pub mod redis;

//my Own error message
#[derive(Debug, Clone, Serialize)]
pub struct ErrorMessage {
    pub message: String,
}

/// Trait for graphql type which is friendly to be mapped
pub trait GraphQLMapFriendly {}

/// trait for mapping redis values to graphql ones
pub trait GraphQLMappable {
    /// method for mapping any object with this trait in to a graphQLModel.
    fn to_graphql_type(&self, key: String) -> impl GraphQLMapFriendly; // adding the key
                                                                       // argument for not doing multiple traits jus for those which don't hae
}
