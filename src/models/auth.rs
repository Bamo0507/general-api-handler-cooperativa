use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub struct SignUpInfo {
    pub user_name: String,
    pub pass_code: String, //TODO: Convience bryan to pass this info hashed
    pub real_name: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct LoginInfo {
    pub user_name: String,
    pub pass_code: String, //TODO: Convience bryan to pass this info hashed
}

#[derive(Clone, Serialize)]
pub struct TokenInfo {
    pub user_name: String,
    pub access_token: String,
    pub user_type: String,
}

#[derive(Clone)]
pub enum UserType {
    Directive,
    General,
}

// Ignore the error, I'm just doing this for parsing things for bryan
impl ToString for UserType {
    fn to_string(&self) -> String {
        match self {
            UserType::Directive => "Directive".to_string(), // funny how bryan it's doing thing in spanish
            UserType::General => "General".to_string(),
        }
    }
}
