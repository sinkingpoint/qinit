use io::{Writable, Readable, write_u8, write_u32, Endianness, read_u8, read_u32, BufferReader};
use std::io::{self, Write, Read};
use super::super::error::NetLinkError;
use num_enum::TryFromPrimitive;
use std::convert::TryInto;
use std::ffi::CStr;
use super::routing_attrs::read_new_attr;

#[repr(u8)]
#[derive(Clone, Copy, Debug, TryFromPrimitive)]
enum AddressType {
    IPv4 = 2, // AF_INET
    IPv6 = 10 // AF_INET6
}

libc_bitflags! {
    #[allow(non_camel_case_types, dead_code)]
    pub struct AddressFlags : u32 {
        IFA_F_SECONDARY;
        IFA_F_NODAD;
        IFA_F_OPTIMISTIC;
        IFA_F_DADFAILED;
        IFA_F_HOMEADDRESS;
        IFA_F_DEPRECATED;
        IFA_F_TENTATIVE;
        IFA_F_PERMANENT;
        IFA_F_MANAGETEMPADDR;
        IFA_F_NOPREFIXROUTE;
        IFA_F_MCAUTOJOIN;
        IFA_F_STABLE_PRIVACY;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InterfaceAddrMessage {
    addr_type: AddressType,
    prefix_len: u8,
    flags: AddressFlags,
    scope: u8, // TODO: Make this an enum
    interface_index: u32,
}

impl Writable for InterfaceAddrMessage {
    type Error = NetLinkError;
    fn write<T: Write>(&self, writer: &mut T) -> Result<(), NetLinkError> {
        write_u8(writer, self.addr_type as u8)?;
        write_u8(writer, self.prefix_len as u8)?;
        write_u8(writer, (self.flags.bits() & 0xFF) as u8)?;
        write_u8(writer, self.scope)?;
        write_u32(writer, self.interface_index, &Endianness::Little)?;

        return Ok(());
    }
}

impl Readable for InterfaceAddrMessage {
    type Error = NetLinkError;
    fn read<T: Read>(reader: &mut T) -> Result<Self, Self::Error> {
        return Ok(InterfaceAddrMessage {
            addr_type: read_u8(reader)?.try_into()?,
            prefix_len: read_u8(reader)?,
            flags: match AddressFlags::from_bits(read_u8(reader)? as u32) {
                Some(flags) => flags,
                None => {
                    return Err(NetLinkError::InvalidEnumPrimitive(0));
                }
            },
            scope: read_u8(reader)?,
            interface_index: read_u32(reader, &Endianness::Little)?
        });
    }
}

impl InterfaceAddrMessage {
    pub fn size() -> u32 {
        return 8;
    }

    pub fn empty() -> Self {
        return InterfaceAddrMessage {
            addr_type: AddressType::IPv4,
            prefix_len: 0,
            flags: AddressFlags::empty(),
            scope: 0,
            interface_index: 10
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AddressBytes {
    IPv4([u8;4]),
    IPv6([u8;8]),
}

impl AddressBytes {
    fn new(buffer: Vec<u8>) -> Result<AddressBytes, NetLinkError> {
        if buffer.len() == 4 {
            return Ok(AddressBytes::IPv4(buffer[..].try_into()?));
        }
        else if buffer.len() == 6 {
            return Ok(AddressBytes::IPv6(buffer[..].try_into()?));
        }
        else {
            return Err(NetLinkError::UnknownRoutingAttribute(buffer.len() as u16));
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct CacheInfo {
    preferred: u32,
    valid: u32,
    cstamp: u32,
    tstamp: u32,
}

impl Readable for CacheInfo {
    type Error = NetLinkError;
    fn read<T: Read>(reader: &mut T) -> Result<Self, Self::Error> {
        return Ok(CacheInfo {
            preferred: read_u32(reader, &Endianness::Little)?,
            valid: read_u32(reader, &Endianness::Little)?,
            cstamp: read_u32(reader, &Endianness::Little)?,
            tstamp: read_u32(reader, &Endianness::Little)?,
        });
    }
}

#[derive(Default, Debug, Clone)]
pub struct AddressRoutingAttributes {
    address: Option<AddressBytes>,
    local: Option<AddressBytes>,
    label: Option<String>,
    broadcast: Option<AddressBytes>,
    anycast: Option<AddressBytes>,
    cache_info: Option<CacheInfo>,
    muiticast: Option<AddressBytes>,
    flags: Option<AddressFlags>,
    unknowns: Vec<(u16, Vec<u8>)>,
}

impl AddressRoutingAttributes {
    pub fn new() -> AddressRoutingAttributes {
        return Self::default();
    }

    pub fn read_new_attr<T: Read>(&mut self, reader: &mut T) -> Result<(), NetLinkError> {
        let (attr_type, data_buffer) = read_new_attr(reader)?;
        let mut data_reader = BufferReader::new(&data_buffer);
        match attr_type {
            1 => self.address = Some(AddressBytes::new(data_buffer)?), // IFA_ADDRESS
            2 => self.local = Some(AddressBytes::new(data_buffer)?), // IFA_LOCAL
            3 => self.label = Some(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()), // IFA_LABEL
            4 => self.broadcast = Some(AddressBytes::new(data_buffer)?), // IFA_BROADCAST
            5 => self.anycast = Some(AddressBytes::new(data_buffer)?), // IFA_ANYCAST
            6 => self.cache_info = Some(CacheInfo::read(&mut data_reader)?), // IFA_CACHE_INFO
            7 => self.muiticast = Some(AddressBytes::new(data_buffer)?), // IFA_MULTICAST
            8 => self.flags = match AddressFlags::from_bits(read_u32(&mut data_reader, &Endianness::Little)?) { // IFA_FLAGS
                Some(flags) => Some(flags),
                None => {
                    return Err(NetLinkError::InvalidEnumPrimitive(0));
                }
            },
            _ => {
                return Err(NetLinkError::UnknownRoutingAttribute(attr_type));
            }
        };

        return Ok(());
    }
}

#[derive(Clone, Debug)]
pub struct Address {
    addr_type: AddressType,
    prefix_len: u8,
    flags: AddressFlags,
    scope: u8, // TODO: Make this an enum
    interface_index: u32,
    rtattrs: AddressRoutingAttributes
}

impl Address {
    pub fn from_raw_messages(info_msg: InterfaceAddrMessage, rtattrs: AddressRoutingAttributes) -> Address {
        return Address {
            addr_type: info_msg.addr_type,
            prefix_len: info_msg.prefix_len,
            flags: info_msg.flags,
            scope: info_msg.scope,
            interface_index: info_msg.interface_index,
            rtattrs: rtattrs
        }
    }
}