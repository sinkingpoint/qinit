extern crate accountant;
extern crate toml;
extern crate serde_derive;

use serde_derive::Deserialize;
use accountant::tasks::serde::{TaskDef, Identifier, DependencyDef};
use std::path::Path;

#[test]
fn test_dependency_def_eq() {
    assert_eq!(DependencyDef::new("test".to_owned(), None), DependencyDef::new("TEST".to_owned(), None));
    assert_ne!(DependencyDef::new("test".to_owned(), None), DependencyDef::new("TES".to_owned(), None));
    assert_eq!(DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg".to_owned())].into_iter().collect())), DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg".to_owned())].into_iter().collect())));
    assert_ne!(DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg".to_owned())].into_iter().collect())), DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg1".to_owned())].into_iter().collect())));
    assert_ne!(DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg".to_owned())].into_iter().collect())), DependencyDef::new("test".to_owned(), None));
    assert_ne!(DependencyDef::new("test".to_owned(), Some(vec![("arg".to_owned(), "arg".to_owned())].into_iter().collect())), DependencyDef::new("test".to_owned(), Some(vec![("gra".to_owned(), "arg".to_owned())].into_iter().collect())));
}

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

#[test]
fn test_task_deserializes() {
    let test = toml::from_str::<TaskDef>("name = \"TestTask\"
    description = \"A Test Service\"
    user = 1000
    init_command = \"echo 'cats'\"
    start_command = \"echo 'more cats'\"
    args = [
        \"test_arg\"
    ]
    
    [[ requires ]]
    name = \"test_task\"
    [ requires.args ]
    test_arg = \"test\"

    [[ requires ]]
    name = \"test_task2\"
    [ requires.args ]
    test_arg2 = \"test2\"
    test_arg3 = \"test3\"

    [conditions]
    [[conditions.unix_sockets]]
    path = \"/var/run/socket\"
    ");
    assert!(test.is_ok(), test.unwrap_err().to_string());
    let test = test.unwrap();

    assert_eq!(test.name, "TestTask");
    assert_eq!(test.description, "A Test Service");
    assert_eq!(test.user, Some(Identifier::NumericID(1000)));
    assert_eq!(test.init_command, Some("echo 'cats'".to_owned()));
    assert_eq!(test.start_command, "echo 'more cats'");
    assert_eq!(test.args, Some(vec!["test_arg".to_owned()]));
    assert!(test.requires.is_some());
    assert_eq!(test.requires.as_ref().unwrap()[0].name, "test_task");
    assert_eq!(test.requires.as_ref().unwrap()[0].args.as_ref().unwrap().len(), 1);
    assert_eq!(test.requires.as_ref().unwrap()[0].args.as_ref().unwrap().get("test_arg").unwrap(), "test");

    assert_eq!(test.requires.as_ref().unwrap()[1].name, "test_task2");
    assert_eq!(test.requires.as_ref().unwrap()[1].args.as_ref().unwrap().len(), 2);
    assert_eq!(test.requires.as_ref().unwrap()[1].args.as_ref().unwrap().get("test_arg2").unwrap(), "test2");
    assert_eq!(test.requires.as_ref().unwrap()[1].args.as_ref().unwrap().get("test_arg3").unwrap(), "test3");
    
    assert_eq!(test.conditions.as_ref().unwrap().unix_sockets.as_ref().unwrap().len(), 1);
    assert_eq!(test.conditions.as_ref().unwrap().unix_sockets.as_ref().unwrap()[0].path, Path::new("/var/run/socket"));
}