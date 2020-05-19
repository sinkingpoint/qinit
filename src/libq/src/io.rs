use std::os::unix::io::{FromRawFd, RawFd};
use std::io::{self, Read, Error, ErrorKind};
use std::path::PathBuf;
use nix::unistd::{write, read};
use nix::sys::stat::FileStat;

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

pub fn read_struct<T, R: Read>(mut read: R) -> io::Result<T> {
    let num_bytes = ::std::mem::size_of::<T>();
    unsafe {
        let mut s = std::mem::MaybeUninit::<T>::uninit();
        let buffer = std::slice::from_raw_parts_mut((&mut s).as_ptr() as *mut T as *mut u8, num_bytes);
        match read.read_exact(buffer) {
            Ok(()) => Ok(s.assume_init()),
            Err(e) => {
                ::std::mem::forget(s);
                Err(e)
            }
        }
    }
}

pub fn to_absolute_from_relative(base: &PathBuf, rel_path: &PathBuf) -> Result<PathBuf, Error> {
    let mut new_path = PathBuf::new();
    new_path.push(base);
    new_path.push(rel_path);
    return new_path.canonicalize();
}

pub const S_IFMT:   u32 = 0o170000;
pub const S_IFSOCK: u32 = 0o140000;
pub const S_IFLNK:  u32 = 0o120000;
pub const S_IFREG:  u32 = 0o100000;
pub const S_IFBLK:  u32 = 0o060000;
pub const S_IFDIR:  u32 = 0o040000;
pub const S_IFCHR:  u32 = 0o020000;
pub const S_IFIFO:  u32 = 0o010000;

pub const S_ISUID: u32 = 0o04000;
pub const S_ISGID: u32 = 0o02000;

pub const S_IRWXU: u32 = 0o00700;
pub const S_IRUSR: u32 = 0o00400;
pub const S_IWUSR: u32 = 0o00200;
pub const S_IXUSR: u32 = 0o00100;

pub const S_IRWXG: u32 = 0o00070;
pub const S_IRGRP: u32 = 0o00040;
pub const S_IWGRP: u32 = 0o00020;
pub const S_IXGRP: u32 = 0o00010;

pub const S_IRWXO: u32 = 0o00007;
pub const S_IROTH: u32 = 0o00004;
pub const S_IWOTH: u32 = 0o00002;
pub const S_IXOTH: u32 = 0o00001;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum FileType {
    Socket,
    Link,
    Regular,
    BlockDevice,
    Directory,
    CharacterDevice,
    Fifo
}

impl FileType {
    pub fn from_stat(stat_result: FileStat) -> Result<FileType, ()>{
        return FileType::from_mode(stat_result.st_mode);
    }

    pub fn from_mode(mode: u32) -> Result<FileType, ()> {
        return match mode & S_IFMT {
            S_IFSOCK => Ok(FileType::Socket),
            S_IFLNK  => Ok(FileType::Link),
            S_IFREG  => Ok(FileType::Regular),
            S_IFBLK  => Ok(FileType::BlockDevice),
            S_IFDIR  => Ok(FileType::Directory),
            S_IFCHR  => Ok(FileType::CharacterDevice),
            S_IFIFO  => Ok(FileType::Fifo),
            _ => Err(())
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            FileType::Socket => 's',
            FileType::Link => 'l',
            FileType::Regular => '-',
            FileType::BlockDevice => 'b',
            FileType::Directory => 'd',
            FileType::CharacterDevice => 'c',
            FileType::Fifo => 'f'
        }
    }
}