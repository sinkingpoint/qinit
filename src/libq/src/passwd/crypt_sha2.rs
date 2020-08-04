use std::cmp::{max, min};

use ring::digest::{self, digest, Digest};

pub enum Sha2Mode {
    Sha256(Option<u32>),
    Sha512(Option<u32>),
}

const ROUNDS_DEFAULT: u32 = 5000;
const ROUNDS_MAX: u32 = 999_999_999;
const ROUNDS_MIN: u32 = 1000;

const B64_TABLE: [char; 64] = [
    '.', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Converts the given 3 bytes into 4 offsets in the Base64 Table
fn crypt_sha2_base64_bytes(c: u8, b: u8, a: u8) -> [u8; 4] {
    let w = ((c as u32) << 16) | ((b as u32) << 8) | (a as u32);
    return [
        ((w >> 0) & 0b111111) as u8,
        ((w >> 6) & 0b111111) as u8,
        ((w >> 12) & 0b111111) as u8,
        ((w >> 18) & 0b111111) as u8,
    ];
}

/// Encodes the given data slice (A Sha digest), into a base64 string
/// using the given mode. Returns none if the given data slice is not
/// valid for the given mode (i.e. 32 bytes for sha256, 64 for sha512)
fn crypt_sha2_base64(data: &[u8], mode: Sha2Mode) -> Option<String> {
    match mode {
        Sha2Mode::Sha256(_) => {
            if data.len() != 32 {
                return None;
            }
        }
        Sha2Mode::Sha512(_) => {
            if data.len() != 64 {
                return None;
            }
        }
    };

    // Each mode shuffles bytes differently
    // This table arranges the bytes into the proper order, as 4 tuples
    // where the first three values are the bytes, and the fourth is the number of characters
    // to extract from those bytes
    let bytes = match mode {
        Sha2Mode::Sha256(_) => vec![
            (data[0], data[10], data[20], 4),
            (data[21], data[1], data[11], 4),
            (data[12], data[22], data[2], 4),
            (data[3], data[13], data[23], 4),
            (data[24], data[4], data[14], 4),
            (data[15], data[25], data[5], 4),
            (data[6], data[16], data[26], 4),
            (data[27], data[7], data[17], 4),
            (data[18], data[28], data[8], 4),
            (data[9], data[19], data[29], 4),
            (0, data[31], data[30], 3),
        ],
        Sha2Mode::Sha512(_) => vec![
            (data[0], data[21], data[42], 4),
            (data[22], data[43], data[1], 4),
            (data[44], data[2], data[23], 4),
            (data[3], data[24], data[45], 4),
            (data[25], data[46], data[4], 4),
            (data[47], data[5], data[26], 4),
            (data[6], data[27], data[48], 4),
            (data[28], data[49], data[7], 4),
            (data[50], data[8], data[29], 4),
            (data[9], data[30], data[51], 4),
            (data[31], data[52], data[10], 4),
            (data[53], data[11], data[32], 4),
            (data[12], data[33], data[54], 4),
            (data[34], data[55], data[13], 4),
            (data[56], data[14], data[35], 4),
            (data[15], data[36], data[57], 4),
            (data[37], data[58], data[16], 4),
            (data[59], data[17], data[38], 4),
            (data[18], data[39], data[60], 4),
            (data[40], data[61], data[19], 4),
            (data[62], data[20], data[41], 4),
            (0, 0, data[63], 2),
        ],
    };

    let mut byte_iter = bytes.into_iter().peekable();
    let mut encode = String::new();
    while byte_iter.peek().is_some() {
        let (a, b, c, n) = byte_iter.next().unwrap();

        let bytes = crypt_sha2_base64_bytes(a, b, c);
        for i in 0..n {
            encode.push(B64_TABLE[bytes[i] as usize]);
        }
    }

    return Some(encode);
}

/// Replicates the functionality of crypt(3), encoding a salt and password
/// using the given Sha2 mode and returning the base64 hash.
/// NOTE: This only returns the hash, not the fully encoded /etc/shadow string,
/// e.g. foo rather than $6$salt$rounds=$foo
pub fn crypt_sha2(salt: &[u8], password: &[u8], mode: Sha2Mode) -> Option<String> {
    let (algorithm, rounds) = match mode {
        Sha2Mode::Sha256(rounds) => (&digest::SHA256, rounds),
        Sha2Mode::Sha512(rounds) => (&digest::SHA512, rounds),
    };

    // Clamp rounds to ROUNDS_MIN <= rounds <= ROUNDS_MAX, using ROUNDS_DEFAULT if it doesn't exist
    let rounds = match rounds {
        Some(rounds) => max(ROUNDS_MIN, min(ROUNDS_MAX, rounds)),
        None => ROUNDS_DEFAULT,
    };

    // Based off of https://akkadia.org/drepper/SHA-crypt.txt

    // start digest A
    let mut digest_a = Vec::new();

    // the password string is added to digest A
    for byte in password.iter() {
        digest_a.push(*byte);
    }

    // the salt string is added to digest A
    for byte in salt.iter() {
        digest_a.push(*byte);
    }

    let mut digest_b = Vec::new();
    // add the password to digest B
    for byte in password.iter() {
        digest_b.push(*byte);
    }

    // add the salt string to digest B
    for byte in salt.iter() {
        digest_b.push(*byte);
    }

    // add the password again to digest B
    for byte in password.iter() {
        digest_b.push(*byte);
    }

    // finish digest B
    let digest_b = digest(algorithm, &digest_b[..]);

    digest_a.append(&mut digest_b.as_ref().iter().map(|x| *x).cycle().take(password.len()).collect());

    // For each bit of the binary representation of the length of the
    // password string up to and including the highest 1-digit, starting
    // from to lowest bit position (numeric value 1):
    let mut length = password.len();
    while length > 0 {
        if length & 1 == 1 {
            // for a 1-digit add digest B to digest A
            for byte in digest_b.as_ref().iter() {
                digest_a.push(*byte);
            }
        } else {
            // for a 0-digit add the password string
            for byte in password.iter() {
                digest_a.push(*byte);
            }
        }
        length >>= 1;
    }

    // finish digest A
    let digest_a = digest(algorithm, &digest_a[..]);

    // start digest DP
    let mut digest_dp = Vec::new();

    // for every byte in the password (excluding the terminating NUL byte
    // in the C representation of the string)
    // add the password to digest DP
    for _ in 0..password.len() {
        for byte in password.iter() {
            digest_dp.push(*byte);
        }
    }

    // finish digest DP
    let digest_dp = digest(algorithm, &digest_dp[..]);

    // produce byte sequence P of the same length as the password where
    // a) for each block of 32 or 64 bytes of length of the password string
    // the entire digest DP is used
    // b) for the remaining N (up to  31 or 63) bytes use the first N
    // bytes of digest DP
    let p: Vec<u8> = digest_dp.as_ref().iter().map(|x| *x).cycle().take(password.len()).collect();

    // start digest DS
    let mut digest_ds = Vec::new();
    // repeat the following 16+A[0] times, where A[0] represents the first
    // byte in digest A interpreted as an 8-bit unsigned value
    for _ in 0..16 + (digest_a.as_ref()[0] as u32) {
        // add the salt to digest DS
        for byte in salt.iter() {
            digest_ds.push(*byte);
        }
    }
    let digest_ds = digest(algorithm, &digest_ds[..]);

    let s: Vec<u8> = digest_ds.as_ref().iter().map(|x| *x).cycle().take(salt.len()).collect();

    let mut previous_digest: Digest = digest_a;

    for round in 0..rounds {
        // println!("{}", round);
        let mut digest_c = Vec::new();
        // for odd round numbers add the byte sequense P to digest C
        if round % 2 == 1 {
            for byte in p.iter() {
                digest_c.push(*byte);
            }
        } else {
            // for even round numbers add digest A/C
            for byte in previous_digest.as_ref().iter() {
                digest_c.push(*byte);
            }
        }

        if round % 3 != 0 {
            for byte in s.iter() {
                digest_c.push(*byte);
            }
        }

        if round % 7 != 0 {
            for byte in p.iter() {
                digest_c.push(*byte);
            }
        }

        if round % 2 == 1 {
            for byte in previous_digest.as_ref().iter() {
                digest_c.push(*byte);
            }
        } else {
            for byte in p.iter() {
                digest_c.push(*byte);
            }
        }

        previous_digest = digest(algorithm, &digest_c[..]);
    }

    return crypt_sha2_base64(previous_digest.as_ref(), mode);
}
