extern crate accountant;
extern crate toml;
extern crate serde_derive;

use serde_derive::Deserialize;
use accountant::tasks::serde::Identifier;

#[test]
fn test_identifier_deserializes() {
    #[derive(Deserialize)]
    struct Test {
        id: Identifier
    };

    let test = toml::from_str::<Test>("id = 42");
    assert!(test.is_ok());
    assert_eq!(test.unwrap().id, Identifier::NumericID(42));

    let test = toml::from_str::<Test>("id = \"colin\"");
    assert!(test.is_ok());
    assert_eq!(test.unwrap().id, Identifier::Name("colin".to_owned()));
}