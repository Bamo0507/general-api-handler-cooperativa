use std::fmt::format;

use r2d2::PooledConnection;
use redis::{cmd, Client, Commands, RedisError};
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

    let access_token = hashing_composite_key(username.clone(), pass);

    //For checking the existance of the field
    match cmd("GET")
        .arg(format!("users_on_used:{}", username.clone()))
        .query::<String>(&mut con)
    {
        //This will look weird, but we are looking here in the case it fails
        Err(_) => {
            //For creating fields
            // IK is pretty repetitive, but is the best way for being the most explicit neccesary

            let _: () = con
                .set(
                    format!("users_on_used:{}", username.clone()),
                    username.clone(),
                )
                .expect("USERNAME CREATION : Couldn't filled username");

            let _: () = con
                .set(
                    format!("users:{}:complete_name", real_name.clone()),
                    username.clone(),
                )
                .expect("ACCESS TOKEN CREATION: Couldn't filled username");

            // For default any new user won't be
            let _: () = con
                .set(
                    format!("users:{}:is_directive", access_token.clone()),
                    false,
                )
                .expect("ACCESS TOKEN CREATION: couldn't filled is_directive");

            let _: () = con
                .set(format!("users:{}:payments", access_token.clone()), false)
                .expect("BASE PAYMENTS CREATION: Couldn't create field");

            let _: () = con
                .set(format!("users:{}:loans", access_token.clone()), false)
                .expect("BASE LOANS CREATION: Couldn't create field");

            return Ok(TokenInfo { access_token });
        }

        Ok(_) => Err(ErrorMessage {
            message: "User Already Exists".to_string(),
        }),
    }
}
