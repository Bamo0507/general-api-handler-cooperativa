pub mod utils;

use std::{fs::File, io::Write, sync::Arc};

use actix_files::NamedFile;
use actix_web::{
    HttpResponse,
    http::header::{ContentDisposition, DispositionType},
    mime::Mime,
};
use aws_sdk_s3::{Client as S3Client, primitives::ByteStream};
use tokio::fs::read;

use crate::{
    models::{
        StatusMessage,
        file::{FileUploadInfo, UploadForm},
    },
    repos::{auth::utils::hashing_composite_key, file::utils::check_file_upload_credentials},
};

//TODO: refactor this for multiple documents

pub async fn upload_ticket_payment(
    form: UploadForm,
    access_token: String,
    s3_client: Arc<S3Client>,
    bucket_name: Arc<String>,
) -> Result<FileUploadInfo, StatusMessage> {
    // in case it doesn't have good credentials, a bit of deffensive programming
    if !check_file_upload_credentials(&access_token) {
        return Err(StatusMessage {
            message: "Couldn't verify user".to_owned(),
        });
    }

    // check how many files there are

    let files_len = s3_client
        .list_objects()
        .bucket(bucket_name.as_str())
        .send()
        .await
        .unwrap()
        .contents
        .unwrap()
        .len();

    let file_name = hashing_composite_key(&[&(files_len.to_string()), &access_token.clone()]);

    // casting in to std file type
    let file = form.file.file.into_file();

    let body = ByteStream::read_from()
        .file(file.into()) // here the casting to the tokio one
        .build()
        .await
        .unwrap();

    match s3_client
        .put_object()
        .bucket(bucket_name.as_str())
        .set_body(Some(body))
        .key(format!("payment-tickets/{file_name}.jpeg"))
        .send()
        .await
    {
        Ok(_) => Ok(FileUploadInfo {
            ticket_id: file_name,
        }),
        Err(e) => {
            println!("{e:?}");
            Err(StatusMessage {
                message: "couldn't upload file".to_owned(),
            })
        }
    }
}

pub async fn get_ticket_payment(
    access_token: String,
    ticket_name: String,
    s3_client: Arc<S3Client>,
    bucket_name: Arc<String>,
) -> Result<HttpResponse, StatusMessage> {
    // in case it doesn't have good credentials, a bit of deffensive programming
    if !check_file_upload_credentials(&access_token) {
        return Err(StatusMessage {
            message: "Couldn't verify user".to_owned(),
        });
    }

    // we need to write it somewhere, then when can delete it
    let mut tmp_file = File::create(&ticket_name).map_err(|err| {
        println!("{err:?}");
        StatusMessage {
            message: "couldn't create temp file".to_owned(),
        }
    })?;

    // byte stream from s3 bucket
    let mut raw_file = match s3_client
        .get_object()
        .bucket(bucket_name.as_str())
        .key(format!("payment-tickets/{ticket_name}.jpeg"))
        .send()
        .await
    {
        Ok(file) => file,
        Err(_) => {
            return Err(StatusMessage {
                message: "couldn't get file".to_owned(),
            });
        }
    };

    // we write bytes to the tmp_file (if they exist)
    while let Some(bytes) = raw_file.body.try_next().await.map_err(|_| StatusMessage {
        message: "couldn't open raw file".to_owned(),
    })? {
        tmp_file.write_all(&bytes).map_err(|_| StatusMessage {
            message: "couldn't write to temp file".to_owned(),
        })?;
    }

    match read(ticket_name).await {
        Ok(image) => Ok(HttpResponse::Ok().content_type("image/jpeg").body(image)),
        Err(_) => Err(StatusMessage {
            message: "Couldn't send Named file".to_owned(),
        }),
    }
}
