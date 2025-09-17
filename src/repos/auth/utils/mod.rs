use sha2::{Digest, Sha256};

/// function that giving n reference to arguments, returns the hasked key in string format
pub fn hashing_composite_key(args: &[&String]) -> String {
    //Passes the args formated

    let mut string_acc = String::new();

    for arg in args {
        string_acc = format!("{}{}", &string_acc, arg);
    }

    let hashed_args = Sha256::digest(string_acc);

    //X is for hexadecimal
    format!("{:X}", hashed_args)
}

// TODO: do function for matching payment end value with Payment Object
