use actix_cors::Cors;
use actix_web::{App, HttpServer};
use general_api::config::Env;
use general_api::endpoints::{
    auth_endpoints::auth_config, graphql_endpoints::graphql_config, health_config,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // !For production
    let config = Env::env_init();

    let port = config.port;
    let host = config.host;

    // Weird bugg in server
    println!("{}", config.redis_url);

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

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
            .configure(health_config)
            .configure(auth_config)
            .wrap(cors)
    })
    .bind((host, port))?
    .run()
    .await
}
