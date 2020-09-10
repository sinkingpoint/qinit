use io::{read_u16, read_u32, read_u64, read_u8, write_u16, write_u32, write_u8, BufferReader, Endianness};
use netlink::error::NetLinkError;
use num_enum::TryFromPrimitive;

use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CStr;
use std::fmt;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct MacAddress(pub [u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        );
    }
}

#[derive(Debug)]
pub struct IPv4Addr(pub [u8; 4]);

impl fmt::Display for IPv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3]);
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

impl InterfaceFlags {
    pub fn to_string(&self) -> String {
        let str_map = {
            let mut map = HashMap::new();
            map.insert(InterfaceFlags::IFF_UP, "UP");
            map.insert(InterfaceFlags::IFF_BROADCAST, "BROADCAST");
            map.insert(InterfaceFlags::IFF_DEBUG, "DEBUG");
            map.insert(InterfaceFlags::IFF_POINTOPOINT, "P2P");
            map.insert(InterfaceFlags::IFF_RUNNING, "RUNNING");
            map.insert(InterfaceFlags::IFF_NOARP, "NOARP");
            map.insert(InterfaceFlags::IFF_PROMISC, "PROMISC");
            map.insert(InterfaceFlags::IFF_NOTRAILERS, "NOTRAILERS");
            map.insert(InterfaceFlags::IFF_ALLMULTI, "ALLMULTI");
            map.insert(InterfaceFlags::IFF_MASTER, "MASTER");
            map.insert(InterfaceFlags::IFF_SLAVE, "SLAVE");
            map.insert(InterfaceFlags::IFF_MULTICAST, "MULTICAST");
            map.insert(InterfaceFlags::IFF_PORTSEL, "PORTSET");
            map.insert(InterfaceFlags::IFF_AUTOMEDIA, "AUTOMEDIA");
            map.insert(InterfaceFlags::IFF_DYNAMIC, "DYNAMIC");
            map.insert(InterfaceFlags::IFF_LOWER_UP, "LOWER_UP");
            map.insert(InterfaceFlags::IFF_DORMANT, "DORMANT");
            map.insert(InterfaceFlags::IFF_ECHO, "ECHO");
            map
        };

        return str_map
            .into_iter()
            .filter(|(k, v)| self.contains(*k))
            .map(|(_, v)| v)
            .collect::<Vec<&str>>()
            .join(",");
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

#[derive(Debug)]
pub struct Interface {
    pub interface_type: InterfaceType,   /* Device type */
    pub interface_index: u32,            /* Interface index */
    pub interface_flags: InterfaceFlags, /* Device flags  */
    pub change_mask: u32,
    pub rattrs: Vec<RoutingAttribute>,
}

impl Interface {
    pub fn from_raw_messages(info_msg: InterfaceInfoMessage, rattrs: Vec<RoutingAttribute>) -> Interface {
        return Interface {
            interface_type: info_msg.interface_type,
            interface_index: info_msg.interface_index,
            interface_flags: info_msg.interface_flags,
            change_mask: info_msg.change_mask,
            rattrs: rattrs,
        };
    }

    pub fn get_name(&self) -> Option<&str> {
        for attr in self.rattrs.iter() {
            if let RoutingAttribute::InterfaceName(name) = attr {
                return Some(&name);
            }
        }

        return None;
    }

    pub fn get_mtu(&self) -> Option<u32> {
        for attr in self.rattrs.iter() {
            if let RoutingAttribute::Mtu(mtu) = attr {
                return Some(*mtu);
            }
        }

        return None;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InterfaceInfoMessage {
    pub interface_type: InterfaceType,   /* Device type */
    pub interface_index: u32,            /* Interface index */
    pub interface_flags: InterfaceFlags, /* Device flags  */
    pub change_mask: u32,
}

impl InterfaceInfoMessage {
    pub fn empty() -> InterfaceInfoMessage {
        return InterfaceInfoMessage {
            interface_type: InterfaceType::ARPHRD_NETROM,
            interface_index: 0,
            interface_flags: InterfaceFlags::empty(),
            change_mask: 0xffffffff,
        };
    }

    pub fn size() -> u32 {
        return 16;
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> Result<(), NetLinkError> {
        write_u16(writer, 0, &Endianness::Little)?;
        write_u16(writer, self.interface_type as u16, &Endianness::Little)?;
        write_u32(writer, self.interface_index, &Endianness::Little)?;
        write_u32(writer, self.interface_flags.bits(), &Endianness::Little)?;
        write_u32(writer, self.change_mask, &Endianness::Little)?;

        return Ok(());
    }

    pub fn read<T: Read>(reader: &mut T) -> Result<Self, NetLinkError> {
        let _family = read_u8(reader)?; // AF_UNSPEC (Should be 0)
        read_u8(reader)?;
        let if_type = read_u16(reader, &Endianness::Little)?;
        let if_type: InterfaceType = match if_type.try_into() {
            Ok(if_type) => if_type,
            Err(_) => {
                return Err(NetLinkError::InvalidEnumPrimitive(if_type as u64));
            }
        };

        let if_index = read_u32(reader, &Endianness::Little)?;
        let if_flags = read_u32(reader, &Endianness::Little)?;
        let if_flags = match InterfaceFlags::from_bits(if_flags) {
            Some(flags) => flags,
            None => {
                return Err(NetLinkError::InvalidEnumPrimitive(if_flags as u64));
            }
        };
        let change = read_u32(reader, &Endianness::Little)?;

        return Ok(InterfaceInfoMessage {
            interface_type: if_type,
            interface_index: if_index,
            interface_flags: if_flags,
            change_mask: change,
        });
    }
}

#[derive(Debug)]
pub struct LinkStats {
    rx_packets: u32, /* total packets received	*/
    u16tx_packets: u32,
    u16rx_bytes: u32,
    u16tx_bytes: u32,
    u16rx_errors: u32,
    u16tx_errors: u32,
    u16rx_dropped: u32,
    u16tx_dropped: u32,
    u16multicast: u32,
    u16collisions: u32,

    /* detailed rx_errors: */
    u16rx_length_errors: u32,
    u16rx_over_errors: u32,
    u16rx_crc_errors: u32,
    u16rx_frame_errors: u32,
    u16rx_fifo_errors: u32,
    u16rx_missed_errors: u32,

    /* detailed tx_errors */
    u16tx_aborted_errors: u32,
    u16tx_carrier_errors: u32,
    u16tx_fifo_errors: u32,
    u16tx_heartbeat_errors: u32,
    u16tx_window_errors: u32,

    /* for cslip etc */
    u16rx_compressed: u32,
    u16tx_compressed: u32,

    u16rx_nohandler: u32,
}

impl LinkStats {
    fn read<T: Read>(reader: &mut T) -> Result<LinkStats, NetLinkError> {
        let endianness = &Endianness::Little;
        return Ok(LinkStats {
            rx_packets: read_u32(reader, endianness)?,
            u16tx_packets: read_u32(reader, endianness)?,
            u16rx_bytes: read_u32(reader, endianness)?,
            u16tx_bytes: read_u32(reader, endianness)?,
            u16rx_errors: read_u32(reader, endianness)?,
            u16tx_errors: read_u32(reader, endianness)?,
            u16rx_dropped: read_u32(reader, endianness)?,
            u16tx_dropped: read_u32(reader, endianness)?,
            u16multicast: read_u32(reader, endianness)?,
            u16collisions: read_u32(reader, endianness)?,
            u16rx_length_errors: read_u32(reader, endianness)?,
            u16rx_over_errors: read_u32(reader, endianness)?,
            u16rx_crc_errors: read_u32(reader, endianness)?,
            u16rx_frame_errors: read_u32(reader, endianness)?,
            u16rx_fifo_errors: read_u32(reader, endianness)?,
            u16rx_missed_errors: read_u32(reader, endianness)?,
            u16tx_aborted_errors: read_u32(reader, endianness)?,
            u16tx_carrier_errors: read_u32(reader, endianness)?,
            u16tx_fifo_errors: read_u32(reader, endianness)?,
            u16tx_heartbeat_errors: read_u32(reader, endianness)?,
            u16tx_window_errors: read_u32(reader, endianness)?,
            u16rx_compressed: read_u32(reader, endianness)?,
            u16tx_compressed: read_u32(reader, endianness)?,
            u16rx_nohandler: read_u32(reader, endianness)?,
        });
    }
}

#[derive(Debug)]
pub struct LinkStats64 {
    rx_packets: u64, /* total packets received	*/
    u16tx_packets: u64,
    u16rx_bytes: u64,
    u16tx_bytes: u64,
    u16rx_errors: u64,
    u16tx_errors: u64,
    u16rx_dropped: u64,
    u16tx_dropped: u64,
    u16multicast: u64,
    u16collisions: u64,

    /* detailed rx_errors: */
    u16rx_length_errors: u64,
    u16rx_over_errors: u64,
    u16rx_crc_errors: u64,
    u16rx_frame_errors: u64,
    u16rx_fifo_errors: u64,
    u16rx_missed_errors: u64,

    /* detailed tx_errors */
    u16tx_aborted_errors: u64,
    u16tx_carrier_errors: u64,
    u16tx_fifo_errors: u64,
    u16tx_heartbeat_errors: u64,
    u16tx_window_errors: u64,

    /* for cslip etc */
    u16rx_compressed: u64,
    u16tx_compressed: u64,

    u16rx_nohandler: u64,
}

impl LinkStats64 {
    fn read<T: Read>(reader: &mut T) -> Result<LinkStats64, NetLinkError> {
        let endianness = &Endianness::Little;
        return Ok(LinkStats64 {
            rx_packets: read_u64(reader, endianness)?,
            u16tx_packets: read_u64(reader, endianness)?,
            u16rx_bytes: read_u64(reader, endianness)?,
            u16tx_bytes: read_u64(reader, endianness)?,
            u16rx_errors: read_u64(reader, endianness)?,
            u16tx_errors: read_u64(reader, endianness)?,
            u16rx_dropped: read_u64(reader, endianness)?,
            u16tx_dropped: read_u64(reader, endianness)?,
            u16multicast: read_u64(reader, endianness)?,
            u16collisions: read_u64(reader, endianness)?,
            u16rx_length_errors: read_u64(reader, endianness)?,
            u16rx_over_errors: read_u64(reader, endianness)?,
            u16rx_crc_errors: read_u64(reader, endianness)?,
            u16rx_frame_errors: read_u64(reader, endianness)?,
            u16rx_fifo_errors: read_u64(reader, endianness)?,
            u16rx_missed_errors: read_u64(reader, endianness)?,
            u16tx_aborted_errors: read_u64(reader, endianness)?,
            u16tx_carrier_errors: read_u64(reader, endianness)?,
            u16tx_fifo_errors: read_u64(reader, endianness)?,
            u16tx_heartbeat_errors: read_u64(reader, endianness)?,
            u16tx_window_errors: read_u64(reader, endianness)?,
            u16rx_compressed: read_u64(reader, endianness)?,
            u16tx_compressed: read_u64(reader, endianness)?,
            u16rx_nohandler: read_u64(reader, endianness)?,
        });
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RoutingAttribute {
    Unknown(u16, Vec<u8>),
    Address(MacAddress),
    Broadcast(MacAddress),
    InterfaceName(String),
    Mtu(u32),
    Link(u32),
    QDisc(String),
    Stats(LinkStats),
    Cost(Vec<u8>),
    Priority(Vec<u8>),
    Master(u32),
    Wireless(Vec<u8>),
    ProtInfo(Vec<u8>),
    TXQLen(u32),
    Map(Vec<u8>),
    Weight(u32),
    OperationalState(u8),
    LinkMode(u8),
    LinkInfo(Vec<u8>),
    NetNSPid(Vec<u8>),
    Alias(String),
    NumVfs(Vec<u8>),
    VfInfoList(Vec<u8>),
    Stats64(LinkStats64),
    VfPorts(Vec<u8>),
    PortSelf(Vec<u8>),
    AfSpec(Vec<u8>),
    Group(IPv4Addr),
    NetNsFd(Vec<u8>),
    ExtMask(Vec<u8>),
    Promiscuity(u32),
    NumTxQueues(u32),
    NumRxQueues(u32),
    Carrier(u8),
    PhysPortId(Vec<u8>),
    CarrierChanges(u32),
    PhysSwitchId(Vec<u8>),
    LinkNetNsId(Vec<u8>),
    PhysPortName(Vec<u8>),
    ProtoDown(u8),
    GsoMaxSegs(u32),
    GsoMaxSize(u32),
    Pad(Vec<u8>),
    Xdp(Vec<u8>),
    Event(Vec<u8>),
    NewNetNsId(Vec<u8>),
    IfNetNsId(Vec<u8>),
    CarrierUpCount(u32),
    CarrierDownCount(u32),
    NewIfIndex(Vec<u8>),
    MinMtu(u32),
    MaxMtu(u32),
    PropList(Vec<u8>),
    AltIfName(Vec<u8>),
    PermAddress(Vec<u8>),
}

impl RoutingAttribute {
    pub fn read<T: Read>(data: &mut T) -> Result<(RoutingAttribute, u32), NetLinkError> {
        let length: u16 = read_u16(data, &Endianness::Little)?;
        let attr_type: u16 = read_u16(data, &Endianness::Little)?;
        const ALIGN_TO: u16 = 4;
        let padding_length: u32 = (((length + ALIGN_TO - 1) & !(ALIGN_TO - 1)) - length) as u32;

        let mut data_buffer = vec![0; length as usize - 4];
        data.read_exact(&mut data_buffer)?;

        let mut data_reader = BufferReader::new(&data_buffer);

        let mut padding_buffer = vec![0; padding_length as usize];
        data.read_exact(&mut padding_buffer)?;

        return Ok((
            match attr_type {
                1 => RoutingAttribute::Address(MacAddress(data_buffer[..].try_into()?)),
                2 => RoutingAttribute::Broadcast(MacAddress(data_buffer[..].try_into()?)),
                3 => RoutingAttribute::InterfaceName(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
                4 => RoutingAttribute::Mtu(read_u32(&mut data_reader, &Endianness::Little)?),
                5 => RoutingAttribute::Link(read_u32(&mut data_reader, &Endianness::Little)?),
                6 => RoutingAttribute::QDisc(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
                7 => RoutingAttribute::Stats(LinkStats::read(&mut data_reader)?),
                8 => RoutingAttribute::Cost(data_buffer),
                9 => RoutingAttribute::Priority(data_buffer),
                10 => RoutingAttribute::Master(read_u32(&mut data_reader, &Endianness::Little)?),
                11 => RoutingAttribute::Wireless(data_buffer),
                12 => RoutingAttribute::ProtInfo(data_buffer),
                13 => RoutingAttribute::TXQLen(read_u32(&mut data_reader, &Endianness::Little)?),
                14 => RoutingAttribute::Map(data_buffer),
                15 => RoutingAttribute::Weight(read_u32(&mut data_reader, &Endianness::Little)?),
                16 => RoutingAttribute::OperationalState(data_buffer[0]),
                17 => RoutingAttribute::LinkMode(data_buffer[0]),
                18 => RoutingAttribute::LinkInfo(data_buffer),
                19 => RoutingAttribute::NetNSPid(data_buffer),
                20 => RoutingAttribute::Alias(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
                21 => RoutingAttribute::NumVfs(data_buffer),
                22 => RoutingAttribute::VfInfoList(data_buffer),
                23 => RoutingAttribute::Stats64(LinkStats64::read(&mut data_reader)?),
                24 => RoutingAttribute::VfPorts(data_buffer),
                25 => RoutingAttribute::PortSelf(data_buffer),
                26 => RoutingAttribute::AfSpec(data_buffer),
                27 => RoutingAttribute::Group(IPv4Addr(data_buffer[0..4].try_into()?)),
                28 => RoutingAttribute::NetNsFd(data_buffer),
                29 => RoutingAttribute::ExtMask(data_buffer),
                30 => RoutingAttribute::Promiscuity(read_u32(&mut data_reader, &Endianness::Little)?),
                31 => RoutingAttribute::NumTxQueues(read_u32(&mut data_reader, &Endianness::Little)?),
                32 => RoutingAttribute::NumRxQueues(read_u32(&mut data_reader, &Endianness::Little)?),
                33 => RoutingAttribute::Carrier(data_buffer[0]),
                34 => RoutingAttribute::PhysPortId(data_buffer),
                35 => RoutingAttribute::CarrierChanges(read_u32(&mut data_reader, &Endianness::Little)?),
                36 => RoutingAttribute::PhysSwitchId(data_buffer),
                37 => RoutingAttribute::LinkNetNsId(data_buffer),
                38 => RoutingAttribute::PhysPortName(data_buffer),
                39 => RoutingAttribute::ProtoDown(data_buffer[0]),
                40 => RoutingAttribute::GsoMaxSegs(read_u32(&mut data_reader, &Endianness::Little)?),
                41 => RoutingAttribute::GsoMaxSize(read_u32(&mut data_reader, &Endianness::Little)?),
                42 => RoutingAttribute::Pad(data_buffer),
                43 => RoutingAttribute::Xdp(data_buffer),
                44 => RoutingAttribute::Event(data_buffer),
                45 => RoutingAttribute::NewNetNsId(data_buffer),
                46 => RoutingAttribute::IfNetNsId(data_buffer),
                47 => RoutingAttribute::CarrierUpCount(read_u32(&mut data_reader, &Endianness::Little)?),
                48 => RoutingAttribute::CarrierDownCount(read_u32(&mut data_reader, &Endianness::Little)?),
                49 => RoutingAttribute::NewIfIndex(data_buffer),
                50 => RoutingAttribute::MinMtu(read_u32(&mut data_reader, &Endianness::Little)?),
                51 => RoutingAttribute::MaxMtu(read_u32(&mut data_reader, &Endianness::Little)?),
                52 => RoutingAttribute::PropList(data_buffer),
                53 => RoutingAttribute::AltIfName(data_buffer),
                54 => RoutingAttribute::PermAddress(data_buffer),
                _ => RoutingAttribute::Unknown(attr_type, data_buffer),
            },
            length as u32 + padding_length,
        ));
    }
}
