pub mod utils;

use std::{io::Read, sync::Arc};

use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};

use crate::{
    models::{
        file::{FileUploadInfo, UploadForm},
        StatusMessage,
    },
    repos::{auth::utils::hashing_composite_key, file::utils::check_file_upload_credentials},
};

//TODO: refactor this for multiple documents

pub async fn upload_ticket_payments(
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
        .key(format!("payment-tickets/{file_name}.pdf"))
        .send()
        .await
    {
        Ok(_) => Ok(FileUploadInfo {
            file_path: format!("payment-tickets/{file_name}.pdf"),
        }),
        Err(e) => {
            println!("{e:?}");
            Err(StatusMessage {
                message: "couldn't upload file".to_owned(),
            })
        }
    }
}
