use std::fs::File;
use std::io::{BufReader, BufRead, Lines};

/// Represents a single line of /etc/group
pub struct GroupEntry {
    pub name: String,
    pub gid: u32,
    pub users: Vec<String>
}

impl GroupEntry {
    fn from_group_line(line: &String) -> Result<GroupEntry, String> {
        let parts: Vec<&str> = line.split(":").collect();
        if parts.len() != 4 {
            return Err(format!("Invalid groups line. Expected 4 parts, got {}", parts.len()));
        }

        return Ok(GroupEntry{
            name: parts[0].to_string(),
            gid: parts[2].parse().unwrap(),
            users: parts[3].split_whitespace().map(|s| s.to_string()).collect(),
        });
    }

    pub fn by_groupname(name: &String) -> Option<GroupEntry> {
        return Groups::new().filter(|g| &g.name == name).next();
    }

    pub fn by_gid(gid: u32) -> Option<GroupEntry> {
        return Groups::new().filter(|g| g.gid == gid).next();
    }
}

pub struct Groups {
    lines: Lines<BufReader<File>>
}

impl Groups {
    pub fn new() -> Groups {
        let file = File::open("/etc/group").expect("Failed to open /etc/group");
        return Groups {
            lines: BufReader::new(file).lines()
        }
    }
}

impl Iterator for Groups {
    type Item = GroupEntry;
    fn next(&mut self) -> Option<Self::Item> {
        return match self.lines.next() {
            Some(Ok(line)) => Some(GroupEntry::from_group_line(&line).unwrap()),
            _ => None
        };
    }
}