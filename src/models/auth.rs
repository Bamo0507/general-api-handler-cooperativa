use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub struct SignUpInfo {
    pub user_name: String,
    pub pass_code: String, //TODO: Convience bryan to pass this info hashed
    pub real_name: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct LoginInfo {
    pub access_token: String,
}

#[derive(Clone, Serialize)]
pub struct TokenInfo {
    pub access_token: String,
}
