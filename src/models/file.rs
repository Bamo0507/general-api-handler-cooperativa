use std::fs::File;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilePayloadRetrival {
    pub access_token: String,
    pub ticket_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilePayloadUpload {
    pub access_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileUploadInfo {
    pub ticket_path: String,
}
