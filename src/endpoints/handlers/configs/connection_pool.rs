use std::time::Duration;

use crate::config::Env;
use actix_web::web;
use r2d2::Pool;
use redis::Client;

pub fn get_pool_connection() -> web::Data<Pool<Client>> {
    let config: Env = Env::env_init();

    //TODO: Change the url for being concat friendly
    let client = Client::open(config.redis_url).expect("Couldn't stablished client");

    match Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(client)
    {
        Ok(val) => {
            return web::Data::<Pool<Client>>::new(val);
        }
        Err(e) => {
            println!("Couldn't Connect Because: {:}", e);
            panic!("Couldn't Connect Because: {:}", e);
        }
    };
}
