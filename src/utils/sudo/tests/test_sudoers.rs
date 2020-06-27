extern crate libsudo;

use libsudo::sudoers::{Permission, Identity};

#[test]
fn test_sudoers_parses_file_correctly() {
    let mut config = libsudo::sudoers::Sudoers::new();
    assert!(config.process_line("Defaults !visiblepw").is_ok());
    assert!(config.process_line("Defaults    env_reset").is_ok());
    assert!(config.process_line("Defaults    env_keep =  \"COLORS DISPLAY HOSTNAME HISTSIZE KDEDIR LS_COLORS\"").is_ok());
    assert!(config.process_line("Defaults    env_keep += \"MAIL QTDIR USERNAME LANG LC_ADDRESS LC_CTYPE\"").is_ok());
    assert!(config.process_line("%ops     ALL = (operator : operator) /bin/ls, /bin/cats, /vcats, (root) /bin/kill, /usr/bin/lprm").is_ok());
}

#[test]
fn test_sudoers_parses_identity_correctly() {
    assert!(Identity::from_str("#colin").is_none());
    assert_eq!(Identity::from_str("#123"), Some(Identity::Uid(123)));
    assert_eq!(Identity::from_str("%#123"), Some(Identity::Gid(123)));
    assert_eq!(Identity::from_str("colin"), Some(Identity::User("colin".to_owned())));
    assert_eq!(Identity::from_str("%colin"), Some(Identity::Group("colin".to_owned())));
    assert_ne!(Identity::from_str("#1"), Some(Identity::Uid(1000)));
}

#[test]
fn test_sudoers_parses_permission_correctly() {
    let permission = Permission::from_str("%#123     ALL=(ALL) /bin/ls, /bin/cats, /vcats, (#1) /bin/kill, /usr/bin/lprm");
    
    assert!(permission.is_some());
    let permission = permission.unwrap();
    println!("{:?}", permission);
    assert!(permission.is_allowed(&vec!["/bin/ls"], &Identity::Gid(123), &Identity::Uid(1000), &Identity::Gid(1000)));
    assert!(permission.is_allowed(&vec!["/bin/kill"], &Identity::Gid(123), &Identity::Uid(1), &Identity::Gid(1000)));
    assert!(permission.is_allowed(&vec!["/bin/kill"], &Identity::Gid(123), &Identity::Uid(1), &Identity::Gid(1)));
    assert!(!permission.is_allowed(&vec!["/bin/kill"], &Identity::Gid(123), &Identity::Uid(1000), &Identity::Gid(1000)));
    assert!(!permission.is_allowed(&vec!["/bin/vls"], &Identity::Gid(123), &Identity::Uid(1000), &Identity::Gid(1000)));
}
