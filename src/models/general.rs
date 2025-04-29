use serde::{Deserialize, Serialize};

//TODO: Add more fields when added new schemas
#[derive(Clone, Serialize, Deserialize)]
pub struct GeneralInfo {
    pub api_version: String,
}
