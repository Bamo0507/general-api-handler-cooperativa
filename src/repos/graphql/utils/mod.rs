use regex::Regex;

use crate::repos::auth::utils::hashing_composite_key;

///Function for returning n number of any type value, having a function as a generator
//(Ik syntaxis looks scary in the parameters, but it ain't)
pub fn return_n_dummies<Value>(dummy_generator: &dyn Fn() -> Value, n: i32) -> Vec<Value> {
    let mut dummy_list: Vec<Value> = vec![];

    //pretty simple logic
    for _ in 0..n {
        dummy_list.push(dummy_generator());
    }

    dummy_list
}

/// Function that returns only the relative payment key
/// where the raw_key is the string of the value, and the key_type is the type of the redis
/// key
pub fn get_key(raw_key: String, key_type: String) -> String {
    // we format for injecting the key_type
    let re = Regex::new(format!(r"users:[\w]+:{}:(?<key>\w+)", key_type).as_str()).unwrap();

    // let's assume this is correct, cause the only value that will enter here will be payment_keys
    let split_key = re.captures(&raw_key).unwrap();

    split_key["key"].to_string()
}
