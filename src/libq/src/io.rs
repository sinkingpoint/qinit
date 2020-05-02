use std::os::unix::io::{FromRawFd, RawFd};
use nix::unistd::{write, read};
use std::io::{Read, Error, ErrorKind};

pub const STDIN_FD: RawFd = 0;
pub const STDOUT_FD: RawFd = 1;
pub const STDERR_FD: RawFd = 2;
pub fn full_write_bytes(fd: RawFd, buf: &[u8]) -> nix::Result<usize>{
    let mut count: usize = 0;
    while count < buf.len() {
        let size = match write(fd, &buf[count..]) {
            Ok(size) => size,
            Err(errno) => {
                if count == 0 {
                    return Err(errno);
                }
                else {
                    return Ok(count);
                }
            }
        };

        count += size;
    }

    return Ok(count);
}

pub fn full_write_str(fd: RawFd, buf: &String) -> nix::Result<usize>{
    return full_write_bytes(fd, buf.as_bytes());
}

pub struct RawFdReader {
    /// RawFdReader provides a Read interface on a RawFd. Unlike a std::io::File, doesn't claim
    /// ownership of the underlying fd so `close`ing must be handled external to this
    fd: RawFd
}

impl FromRawFd for RawFdReader {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }   
}

impl Read for RawFdReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        assert!(buf.len() <= isize::max_value() as usize);
        return match read(self.fd, buf) {
            Ok(amt) => Ok(amt),
            Err(e) => {
                if let Some(errno) = e.as_errno() {
                    return Err(Error::from_raw_os_error(errno as i32));
                }
                return Err(Error::new(ErrorKind::Other, e));
            }
        }
    }
}