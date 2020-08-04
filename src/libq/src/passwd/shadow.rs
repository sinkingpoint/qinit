use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use super::crypt_sha2::{crypt_sha2, Sha2Mode};

/// An enum representing a possible algorithm used to hash
/// a password in /etc/shadow. Currently we only support hashing SHA2 variants,
/// but we include the others for completeness
#[derive(Debug, PartialEq)]
pub enum PasswdAlgorithm {
    /// Indicates the hash has been done using MD5
    MD5,

    /// Indicates the hash has been done using BCrypt/Blowfish
    /// Note: the BCrypt variant has a couple of differnt version strings
    /// so we store which one was used in order to be able to write it back if necessary
    BCrypt(String),

    /// Indicates the hash has been done using SHA256
    SHA256,

    /// Indicates the hash has been done using SHA512
    SHA512,
}

impl PasswdAlgorithm {
    /// Gets a Passwd Algorithm from a given ID in a /etc/shadow line
    /// or returns None if the given ID doesn't match a known crypt(3) algorithm
    pub fn from_id(id: &str) -> Option<PasswdAlgorithm> {
        return match id {
            "1" => Some(PasswdAlgorithm::MD5),
            "2" | "2a" | "2x" | "2y" => Some(PasswdAlgorithm::BCrypt(id.to_string())),
            "5" => Some(PasswdAlgorithm::SHA256),
            "6" => Some(PasswdAlgorithm::SHA512),
            _ => None,
        };
    }

    /// Converts the algorithm to an ID string, ready for generating a line in /etc/shadow. For anything except BCrypt this is a static string
    /// For BCrypt we return the version string created with this PasswdAlgorithm
    pub fn to_string(&self) -> String {
        return match self {
            PasswdAlgorithm::BCrypt(algo) => algo.as_str(),
            PasswdAlgorithm::MD5 => "1",
            PasswdAlgorithm::SHA256 => "5",
            PasswdAlgorithm::SHA512 => "6",
        }
        .to_string();
    }

    /// Hashes the given salt and password using the given number of rounds using this algorithm
    /// Currently we only support SHA2 variants - everything else will return None indicating an unsupport algorithm
    /// If the given rounds is None, its up to the given algorithm method to handle that
    pub fn hash(&self, salt: &[u8], password: &[u8], rounds: Option<u32>) -> Option<String> {
        return match self {
            PasswdAlgorithm::SHA256 => crypt_sha2(salt, password, Sha2Mode::Sha256(rounds)),
            PasswdAlgorithm::SHA512 => crypt_sha2(salt, password, Sha2Mode::Sha512(rounds)),
            _ => None,
        };
    }
}

/// A struct representing a breakdown of a unix password hash from /etc/shadow
pub struct UnixPasswordHash {
    pub algorithm: PasswdAlgorithm,
    pub rounds: Option<u32>,
    pub salt: String,
    pub hash: String,
}

impl UnixPasswordHash {
    /// Parses out a UnixPasswordHash from a string in the form of a hash from /etc/shadow
    /// i.e. $6$password , $6$salt$password or $6$rounds=1000$salt$password
    pub fn from_unix_hash_str(hash: &str) -> Option<UnixPasswordHash> {
        let parts: Vec<&str> = hash.split("$").collect();
        // In a well formatted string, hash should always start with a $, so parts[0] == ""
        if parts[0] != "" {
            return None;
        }

        let algorithm = match PasswdAlgorithm::from_id(parts[1]) {
            Some(alg) => alg,
            None => {
                return None;
            }
        };

        if parts.len() == 3 {
            // We only have an algorithm and a password hash
            return Some(UnixPasswordHash {
                algorithm: algorithm,
                rounds: None,
                salt: String::new(),
                hash: parts[2].to_owned(),
            });
        } else if parts.len() == 4 {
            // We have an algorithm, a salt and a password hash
            return Some(UnixPasswordHash {
                algorithm: algorithm,
                rounds: None,
                salt: parts[2].to_owned(),
                hash: parts[3].to_owned(),
            });
        } else if parts.len() == 5 {
            // We have an algorithm, rounds, a salt and a password hash

            // If we have any other kvs except for rounds=, reject the line as unsupported
            if !parts[2].starts_with("rounds=") {
                return None;
            }

            // Try parse out the rounds=num string as an int. If its not, reject the line as malformed
            let rounds = match parts[2].trim_start_matches("rounds=").parse::<u32>() {
                Ok(rounds) => rounds,
                Err(_) => {
                    return None;
                }
            };

            return Some(UnixPasswordHash {
                algorithm: algorithm,
                rounds: Some(rounds),
                salt: parts[3].to_owned(),
                hash: parts[4].to_owned(),
            });
        }

        return None;
    }

    pub fn from_unix_hash(hash: &String) -> Option<UnixPasswordHash> {
        return UnixPasswordHash::from_unix_hash_str(hash.as_str());
    }

    /// Verifies that the given password, coupled with the algorithm and hash of this UnixPasswordHash
    /// hashes into the same hash as this hash
    pub fn verify_str(&self, password: &str) -> bool {
        return match self.algorithm.hash(&self.salt.as_bytes(), &password.as_bytes(), self.rounds) {
            Some(b64digest) => b64digest == self.hash,
            None => false, // Unsupported algorithm, or an invalid hash
        };
    }

    /// Verifies that the given password, coupled with the algorithm and hash of this UnixPasswordHash
    /// hashes into the same hash as this hash
    pub fn verify(&self, password: &String) -> bool {
        return self.verify_str(password.as_str());
    }
}

/// A struct representing a breakdown of a single line in /etc/shadow
pub struct ShadowEntry {
    pub username: String,
    pub password_hash: UnixPasswordHash,
    pub day_of_last_change: u32,
    pub min_time_days: u32,
    pub max_time_days: u32,
    pub warn_time_days: u32,
    pub inactive_time_days: u32,
    pub expire_time_days: u32,
}

impl ShadowEntry {
    pub fn from_shadow_line(line: &String) -> Result<ShadowEntry, String> {
        let parts: Vec<&str> = line.split(":").collect();
        if parts.len() != 9 {
            return Err(format!("Invalid shadow line. Expected 8 parts, got {}", parts.len()));
        }

        let pw_hash = match UnixPasswordHash::from_unix_hash_str(parts[1]) {
            Some(hash) => hash,
            None => {
                return Err("Invalid password hash".to_owned());
            }
        };

        return Ok(ShadowEntry {
            username: parts[0].to_string(),
            password_hash: pw_hash,
            day_of_last_change: parts[2].parse().unwrap_or(0),
            min_time_days: parts[3].parse().unwrap_or(0),
            max_time_days: parts[4].parse().unwrap_or(0),
            warn_time_days: parts[5].parse().unwrap_or(0),
            inactive_time_days: parts[6].parse().unwrap_or(0),
            expire_time_days: parts[7].parse().unwrap_or(0),
        });
    }

    pub fn by_username_str(username: &str) -> Option<ShadowEntry> {
        return Shadows::new().filter(|u| &u.username == username).next();
    }

    pub fn by_username(username: &String) -> Option<ShadowEntry> {
        return Shadows::new().filter(|u| &u.username == username).next();
    }
}

/// An iterator over the ShadowEntry's in /etc/shadow
pub struct Shadows {
    lines: Lines<BufReader<File>>,
}

impl Shadows {
    pub fn new() -> Shadows {
        let file = File::open("/etc/shadow").expect("Failed to open /etc/shadow");
        return Shadows {
            lines: BufReader::new(file).lines(),
        };
    }
}

impl Iterator for Shadows {
    type Item = ShadowEntry;
    fn next(&mut self) -> Option<Self::Item> {
        return match self.lines.next() {
            Some(Ok(line)) => Some(ShadowEntry::from_shadow_line(&line).unwrap()),
            _ => None,
        };
    }
}
