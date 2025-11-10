pub mod utils;

use std::{io::Read, sync::Arc};

use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};

use crate::models::{file::UploadForm, StatusMessage};

//TODO: refactor this for multiple documents

pub async fn upload_ticket_payments(
    form: UploadForm,
    s3_client: Arc<S3Client>,
    bucket_name: Arc<String>,
) -> Result<StatusMessage, StatusMessage> {
    // casting cause slicing needs size at compile time
    let file = form.file.file.into_file();

    let body = ByteStream::read_from()
        .file(file.into())
        .build()
        .await
        .unwrap();

    match s3_client
        .put_object()
        .bucket(bucket_name.as_str())
        .key("payment-tickets/idunno.pdf")
        .body(body)
        .send()
        .await
    {
        Ok(_) => Ok(StatusMessage {
            message: "upload it to bucket".to_owned(),
        }),
        Err(e) => {
            println!("{e:?}");
            Err(StatusMessage {
                message: "couldn't upload file".to_owned(),
            })
        }
    }
}
