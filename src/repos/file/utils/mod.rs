use redis::cmd;

use crate::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    repos::auth::utils::hashing_composite_key,
};

pub fn check_file_upload_credentials(access_token: &String) -> bool {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool"); //Can't abstracted to a struct, :C

    // How is registered on the db
    let db_access_token = hashing_composite_key(&[&access_token]);

    match cmd("EXISTS")
        .arg(format!("users:{db_access_token}:complete_name")) //Closests key-value we have at hand
        .query::<bool>(&mut con)
    {
        Ok(it_exists) => it_exists,
        Err(_) => return false, // let's just say if it can't be found, it doesn't exists
    }
}
