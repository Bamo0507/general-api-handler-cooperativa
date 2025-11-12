#[derive(Clone, Deserialize, Debug)]
pub struct ConfigureAllSecurityAnswersRequest {
    pub access_token: String,
    pub answers: [String; 3],
}
#[derive(Deserialize)]
pub struct SecurityQuestionsQuery {
    pub user_name: String,
}
use serde::{Deserialize, Serialize};

// Preguntas de seguridad disponibles (hardcodeadas)
pub const SECURITY_QUESTIONS: [&str; 3] = [
    "¿Cuál fue el nombre de la primera escuela o colegio al que asististe?",
    "¿En qué colonia o barrio viviste durante tu infancia?",
    "¿Cuál era tu materia o clase favorita en la escuela?",
];

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

#[derive(Clone, Serialize, Debug)]
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
            UserType::Directive => "Directive".to_owned(), // funny how bryan it's doing thing in spanish
            UserType::General => "General".to_owned(),
        }
    }
}

// Password Recovery - Request/Response Structs

#[derive(Clone, Serialize)]
pub struct SecurityQuestionsResponse {
    pub questions: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ValidateSecurityAnswerRequest {
    pub user_name: String,
    pub question_index: u8,  // 0, 1, or 2
    pub security_answer: String,
}

#[derive(Clone, Serialize)]
pub struct ValidateSecurityAnswerResponse {
    pub message: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ResetPasswordRequest {
    pub user_name: String,
    pub question_index: u8,  // 0, 1, or 2
    pub security_answer: String,
    pub new_pass_code: String,
}

#[derive(Clone, Serialize)]
pub struct ResetPasswordResponse {
    pub access_token: String,
}
