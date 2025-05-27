use crate::config::Env;
use r2d2::Pool;
use redis::Client;

pub fn get_pool_connection() -> Pool<Client> {
    let config: Env = Env::env_init();

    //TODO: Change the url for TLC, for being concat friendly
    let client = Client::open(config.redis_url).expect("Couldn't Connect to redis-db");

    return match Pool::builder().build(client) {
        Ok(val) => val,
        Err(e) => panic!("Couldn't Connect Because: {:}", e),
    };
}
