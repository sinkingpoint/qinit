extern crate breuer;

use breuer::{UUID};

#[test]
fn test_uuid_parses() {
    assert!(UUID::try_from_string("aaaa").is_none());
    let uuid = UUID::try_from_string("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
    assert!(uuid.is_some());
    assert_eq!(uuid.unwrap(), UUID{uuid: [0xAA; 16]})
}

#[test]
fn test_uuid_stringifies() {
    let uuid = UUID{uuid: [0xAA; 16]};
    assert_eq!(uuid.to_string(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

    let uuid = UUID{uuid: [0xFF; 16]};
    assert_eq!(uuid.to_string(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
}