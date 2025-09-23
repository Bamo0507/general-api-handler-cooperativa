use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;

use crate::{
    models::{graphql::Fine, redis::Fine as RedisFine},
    repos::graphql::utils::get_multiple_models,
};

pub struct FineRepo {
    pub pool: Data<Pool<Client>>,
}

impl FineRepo {
    pub fn get_user_fines(&self, access_token: String) -> Result<Vec<Fine>, String> {
        get_multiple_models::<Fine, RedisFine>(
            access_token,
            self.pool.clone(),
            "fines".to_owned(), // TODO: see a way to don't burn the keys
        )
    }
}
