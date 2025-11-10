use actix_web::web::{post, resource, Data, ServiceConfig};
use aws_sdk_s3::Client as S3Client;

use crate::endpoints::handlers::rest::file::upload_ticket_for_payment;

pub fn file_endpoints(config: &mut ServiceConfig, s3_client: S3Client, bucket_name: String) {
    config
        .app_data(Data::new(s3_client))
        .app_data(Data::new(bucket_name))
        .service(
            resource("/general/upload_ticket_payment").route(post().to(upload_ticket_for_payment)),
        );
}
