use std::fs::File;
use std::io::{BufReader, BufRead, Lines};
use std::path::PathBuf;

/// Represents a single line of /etc/passwd
#[derive(Debug)]
pub struct PasswdEntry {
    pub username: String,
    pub password: String,
    pub uid: u32,
    pub gid: u32,
    pub groups: Vec<String>,
    pub home: PathBuf,
    pub shell: PathBuf
}

pub struct Users {
    lines: Lines<BufReader<File>>
}

impl Users {
    pub fn new() -> Users {
        let file = File::open("/etc/passwd").expect("Failed to open /etc/passwd");
        return Users {
            lines: BufReader::new(file).lines()
        }
    }
}

impl Iterator for Users {
    type Item = PasswdEntry;
    fn next(&mut self) -> Option<Self::Item> {
        return match self.lines.next() {
            Some(Ok(line)) => Some(PasswdEntry::from_passwd_line(&line).unwrap()),
            _ => None
        }
    }
}

impl PasswdEntry {
    fn from_passwd_line(line: &String) -> Result<PasswdEntry, String> {
        let parts: Vec<&str> = line.split(":").collect();
        if parts.len() != 7 {
            return Err(format!("Invalid passwd line. Expected 7 parts, got {}", parts.len()));
        }

        return Ok(PasswdEntry {
            username: parts[0].to_string(),
            password: parts[1].to_string(),
            uid: parts[2].parse().unwrap(),
            gid: parts[3].parse().unwrap(),
            groups: parts[4].split_whitespace().map(|s| s.to_string()).collect(),
            home: PathBuf::from(parts[5]),
            shell: PathBuf::from(parts[6])
        });
    }

    pub fn by_uid(uid: u32) -> Option<PasswdEntry> {
        return Users::new().filter(|u| u.uid == uid).next();
    }

    pub fn by_username_str(username: &str) -> Option<PasswdEntry> {
        return Users::new().filter(|u| &u.username == username).next();
    }

    pub fn by_username(username: &String) -> Option<PasswdEntry> {
        return Users::new().filter(|u| &u.username == username).next();
    }
}
