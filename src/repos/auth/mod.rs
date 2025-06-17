use std::fmt::format;

use r2d2::PooledConnection;
use redis::{cmd, from_redis_value, Client, Commands, Value as RedisValue};
use utils::hashing_composite_key;

use crate::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    models::{auth::TokenInfo, ErrorMessage},
};

pub(super) mod utils;

pub struct AuthRepo {
    pub con: PooledConnection<Client>,
}

//TODO: Set for ALC
pub fn create_user_with_access_token(
    username: String,
    pass: String,
    real_name: String,
) -> Result<TokenInfo, ErrorMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool"); //Can't abstracted to a struct, :C

    let access_token = hashing_composite_key(&username, &pass);

    let db_composite_key = hashing_composite_key(&access_token, &"".to_string());

    //For checking the existance of the field
    match cmd("GET")
        .arg(format!("users_on_used:{}", username.clone()))
        .query::<String>(&mut con)
    {
        //This will look weird, but we are looking here in the case it fails
        Err(_) => {
            //For creating fields
            // IK is pretty repetitive, but is the best way for being the most explicit neccesary

            //Want to have the resource the closest to key level, cause is just for checking if it exists
            let _: () = con
                .set(format!("users_on_used:{}", &username), "")
                .expect("USERNAME CREATION : Couldn't filled username");

            let _: () = con
                .set(
                    format!("users:{}:complete_name", &db_composite_key),
                    real_name.clone(),
                )
                .expect("ACCESS TOKEN CREATION: Couldn't filled username");

            // For default any new user won't be
            let _: () = con
                .set(format!("users:{}:is_directive", &db_composite_key), false)
                .expect("ACCESS TOKEN CREATION: couldn't filled is_directive");

            let _: () = con
                .set(format!("users:{}:payments", &db_composite_key), false)
                .expect("BASE PAYMENTS CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:loans", &db_composite_key), false)
                .expect("BASE LOANS CREATION: Couldn't create field");

            return Ok(TokenInfo { access_token });
        }

        Ok(_) => Err(ErrorMessage {
            message: "Couldn't Create User".to_string(),
        }),
    }
}

//TODO: Refactor this for recieving the access token
pub fn get_user_access_token(username: String, pass: String) -> Result<TokenInfo, ErrorMessage> {
    let mut con = get_pool_connection()
        .get()
        .expect("Couldn't connect to pool"); //Can't abstracted to a struct, :C

    let access_token = hashing_composite_key(&username, &pass);

    //Passing an String for recieving an nil
    match cmd("EXISTS")
        .arg(format!("users:{}:complete_name", access_token)) //Closests key-value we have at hand
        .query::<bool>(&mut con)
    {
        Ok(it_exists) => {
            if it_exists {
                return Ok(TokenInfo { access_token });
            }

            return Err(ErrorMessage {
                message: "User Might Not Exist or User/Password is wrong".to_string(),
            });
        }
        Err(e) => {
            return Err(ErrorMessage {
                message: "Couldn't Get User Info:".to_string(),
            });
        }
    };
}
