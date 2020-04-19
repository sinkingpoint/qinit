extern crate nix;

pub mod strings;

pub mod io {
    pub const STDIN_FD: i32 = 0;
    pub const STDOUT_FD: i32 = 1;
    pub fn full_write_bytes(fd: std::os::unix::io::RawFd, buf: &[u8]) -> nix::Result<usize>{
        let mut count: usize = 0;
        while count < buf.len() {
            let size = match nix::unistd::write(fd, &buf[count..]) {
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

    pub fn full_write_str(fd: std::os::unix::io::RawFd, buf: String) -> nix::Result<usize>{
        return full_write_bytes(fd, buf.as_bytes());
    }
}