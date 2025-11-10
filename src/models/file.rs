use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use serde::{Deserialize, Serialize};

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100MB")]
    pub file: TempFile,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileUploadCredentials {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct FileUploadInfo {
    pub file_path: String,
}
