use actix_cors::Cors;
use actix_web::{App, HttpServer};
use aws_sdk_s3::Client as S3Client;
use general_api::config::Env;
use general_api::endpoints::file_endpoints::{self, file_endpoints};
use general_api::endpoints::{
    auth_endpoints::auth_config, graphql_endpoints::graphql_config, health_config,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno desde .env
    dotenv::dotenv().ok();
    // !For production
    let config = Env::env_init();

    let port = config.port;
    let host = config.host;
    let bucket_name = config.bucket_name;

    println!("bucket name: {bucket_name}");

    // Weird bugg in server
    println!("{}", config.redis_url);

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // aws client pool
    let s3_config = aws_config::load_from_env().await;
    let s3_client = S3Client::new(&s3_config);

    HttpServer::new(move || {
        //TODO: change the cors in production
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        //TODO: add the auth config when capable
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
