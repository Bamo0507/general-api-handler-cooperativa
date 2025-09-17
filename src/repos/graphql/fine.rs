use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

pub struct FineRepo {
    pub pool: Data<Pool<Client>>,
}

impl FineRepo {}
