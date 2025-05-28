use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub struct SignUpInfo {
    pub user_name: String,
    pub pass_code: String, //TODO: Convience bryan to pass this info hashed
    pub real_name: String,
}

#[derive(Clone, Serialize)]
pub struct TokenInfo {
    pub access_token: String,
}
