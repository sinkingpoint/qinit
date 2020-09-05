use io::{read_u16, read_u32, Endianness};
use nix::unistd::getpid;
use num_enum::TryFromPrimitive;

use super::error::NetLinkError;

use std::convert::TryFrom;
use std::io::Read;

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

libc_bitflags! {
    #[allow(non_camel_case_types, dead_code)]
    pub struct InterfaceFlags : u32 {
        IFF_UP as u32;
        IFF_BROADCAST as u32;
        IFF_DEBUG as u32;
        IFF_LOOPBACK as u32;
        IFF_POINTOPOINT as u32;
        IFF_RUNNING as u32;
        IFF_NOARP as u32;
        IFF_PROMISC as u32;
        IFF_NOTRAILERS as u32;
        IFF_ALLMULTI as u32;
        IFF_MASTER as u32;
        IFF_SLAVE as u32;
        IFF_MULTICAST as u32;
        IFF_PORTSEL as u32;
        IFF_AUTOMEDIA as u32;
        IFF_DYNAMIC as u32;
        IFF_LOWER_UP as u32;
        IFF_DORMANT as u32;
        IFF_ECHO as u32;
    }
}

libc_enum! {
    #[allow(non_camel_case_types, dead_code)]
    #[derive(TryFromPrimitive)]
    #[repr(u16)]
    pub enum InterfaceType {
        ARPHRD_NETROM as u16,
        ARPHRD_ETHER as u16,
        ARPHRD_EETHER as u16,
        ARPHRD_AX25 as u16,
        ARPHRD_PRONET as u16,
        ARPHRD_CHAOS as u16,
        ARPHRD_IEEE802 as u16,
        ARPHRD_ARCNET as u16,
        ARPHRD_APPLETLK as u16,
        ARPHRD_DLCI as u16,
        ARPHRD_ATM as u16,
        ARPHRD_METRICOM as u16,
        ARPHRD_IEEE1394 as u16,
        ARPHRD_EUI64 as u16,
        ARPHRD_INFINIBAND as u16,
        ARPHRD_SLIP as u16,
        ARPHRD_CSLIP as u16,
        ARPHRD_SLIP6 as u16,
        ARPHRD_CSLIP6 as u16,
        ARPHRD_RSRVD as u16,
        ARPHRD_ADAPT as u16,
        ARPHRD_ROSE as u16,
        ARPHRD_X25 as u16,
        ARPHRD_HWX25 as u16,
        ARPHRD_PPP as u16,
        ARPHRD_CISCO as u16,
        ARPHRD_LAPB as u16,
        ARPHRD_DDCMP as u16,
        ARPHRD_RAWHDLC as u16,
        ARPHRD_TUNNEL as u16,
        ARPHRD_TUNNEL6 as u16,
        ARPHRD_FRAD as u16,
        ARPHRD_SKIP as u16,
        ARPHRD_LOOPBACK as u16,
        ARPHRD_LOCALTLK as u16,
        ARPHRD_FDDI as u16,
        ARPHRD_BIF as u16,
        ARPHRD_SIT as u16,
        ARPHRD_IPDDP as u16,
        ARPHRD_IPGRE as u16,
        ARPHRD_PIMREG as u16,
        ARPHRD_HIPPI as u16,
        ARPHRD_ASH as u16,
        ARPHRD_ECONET as u16,
        ARPHRD_IRDA as u16,
        ARPHRD_FCPP as u16,
        ARPHRD_FCAL as u16,
        ARPHRD_FCPL as u16,
        ARPHRD_FCFABRIC as u16,
        ARPHRD_IEEE802_TR as u16,
        ARPHRD_IEEE80211 as u16,
        ARPHRD_IEEE80211_PRISM as u16,
        ARPHRD_IEEE80211_RADIOTAP as u16,
        ARPHRD_IEEE802154 as u16,
        ARPHRD_VOID as u16,
        ARPHRD_NONE as u16,
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
#[repr(C)]
pub struct NetLinkMessageHeader {
    pub length: u32,                /* Length of message including header */
    pub msg_type: MessageType,      /* Type of message content */
    pub flags: NetLinkMessageFlags, /* Additional flags */
    pub sequence_number: u32,       /* Sequence number */
    pub pid: u32,                   /* Sending process PID */
}

impl NetLinkMessageHeader {
    pub fn new(msg_type: MessageType, sequence_number: u32, length: u32, flags: NetLinkMessageFlags) -> NetLinkMessageHeader {
        return NetLinkMessageHeader {
            length: length,
            msg_type: msg_type,
            flags: flags,
            sequence_number: sequence_number,
            pid: getpid().as_raw() as u32,
        };
    }

    pub fn read<T: Read>(mut reader: &mut T) -> Result<NetLinkMessageHeader, NetLinkError> {
        let endianness = &Endianness::Little;
        return Ok(NetLinkMessageHeader {
            length: read_u32(&mut reader, endianness)?,
            msg_type: MessageType::try_from(read_u16(&mut reader, endianness)?)?,
            flags: NetLinkMessageFlags::from_bits(read_u16(&mut reader, endianness)?).unwrap_or(NetLinkMessageFlags::empty()),
            sequence_number: read_u32(&mut reader, endianness)?,
            pid: read_u32(&mut reader, endianness)?,
        });
    }
}
