use super::error::NetLinkError;
use io::RawFdReceiver;
use nix::sys::socket::{bind, send, socket, AddressFamily, MsgFlags, NetlinkAddr, SockAddr, SockFlag, SockProtocol, SockType};
use nix::unistd::{close, getpid};

use std::io::{self, BufReader, Read, Write};
use std::os::unix::io::RawFd;

pub struct NetLinkSocket {
    socket_fd: RawFd,
    reader: BufReader<RawFdReceiver>,
    sequence_number: u32,
    pub protocol: SockProtocol,
}

impl NetLinkSocket {
    pub fn new(protocol: SockProtocol) -> Result<NetLinkSocket, NetLinkError> {
        let socket_fd = socket(AddressFamily::Netlink, SockType::Datagram, SockFlag::empty(), protocol)?;

        let address = NetlinkAddr::new(getpid().as_raw() as u32, 1);

        bind(socket_fd, &SockAddr::Netlink(address))?;

        return Ok(NetLinkSocket {
            socket_fd: socket_fd,
            reader: BufReader::new(RawFdReceiver::new(socket_fd, MsgFlags::empty())),
            sequence_number: 1,
            protocol: protocol,
        });
    }

    pub fn get_next_sequence_number(&mut self) -> u32 {
        self.sequence_number += 1;
        return self.sequence_number - 1;
    }

    pub fn new_rtnetlink() -> Result<NetLinkSocket, NetLinkError> {
        return NetLinkSocket::new(SockProtocol::NetlinkRoute);
    }

    pub fn new_uevent() -> Result<NetLinkSocket, NetLinkError> {
        return NetLinkSocket::new(SockProtocol::NetlinkKObjectUEvent);
    }
}

impl Read for NetLinkSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        return self.reader.read(buf);
    }
}

impl Write for NetLinkSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return match send(self.socket_fd, buf, MsgFlags::empty()) {
            Ok(r) => Ok(r),
            Err(err) => Err(io::Error::from_raw_os_error(err.as_errno().unwrap() as i32)),
        };
    }

    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
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
