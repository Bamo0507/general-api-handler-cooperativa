use actix_multipart::form::MultipartForm;
use actix_web::{web::Data, HttpResponse};
use aws_sdk_s3::Client as S3Client;

use crate::{models::file::UploadForm, repos::file::upload_ticket_payments};

pub async fn upload_ticket_for_payment(
    MultipartForm(form): MultipartForm<UploadForm>,
    s3_client: Data<S3Client>,
    bucket_name: Data<String>,
) -> HttpResponse {
    HttpResponse::Ok()
        .json(upload_ticket_payments(form, s3_client.into_inner(), bucket_name.into_inner()).await)
}
