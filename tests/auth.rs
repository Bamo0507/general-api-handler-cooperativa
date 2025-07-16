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
fn from_access_tokens_to_credentials() {}

/// For checking if credentials cretead an user instance in Redis DB (Signup)
#[test]
fn from_credentials_to_data() {}

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
