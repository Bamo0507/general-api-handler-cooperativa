use redis::Commands;
use utils::hashing_composite_key;

use crate::endpoints::handlers::configs::connection_pool::get_pool_connection;

mod utils;

//TODO: Set for ALC
pub fn create_access_token(username: String, pass: String) -> String {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool");

    let hashed_key = hashing_composite_key(username, pass);

    //TODO: Add fields creations
    return hashed_key;
}
