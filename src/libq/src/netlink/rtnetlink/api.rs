use super::addr_types::{Address, AddressRoutingAttributes, InterfaceAddrMessage};
use super::link_types::{Interface, InterfaceInfoMessage, InterfaceRoutingAttributes};
use io::{BufferReader, Readable, Writable};
use netlink::api::{MessageType, NetLinkMessageFlags, NetLinkMessageHeader};
use netlink::error::NetLinkError;
use netlink::NetLinkSocket;
use nix::sys::socket::SockProtocol;
use std::io::{Read, Write};

pub struct Addrs<'a> {
    socket: &'a mut NetLinkSocket,
    done: bool,
}

impl<'a> Addrs<'a> {
    fn new(socket: &'a mut NetLinkSocket) -> Result<Addrs<'a>, NetLinkError> {
        let request = InterfaceAddrMessage::empty();
        let header = NetLinkMessageHeader::new(
            MessageType::RTM_GETADDR,
            socket.get_next_sequence_number(),
            NetLinkMessageHeader::size() + InterfaceAddrMessage::size(),
            NetLinkMessageFlags::NLM_F_REQUEST | NetLinkMessageFlags::NLM_F_MATCH,
        );
        let mut buffer = Vec::new();
        header.write(&mut buffer)?;
        request.write(&mut buffer)?;

        socket.write_all(&mut buffer)?;

        return Ok(Addrs {
            socket: socket,
            done: false,
        });
    }
}

impl Iterator for Addrs<'_> {
    type Item = Result<Address, NetLinkError>;
    fn next(&mut self) -> Option<Self::Item> {
        let header = match NetLinkMessageHeader::read(self.socket) {
            Ok(header) => header,
            Err(e) => {
                return Some(Err(e));
            }
        };

        if header.msg_type == MessageType::NLMSG_DONE {
            self.done = true;
            return None;
        }

        let interface_message = match InterfaceAddrMessage::read(self.socket) {
            Ok(msg) => msg,
            Err(e) => {
                return Some(Err(e));
            }
        };

        let rtinfo_length = (header.length - NetLinkMessageHeader::size() - InterfaceAddrMessage::size()) as usize;
        let mut buffer = vec![0; rtinfo_length];
        match self.socket.read_exact(&mut buffer) {
            Ok(()) => {}
            Err(err) => {
                return Some(Err(err.into()));
            }
        }

        let mut reader = BufferReader::new(&buffer);
        let mut rattrs = AddressRoutingAttributes::new();

        while reader.has_more() {
            match rattrs.read_new_attr(&mut reader) {
                Ok(_) => {}
                Err(err) => {
                    return Some(Err(err));
                }
            }
        }

        return Some(Ok(Address::from_raw_messages(interface_message, rattrs)));
    }
}

pub struct Links<'a> {
    socket: &'a mut NetLinkSocket,
    done: bool,
}

impl<'a> Links<'a> {
    fn new(socket: &'a mut NetLinkSocket) -> Result<Links<'a>, NetLinkError> {
        let request = InterfaceInfoMessage::empty();
        let header = NetLinkMessageHeader::new(
            MessageType::RTM_GETLINK,
            socket.get_next_sequence_number(),
            NetLinkMessageHeader::size() + InterfaceInfoMessage::size(),
            NetLinkMessageFlags::NLM_F_REQUEST | NetLinkMessageFlags::NLM_F_MATCH,
        );
        let mut buffer = Vec::new();
        header.write(&mut buffer)?;
        request.write(&mut buffer)?;

        socket.write_all(&mut buffer)?;

        return Ok(Links {
            socket: socket,
            done: false,
        });
    }
}

impl Iterator for Links<'_> {
    type Item = Result<Interface, NetLinkError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let header = match NetLinkMessageHeader::read(self.socket) {
            Ok(header) => header,
            Err(e) => {
                return Some(Err(e));
            }
        };

        if header.msg_type == MessageType::NLMSG_DONE {
            self.done = true;
            return None;
        }

        let interface_message = match InterfaceInfoMessage::read(self.socket) {
            Ok(msg) => msg,
            Err(e) => {
                return Some(Err(e));
            }
        };

        let rtinfo_length = (header.length - NetLinkMessageHeader::size() - InterfaceInfoMessage::size()) as usize;
        let mut buffer = vec![0; rtinfo_length];
        match self.socket.read_exact(&mut buffer) {
            Ok(()) => {}
            Err(err) => {
                return Some(Err(err.into()));
            }
        }

        let mut reader = BufferReader::new(&buffer);
        let mut rattrs = InterfaceRoutingAttributes::new();

        while reader.has_more() {
            match rattrs.read_new_attr(&mut reader) {
                Ok(_) => {}
                Err(err) => {
                    return Some(Err(err));
                }
            }
        }

        return Some(Ok(Interface::from_raw_messages(interface_message, rattrs)));
    }
}

pub trait RTNetlink {
    fn get_links(&mut self) -> Result<Vec<Interface>, NetLinkError>;
    fn get_addrs(&mut self) -> Result<Vec<Address>, NetLinkError>;
    fn set_link(&mut self, interface: &Interface) -> Result<(), NetLinkError>;
}

impl RTNetlink for NetLinkSocket {
    fn get_links(&mut self) -> Result<Vec<Interface>, NetLinkError> {
        if self.protocol != SockProtocol::NetlinkRoute {
            return Err(NetLinkError::InvalidNetlinkProtocol);
        }

        return Links::new(self)?.collect();
    }

    fn get_addrs(&mut self) -> Result<Vec<Address>, NetLinkError> {
        if self.protocol != SockProtocol::NetlinkRoute {
            return Err(NetLinkError::InvalidNetlinkProtocol);
        }

        return Addrs::new(self)?.collect();
    }

    fn set_link(&mut self, interface: &Interface) -> Result<(), NetLinkError> {
        let mut cmd = Vec::new();

        let mut rtattrs = Vec::new();
        interface.rtattrs.write(&mut rtattrs);

        let header = NetLinkMessageHeader::new(
            MessageType::RTM_NEWLINK,
            self.get_next_sequence_number(),
            NetLinkMessageHeader::size() + InterfaceInfoMessage::size() + (rtattrs.len() as u32),
            NetLinkMessageFlags::NLM_F_ACK | NetLinkMessageFlags::NLM_F_REQUEST,
        );

        header.write(&mut cmd);
        interface.write(&mut cmd);
        cmd.append(&mut rtattrs);

        self.write_all(&cmd)?;

        return Ok(());
    }
}
