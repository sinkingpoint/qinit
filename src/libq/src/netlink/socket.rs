use nix::sys::socket::{sendmsg, socket, AddressFamily, MsgFlags, SockFlag, SockType};
use nix::sys::uio::IoVec;
use std::os::unix::io::RawFd;

use super::super::io::RawFdReader;
use super::super::mem::any_as_u8_slice;
use super::api::{MessageType, NLMsgHeader};
use super::commands::{GetInterfacesCommand, Interface, NewInterfaceCommand};
use std::io::{BufReader, Read};
use std::os::unix::io::FromRawFd;

pub struct NLSocket {
    socket_fd: RawFd,
    sequence_number: u32,
}

impl NLSocket {
    pub fn new() -> NLSocket {
        return NLSocket {
            socket_fd: socket(AddressFamily::Netlink, SockType::Datagram, SockFlag::empty(), None).unwrap(),
            sequence_number: 1,
        };
    }

    pub fn get_interfaces(&self) -> Vec<Interface> {
        let data;
        let command = GetInterfacesCommand::new(self.sequence_number);
        unsafe {
            data = any_as_u8_slice(&command);
        }

        match sendmsg(self.socket_fd, &[IoVec::from_slice(data)], &[], MsgFlags::empty(), None) {
            Ok(_) => {}
            // TODO: Send back an error message here
            Err(_e) => {}
        }

        let mut reader = unsafe { BufReader::new(RawFdReader::from_raw_fd(self.socket_fd)) };
        let mut header_buffer: [u8; 16] = [0; 16];

        let mut interfaces = Vec::new();

        loop {
            reader.read_exact(&mut header_buffer);
            let header = unsafe { NLMsgHeader::from_slice(&header_buffer) };
            if header.msg_type == MessageType::NLMSG_DONE {
                break;
            }
            
            unsafe {
                let body = NewInterfaceCommand::read_after_header(header, &mut reader).unwrap();
                interfaces.push(body.into());
            }
        }

        return interfaces;
    }
}
