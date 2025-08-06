// This tests assume that the redis db is running and the env variables are declare on the cli
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

use general_api::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    repos::{
        auth::{
            create_user_with_access_token, get_user_access_token, utils::hashing_composite_key,
        },
        graphql::payment::PaymentRepo,
    },
};
use redis::Commands;

/// Tests for checking login function integrity
/// For for getting acess_token with the given credentials in Redis DB (Login)
#[test]
fn from_credentials_to_acess_token() {
    let acess_token = get_user_access_token(
        "El_Mago_Pero_Del_Test".to_string(),
        "ElTestoPaga".to_string(),
    );

    assert_eq!(
        acess_token.unwrap().access_token.to_uppercase(),
        "c50329f3e834e2d6a27d0e1a81fc12579aa4570fa889eb302ca192f82961edb0"
            .to_string()
            .to_uppercase()
    );
}

/// For checking if credentials cretead an user instance in Redis DB (Signup)
#[test]
fn from_credentials_to_data() {
    let repo = PaymentRepo::init(get_pool_connection());

    // random string
    let mut random_string = Alphanumeric.sample_string(&mut rng(), 16);

    let mut access_token = String::new();

    // Loop for getting new access token
    loop {
        access_token = match create_user_with_access_token(
            random_string.clone(),
            random_string.clone(),
            random_string.clone(),
        ) {
            Ok(token_info) => {
                let mut con = get_pool_connection().into_inner().get().unwrap();

                let db_acess_token = hashing_composite_key(&[&token_info.access_token]);
                con.set::<String, f32, f32>(
                    format!("users:{}:owed_capital", db_acess_token),
                    10101.0,
                );
                con.set::<String, f32, f32>(
                    format!("users:{}:payed_to_capital", db_acess_token),
                    1010.0,
                );

                token_info.access_token
            }
            Err(_) => {
                // for retrying a new string
                random_string = Alphanumeric.sample_string(&mut rng(), 13);
                "Error Token".to_string()
            }
        };

        // Just breaks the loop until it could create a signup
        if access_token != "Error Token".to_string() {
            break;
        };
    }

    // Pretty similar as the Login test
    assert_eq!(
        10101.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .owed_capital
    );

    assert_eq!(
        1010.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .payed_to_capital
    );
}

//*! 2 Tests go here
/// For getting data from User in Redis DB (Login)
#[test]
fn check_if_can_acess_data() {
    let username = "El_Mago_Pero_Del_Test".to_string();
    let passcode = "ElTestoPaga".to_string();

    let repo = PaymentRepo::init(get_pool_connection());

    // Creates user in DB if not already created (assuming that signup works)
    let access_token = match get_user_access_token(username.clone(), passcode.clone()) {
        Ok(val) => val.access_token,
        Err(_) => {
            // Omg, how nested is this
            let mut con = get_pool_connection().into_inner().get().unwrap();

            // this assume the signup test already passed on
            let token = create_user_with_access_token(
                username.clone(),
                passcode.clone(),
                "EL Pedro Del Testo".to_string(),
            )
            .unwrap()
            .access_token;

            let db_acess_token = hashing_composite_key(&[&token]);
            con.set::<String, f32, f32>(format!("users:{}:owed_capital", db_acess_token), 10101.0);
            con.set::<String, f32, f32>(
                format!("users:{}:payed_to_capital", db_acess_token),
                1010.0,
            );

            token
        }
    };

    assert_eq!(
        10101.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .owed_capital
    );

    assert_eq!(
        1010.0,
        repo.get_user_history(access_token.clone())
            .unwrap()
            .payed_to_capital
    );
}
