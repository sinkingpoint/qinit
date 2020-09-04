use nix::sys::socket::{socket, bind, SockAddr, AddressFamily, SockFlag, SockType, SockProtocol, NetlinkAddr, MsgFlags};
use nix::unistd::{getpid, close};
use super::error::NetLinkError;
use io::RawFdReceiver;

use std::os::unix::io::{RawFd};
use std::io::{self, Read};

pub struct NetLinkSocket {
    socket_fd: RawFd,
    reader: RawFdReceiver,
    sequence_number: u32,
    address: NetlinkAddr
}

impl NetLinkSocket {
    pub fn new() -> Result<NetLinkSocket, NetLinkError> {
        let socket_fd = socket(AddressFamily::Netlink, SockType::Datagram, SockFlag::empty(), SockProtocol::NetlinkKObjectUEvent)?;
        let address = NetlinkAddr::new(getpid().as_raw() as u32, 1);
        bind(socket_fd, &SockAddr::Netlink(address))?;
        return Ok(NetLinkSocket {
            socket_fd: socket_fd,
            reader: RawFdReceiver::new(socket_fd, MsgFlags::empty()),
            sequence_number: 1,
            address: address
        });
    }
}

impl Read for NetLinkSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return self.reader.read(buf);
    }
}

impl Drop for NetLinkSocket {
    fn drop(&mut self) {
        // TODO: Store subscription IDs and unsubscribe here
        if self.socket_fd != 0 {
            close(self.socket_fd).expect("Failed to close socket");
            self.socket_fd = 0;
        }
    }
}