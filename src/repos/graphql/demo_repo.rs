use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, GraphQLObject, Debug)]
pub struct Demo {
    hello: String,
}

impl Demo {
    pub fn init() -> Demo {
        return Demo {
            hello: "hello".to_string(),
        };
    }
}
