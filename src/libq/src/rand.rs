use std::fs::File;
use std::io;
use std::io::Read;

pub fn fill_exact(buf: &mut [u8]) -> Result<(), io::Error> {
    let mut file = File::open("/dev/urandom")?;
    return file.read_exact(buf);
}
