use nix::sys::socket::{recv, MsgFlags};
use nix::sys::stat::FileStat;
use nix::unistd::{read, write};

use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::PathBuf;

use mem::{int_from_bytes_little_endian, long_from_bytes_little_endian, short_from_bytes_little_endian};

pub const STDIN_FD: RawFd = 0;
pub const STDOUT_FD: RawFd = 1;
pub const STDERR_FD: RawFd = 2;

pub trait Writable {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error>;
}

impl Writable for Vec<u8> {
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), io::Error> {
        writer.write_all(self)?;
        return Ok(());
    }
}

pub fn full_write_bytes(fd: RawFd, buf: &[u8]) -> nix::Result<usize> {
    let mut count: usize = 0;
    while count < buf.len() {
        let size = match write(fd, &buf[count..]) {
            Ok(size) => size,
            Err(errno) => {
                if count == 0 {
                    return Err(errno);
                } else {
                    return Ok(count);
                }
            }
        };

        count += size;
    }

    return Ok(count);
}

pub fn full_write_str(fd: RawFd, buf: &String) -> nix::Result<usize> {
    return full_write_bytes(fd, buf.as_bytes());
}

pub struct RawFdReader {
    /// RawFdReader provides a Read interface on a RawFd. Unlike a std::io::File, doesn't claim
    /// ownership of the underlying fd so `close`ing must be handled external to this
    fd: RawFd,
}

impl RawFdReader {
    pub fn new(fd: RawFd) -> Self {
        return Self { fd };
    }
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
        };
    }
}

pub struct RawFdReceiver {
    /// RawFdReader provides a Read interface on a RawFd, using recv instead of . Unlike a std::io::File, doesn't claim
    /// ownership of the underlying fd so `close`ing must be handled external to this
    fd: RawFd,

    flags: MsgFlags,
}

impl RawFdReceiver {
    pub fn new(fd: RawFd, flags: MsgFlags) -> RawFdReceiver {
        return RawFdReceiver { fd: fd, flags: flags };
    }
}

impl Read for RawFdReceiver {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        assert!(buf.len() <= isize::max_value() as usize);
        return match recv(self.fd, buf, self.flags) {
            Ok(amt) => Ok(amt),
            Err(e) => {
                if let Some(errno) = e.as_errno() {
                    return Err(Error::from_raw_os_error(errno as i32));
                }
                return Err(Error::new(ErrorKind::Other, e));
            }
        };
    }
}

pub struct BufferReader<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> BufferReader<'a> {
    pub fn new(buffer: &'a [u8]) -> BufferReader<'a> {
        return BufferReader { buffer: buffer, offset: 0 };
    }

    pub fn has_more(&self) -> bool {
        return self.offset < self.buffer.len();
    }
}

impl<'a> Seek for BufferReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut new_offset: i64 = self.offset as i64;
        match pos {
            SeekFrom::Start(i) => {
                new_offset = i as i64;
            }
            SeekFrom::Current(i) => {
                if -i > new_offset {
                    return Err(io::Error::from(io::ErrorKind::Other));
                }
                new_offset += i;
            }
            SeekFrom::End(i) => {
                if -i > self.buffer.len() as i64 {
                    return Err(io::Error::from(io::ErrorKind::Other));
                }

                new_offset += i;
            }
        }

        self.offset = new_offset as usize;

        return Ok(new_offset as u64);
    }
}

impl<'a> Read for BufferReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut dst_offset = 0;
        while self.offset < self.buffer.len() && dst_offset < buf.len() {
            buf[dst_offset] = self.buffer[self.offset];
            dst_offset += 1;
            self.offset += 1;
        }

        return Ok(dst_offset);
    }
}

pub fn to_absolute_from_relative(base: &PathBuf, rel_path: &PathBuf) -> Result<PathBuf, Error> {
    let mut new_path = PathBuf::new();
    new_path.push(base);
    new_path.push(rel_path);
    return new_path.canonicalize();
}

pub const S_IFMT: u32 = 0o170000;
pub const S_IFSOCK: u32 = 0o140000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFIFO: u32 = 0o010000;

pub const S_ISUID: u32 = 0o04000;
pub const S_ISGID: u32 = 0o02000;

pub const S_IRWXU: u32 = 0o00700;
pub const S_IRWUSR: u32 = 0o00600;
pub const S_IRUSR: u32 = 0o00400;
pub const S_IWUSR: u32 = 0o00200;
pub const S_IXUSR: u32 = 0o00100;

pub const S_IRWXG: u32 = 0o00070;
pub const S_IRWGRP: u32 = 0o00060;
pub const S_IRGRP: u32 = 0o00040;
pub const S_IWGRP: u32 = 0o00020;
pub const S_IXGRP: u32 = 0o00010;

pub const S_IRWXO: u32 = 0o00007;
pub const S_IRWOTH: u32 = 0o00006;
pub const S_IROTH: u32 = 0o00004;
pub const S_IWOTH: u32 = 0o00002;
pub const S_IXOTH: u32 = 0o00001;

#[derive(PartialEq, Debug)]
pub enum FileType {
    Socket,
    Link,
    Regular,
    BlockDevice,
    Directory,
    CharacterDevice,
    Fifo,
}

impl FileType {
    pub fn from_stat(stat_result: FileStat) -> Result<FileType, ()> {
        return FileType::from_mode(stat_result.st_mode);
    }

    pub fn from_mode(mode: u32) -> Result<FileType, ()> {
        return match mode & S_IFMT {
            S_IFSOCK => Ok(FileType::Socket),
            S_IFLNK => Ok(FileType::Link),
            S_IFREG => Ok(FileType::Regular),
            S_IFBLK => Ok(FileType::BlockDevice),
            S_IFDIR => Ok(FileType::Directory),
            S_IFCHR => Ok(FileType::CharacterDevice),
            S_IFIFO => Ok(FileType::Fifo),
            _ => Err(()),
        };
    }

    pub fn to_char(&self) -> char {
        match self {
            FileType::Socket => 's',
            FileType::Link => 'l',
            FileType::Regular => '-',
            FileType::BlockDevice => 'b',
            FileType::Directory => 'd',
            FileType::CharacterDevice => 'c',
            FileType::Fifo => 'f',
        }
    }
}

pub enum Endianness {
    Little,
    Big,
}

pub fn read_u8<T: Read>(r: &mut T) -> io::Result<u8> {
    let mut buffer = [0; 1];
    r.read_exact(&mut buffer)?;
    return Ok(buffer[0]);
}

pub fn read_u16<T: Read>(r: &mut T, endian: &Endianness) -> io::Result<u16> {
    let mut buffer = [0; 2];
    r.read_exact(&mut buffer)?;
    return Ok(match endian {
        Endianness::Little => short_from_bytes_little_endian(buffer[0], buffer[1]),
        Endianness::Big => short_from_bytes_little_endian(buffer[1], buffer[0]),
    });
}

pub fn read_u32<T: Read>(r: &mut T, endian: &Endianness) -> io::Result<u32> {
    let mut buffer = [0; 4];
    r.read_exact(&mut buffer)?;
    return Ok(match endian {
        Endianness::Little => int_from_bytes_little_endian(buffer[0], buffer[1], buffer[2], buffer[3]),
        Endianness::Big => int_from_bytes_little_endian(buffer[3], buffer[2], buffer[1], buffer[0]),
    });
}

pub fn write_u8<T: Write>(w: &mut T, data: u8) -> io::Result<()> {
    let mut buf = [data];

    return w.write_all(&mut buf);
}

pub fn write_u16<T: Write>(w: &mut T, data: u16, endian: &Endianness) -> io::Result<()> {
    let mut buf = match endian {
        Endianness::Little => [(data & 0xFF) as u8, ((data >> 8) & 0xFF) as u8],
        Endianness::Big => [((data >> 8) & 0xFF) as u8, (data & 0xFF) as u8],
    };

    return w.write_all(&mut buf);
}

pub fn write_u32<T: Write>(w: &mut T, data: u32, endian: &Endianness) -> io::Result<()> {
    let mut buf = match endian {
        Endianness::Little => [
            (data & 0xFF) as u8,
            ((data >> 8) & 0xFF) as u8,
            ((data >> 16) & 0xFF) as u8,
            ((data >> 24) & 0xFF) as u8,
        ],
        Endianness::Big => [
            ((data >> 24) & 0xFF) as u8,
            ((data >> 16) & 0xFF) as u8,
            ((data >> 8) & 0xFF) as u8,
            (data & 0xFF) as u8,
        ],
    };

    return w.write_all(&mut buf);
}

pub fn read_u64<T: Read>(r: &mut T, endian: &Endianness) -> io::Result<u64> {
    let mut buffer = [0; 8];
    r.read_exact(&mut buffer)?;
    return Ok(match endian {
        Endianness::Little => long_from_bytes_little_endian(
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ),
        Endianness::Big => long_from_bytes_little_endian(
            buffer[7], buffer[6], buffer[5], buffer[4], buffer[3], buffer[2], buffer[1], buffer[0],
        ),
    });
}
