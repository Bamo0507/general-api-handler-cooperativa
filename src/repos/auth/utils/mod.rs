use sha2::{Digest, Sha256};

pub fn hashing_composite_key(first_arg: String, second_arg: String) -> String {
    let mut hasher = Sha256::new();

    hasher.update(format!("{}:{}", first_arg, second_arg));

    return format!("{:?}", hasher.finalize());
}
