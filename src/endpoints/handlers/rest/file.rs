use actix_multipart::form::MultipartForm;
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use aws_sdk_s3::Client as S3Client;

use crate::{
    models::file::{FileUploadCredentials, UploadForm},
    repos::file::upload_ticket_payment,
};

pub async fn upload_ticket_for_payment(
    MultipartForm(form): MultipartForm<UploadForm>,
    file_upload_credentials: Query<FileUploadCredentials>,
    s3_client: Data<S3Client>,
    bucket_name: Data<String>,
) -> HttpResponse {
    // safe check for just letting users upload payments

    let file_upload_credentials = file_upload_credentials.into_inner();

    HttpResponse::Ok().json(
        upload_ticket_payment(
            form,
            file_upload_credentials.access_token.clone(),
            s3_client.into_inner(),
            bucket_name.into_inner(),
        )
        .await,
    )
}
