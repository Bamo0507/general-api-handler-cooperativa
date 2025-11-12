use actix_cors::Cors;
use actix_web::{App, HttpServer};
use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_s3::Client as S3Client;
use aws_smithy_http_client::{Builder, tls};
use general_api::config::Env;
use general_api::endpoints::file_endpoints::{self, file_endpoints};
use general_api::endpoints::{
    auth_endpoints::auth_config, graphql_endpoints::graphql_config, health_config,
};
use std::fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno desde .env
    dotenv::dotenv().ok();
    // !For production
    let config = Env::env_init();

    let port = config.port;
    let host = config.host;
    let bucket_name = config.bucket_name;

    env_logger::init();

    let s3_config: SdkConfig = if (config.tls_on == 0) {
        let pem_contents = fs::read("cld.crt").expect("could not read CA pem");
        let trust_store = tls::TrustStore::empty().with_pem_certificate(&*pem_contents);

        let tls_context = tls::TlsContext::builder()
            .with_trust_store(trust_store)
            .build()
            .expect("valid TLS context");

        let http_client = Builder::new()
            .tls_provider(tls::Provider::Rustls(
                tls::rustls_provider::CryptoMode::AwsLc,
            ))
            .tls_context(tls_context)
            .build_https();

        // Build AWS config from env vars + custom HTTP client
        aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.aws_region))
            .http_client(http_client)
            .load()
            .await
    } else {
        aws_config::load_from_env().await
    };

    let s3_client = S3Client::new(&s3_config);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .configure(graphql_config)
            .configure(|config| {
                file_endpoints(config, s3_client.to_owned(), bucket_name.to_owned())
            })
            .configure(health_config)
            .configure(auth_config)
            .wrap(cors)
    })
    .bind((host, port))?
    .run()
    .await
}
