use regex::Regex;


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
pub fn get_payment_key(raw_payment_key: String) -> String {
    let re = Regex::new(r"users:[\w]+:payments:(?<payment_key>\w+)").unwrap();

    // let's assume this is correct, cause the only value that will enter here will be payment_keys
    let split_key = re.captures(&raw_payment_key).unwrap();

    split_key["payment_key"].to_string()
}
