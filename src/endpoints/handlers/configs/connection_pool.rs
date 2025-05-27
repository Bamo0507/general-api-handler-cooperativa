use crate::config::Env;
use actix_web::{cookie::time, web};
use r2d2::Pool;
use redis::Client;

pub fn get_pool_connection() -> web::Data<Pool<Client>> {
    let config: Env = Env::env_init();

    //TODO: Change the url for being concat friendly
    let client = Client::open(config.redis_url).expect("Couldn't Connect to redis-db");

    //TODO: fix Pool
    match Pool::builder().build(client) {
        Ok(val) => {
            println!("{:?}", val.connection_timeout());
            return web::Data::<Pool<Client>>::new(val);
        }
        Err(e) => {
            println!("Couldn't Connect Because: {:}", e);
            panic!("Couldn't Connect Because: {:}", e);
        }
    };
}
