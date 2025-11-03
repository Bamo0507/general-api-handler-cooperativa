pub mod utils;

use std::sync::Arc;

use actix_multipart::form::MultipartForm;
use aws_sdk_s3::Client as S3Client;

use crate::models::{file::UploadForm, StatusMessage};

//TODO: refactor this for multiple documents

pub fn upload_ticket_payments(
    form: UploadForm,
    s3_client: Arc<&S3Client>,
    bucket_name: Arc<String>,
) -> Result<StatusMessage, StatusMessage> {
    //s3_client
    //    .put_object()
    //    .bucket(bucket_name.as_str())
    //    .key("payment-tickets/")
    todo!()
}
