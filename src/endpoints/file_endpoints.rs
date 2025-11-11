use actix_web::web::{get, post, resource, Data, ServiceConfig};
use aws_sdk_s3::Client as S3Client;

use crate::endpoints::handlers::rest::file::{get_ticket_from_payment, upload_ticket_for_payment};

pub fn file_endpoints(config: &mut ServiceConfig, s3_client: S3Client, bucket_name: String) {
    config
        .app_data(Data::new(s3_client))
        .app_data(Data::new(bucket_name))
        .service(
            resource("/general/upload_ticket_payment").route(post().to(upload_ticket_for_payment)),
        )
        .service(resource("/general/get_ticket_payment").route(get().to(get_ticket_from_payment)));
}
