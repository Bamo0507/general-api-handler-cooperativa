use hex_literal::hex;

use sha2::{Digest, Sha256};

pub fn hashing_composite_key(first_arg: String, second_arg: String) -> String {
    //Passes the args formated
    let hashed_args = Sha256::digest(format!("{}:{}", first_arg, second_arg));

    //X is for hexadecimal
    return format!("{:X}", hashed_args);
}
