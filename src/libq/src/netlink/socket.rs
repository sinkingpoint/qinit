use nix::sys::socket::{socket, bind, SockAddr, AddressFamily, SockFlag, SockType, SockProtocol, NetlinkAddr, MsgFlags, setsockopt, SetSockOpt};
use nix::sys::socket::sockopt::RcvBuf;
use nix::unistd::{getpid, close};
use std::os::unix::io::{RawFd, FromRawFd};
use super::api::NetLinkRawMessage;
use super::error::NetLinkError;
use io::RawFdReceiver;

pub struct NetLinkSocket {
    socket_fd: RawFd,
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
            sequence_number: 1,
            address: address
        });
    }

    pub fn read_raw_message(&self) -> Result<NetLinkRawMessage, NetLinkError> {
        let mut reader = unsafe { RawFdReceiver::new(self.socket_fd, MsgFlags::empty()) };
        return NetLinkRawMessage::read(&mut reader);
    }
}

impl Drop for NetLinkSocket {
    fn drop(&mut self) {
        if self.socket_fd != 0 {
            close(self.socket_fd);
            self.socket_fd = 0;
        }
    }
}