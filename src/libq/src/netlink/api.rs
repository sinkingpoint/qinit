use io::{read_u16, read_u32, write_u16, write_u32, BufferReader, Endianness};
use nix::unistd::getpid;
use num_enum::TryFromPrimitive;

use super::error::NetLinkError;

use std::convert::TryFrom;
use std::io::{Read, Write};

libc_bitflags! {
    #[allow(non_camel_case_types, dead_code)]
    pub struct NetLinkMessageFlags : u16 {
        NLM_F_REQUEST as u16;
        NLM_F_MULTI as u16;
        NLM_F_ACK as u16;
        NLM_F_ECHO as u16;
        NLM_F_ROOT as u16;
        NLM_F_MATCH as u16;
        NLM_F_ATOMIC as u16;
        NLM_F_REPLACE as u16;
        NLM_F_EXCL as u16;
        NLM_F_CREATE as u16;
        NLM_F_APPEND as u16;
    }
}

libc_enum! {
    #[allow(non_camel_case_types, dead_code)]
    #[derive(TryFromPrimitive)]
    #[repr(u16)]
    pub enum MessageType {
        NLMSG_NOOP as u16,
        NLMSG_ERROR as u16,
        NLMSG_DONE as u16,
        NLMSG_OVERRUN as u16,
        RTM_NEWLINK as u16,
        RTM_DELLINK as u16,
        RTM_GETLINK as u16,
        RTM_SETLINK as u16,
        RTM_NEWADDR as u16,
        RTM_DELADDR as u16,
        RTM_GETADDR as u16,
        RTM_NEWROUTE as u16,
        RTM_DELROUTE as u16,
        RTM_GETROUTE as u16,
        RTM_NEWNEIGH as u16,
        RTM_DELNEIGH as u16,
        RTM_GETNEIGH as u16,
        RTM_NEWRULE as u16,
        RTM_DELRULE as u16,
        RTM_GETRULE as u16,
        RTM_NEWQDISC as u16,
        RTM_DELQDISC as u16,
        RTM_GETQDISC as u16,
        RTM_NEWTCLASS as u16,
        RTM_DELTCLASS as u16,
        RTM_GETTCLASS as u16,
        RTM_NEWTFILTER as u16,
        RTM_DELTFILTER as u16,
        RTM_GETTFILTER as u16,
        RTM_NEWACTION as u16,
        RTM_DELACTION as u16,
        RTM_GETACTION as u16,
        RTM_NEWPREFIX as u16,
        RTM_GETMULTICAST as u16,
        RTM_GETANYCAST as u16,
        RTM_NEWNEIGHTBL as u16,
        RTM_GETNEIGHTBL as u16,
        RTM_SETNEIGHTBL as u16,
        RTM_NEWNDUSEROPT as u16,
        RTM_NEWADDRLABEL as u16,
        RTM_DELADDRLABEL as u16,
        RTM_GETADDRLABEL as u16,
        RTM_GETDCB as u16,
        RTM_SETDCB as u16,
        RTM_NEWNETCONF as u16,
        RTM_GETNETCONF as u16,
        RTM_NEWMDB as u16,
        RTM_DELMDB as u16,
        RTM_GETMDB as u16,
        RTM_NEWNSID as u16,
        RTM_DELNSID as u16,
        RTM_GETNSID as u16,
    }
}

#[derive(Debug)]
pub struct NetLinkMessageHeader {
    pub length: u32,                /* Length of message including header */
    pub msg_type: MessageType,      /* Type of message content */
    pub flags: NetLinkMessageFlags, /* Additional flags */
    pub sequence_number: u32,       /* Sequence number */
    pub pid: u32,                   /* Sending process PID */
}

impl NetLinkMessageHeader {
    /// Gets the size of a NetLinkMessageHeader (16 bytes)
    pub fn size() -> u32 {
        return 16;
    }

    pub fn new(msg_type: MessageType, sequence_number: u32, length: u32, flags: NetLinkMessageFlags) -> NetLinkMessageHeader {
        return NetLinkMessageHeader {
            length: length,
            msg_type: msg_type,
            flags: flags,
            sequence_number: sequence_number,
            pid: getpid().as_raw() as u32,
        };
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), NetLinkError> {
        write_u32(writer, self.length, &Endianness::Little)?;
        write_u16(writer, self.msg_type as u16, &Endianness::Little)?;
        write_u16(writer, self.flags.bits(), &Endianness::Little)?;
        write_u32(writer, self.sequence_number, &Endianness::Little)?;
        write_u32(writer, self.pid, &Endianness::Little)?;

        return Ok(());
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<NetLinkMessageHeader, NetLinkError> {
        let endianness = &Endianness::Little;
        let mut buffer = vec![0; Self::size() as usize];
        reader.read(&mut buffer);
        let mut reader = BufferReader::new(&buffer);
        return Ok(NetLinkMessageHeader {
            length: read_u32(&mut reader, endianness)?,
            msg_type: MessageType::try_from(read_u16(&mut reader, endianness)?)?,
            flags: NetLinkMessageFlags::from_bits(read_u16(&mut reader, endianness)?).unwrap_or(NetLinkMessageFlags::empty()),
            sequence_number: read_u32(&mut reader, endianness)?,
            pid: read_u32(&mut reader, endianness)?,
        });
    }
}
