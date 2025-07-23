use general_api::repos::auth::utils::hashing_composite_key;

/// For trying to hash string of numbers (concatenated) in to sha256
#[test]
fn hashing_numbers() {
    let first_number = "123".to_string();

    assert_eq!(
        "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3".to_uppercase(),
        hashing_composite_key(&[&first_number])
    );

    let second_number = "0".to_string();
    assert_eq!(
        "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9".to_uppercase(),
        hashing_composite_key(&[&second_number])
    );
}

/// For trying to string of numbers in to sha256
#[test]
fn hashing_text() {
    let first_number = "El_Del_Testeo".to_string();

    assert_eq!(
        "3b1e29febd682e079a9c3bd992e18fd4008a6cf89d97f9621e03b169287e0876".to_uppercase(),
        hashing_composite_key(&[&first_number])
    );

    let second_number = "TesteoDelDQ".to_string();
    assert_eq!(
        "72ef4e287a6bfac153913e2c033f2aff35171a34fea47bcdf376d4ac71240dd4".to_uppercase(),
        hashing_composite_key(&[&second_number])
    );
}
