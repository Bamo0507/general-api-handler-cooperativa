use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
    error::ErrorBadRequest,
    mime::Mime,
    web::{Data, Query},
    HttpResponse, Responder,
};
use aws_sdk_s3::Client as S3Client;

use crate::{
    models::file::{FileCredentials, UploadForm},
    repos::file::{get_ticket_payment, upload_ticket_payment},
};

pub async fn upload_ticket_for_payment(
    MultipartForm(form): MultipartForm<UploadForm>,
    file_upload_credentials: Query<FileCredentials>,
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

pub async fn get_ticket_from_payment(
    file_getter_credentials: Query<FileCredentials>,
    ticket_path: String,
    s3_client: Data<S3Client>,
    bucket_name: Data<String>,
) -> HttpResponse {
    let file_getter_credentials = file_getter_credentials.into_inner();

    let ticket = get_ticket_payment(
        file_getter_credentials.access_token,
        ticket_path,
        s3_client.into_inner(),
        bucket_name.into_inner(),
    )
    .await;

    todo!()
}
