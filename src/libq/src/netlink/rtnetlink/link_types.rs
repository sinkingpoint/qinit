use io::{read_u16, read_u32, read_u64, read_u8, write_u16, write_u32, BufferReader, Endianness};
use netlink::error::NetLinkError;
use num_enum::TryFromPrimitive;
use super::routing_attrs::read_new_attr;

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::fmt;
use std::io::{Read, Write};

#[derive(Debug, Default, Clone, Copy)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    fn to_string(&self) -> String {
        return format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        );
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
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
            .filter(|(k, _)| self.contains(*k))
            .map(|(_, v)| v)
            .collect::<Vec<&str>>()
            .join(",");
    }
}

impl fmt::Display for InterfaceFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
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

impl InterfaceType {
    pub fn to_string(&self) -> String {
        return match self {
            InterfaceType::ARPHRD_NETROM => "netrom",
            InterfaceType::ARPHRD_ETHER => "ether",
            InterfaceType::ARPHRD_EETHER => "eether",
            InterfaceType::ARPHRD_AX25 => "ax25",
            InterfaceType::ARPHRD_PRONET => "pronet",
            InterfaceType::ARPHRD_CHAOS => "chaos",
            InterfaceType::ARPHRD_IEEE802 => "ieee802",
            InterfaceType::ARPHRD_ARCNET => "arcnet",
            InterfaceType::ARPHRD_APPLETLK => "appletk",
            InterfaceType::ARPHRD_DLCI => "dlci",
            InterfaceType::ARPHRD_ATM => "atm",
            InterfaceType::ARPHRD_METRICOM => "metricom",
            InterfaceType::ARPHRD_IEEE1394 => "ieee1394",
            InterfaceType::ARPHRD_EUI64 => "eui64",
            InterfaceType::ARPHRD_INFINIBAND => "infiniband",
            InterfaceType::ARPHRD_SLIP => "slip",
            InterfaceType::ARPHRD_CSLIP => "cslip",
            InterfaceType::ARPHRD_SLIP6 => "slip6",
            InterfaceType::ARPHRD_CSLIP6 => "cslip6",
            InterfaceType::ARPHRD_RSRVD => "rsrvd",
            InterfaceType::ARPHRD_ADAPT => "adapt",
            InterfaceType::ARPHRD_ROSE => "rose",
            InterfaceType::ARPHRD_X25 => "x25",
            InterfaceType::ARPHRD_HWX25 => "hwx25",
            InterfaceType::ARPHRD_PPP => "ppp",
            InterfaceType::ARPHRD_CISCO => "cisco",
            InterfaceType::ARPHRD_LAPB => "lapb",
            InterfaceType::ARPHRD_DDCMP => "ddcmp",
            InterfaceType::ARPHRD_RAWHDLC => "rawhdlc",
            InterfaceType::ARPHRD_TUNNEL => "tunnel",
            InterfaceType::ARPHRD_TUNNEL6 => "tunnel6",
            InterfaceType::ARPHRD_FRAD => "frad",
            InterfaceType::ARPHRD_SKIP => "skip",
            InterfaceType::ARPHRD_LOOPBACK => "loopback",
            InterfaceType::ARPHRD_LOCALTLK => "localtlk",
            InterfaceType::ARPHRD_FDDI => "fddi",
            InterfaceType::ARPHRD_BIF => "bif",
            InterfaceType::ARPHRD_SIT => "sit",
            InterfaceType::ARPHRD_IPDDP => "ipddp",
            InterfaceType::ARPHRD_IPGRE => "ipgre",
            InterfaceType::ARPHRD_PIMREG => "pimreg",
            InterfaceType::ARPHRD_HIPPI => "hippi",
            InterfaceType::ARPHRD_ASH => "ash",
            InterfaceType::ARPHRD_ECONET => "econet",
            InterfaceType::ARPHRD_IRDA => "irda",
            InterfaceType::ARPHRD_FCPP => "fccp",
            InterfaceType::ARPHRD_FCAL => "fcal",
            InterfaceType::ARPHRD_FCPL => "fcpl",
            InterfaceType::ARPHRD_FCFABRIC => "fcfabric",
            InterfaceType::ARPHRD_IEEE802_TR => "ieee802_tr",
            InterfaceType::ARPHRD_IEEE80211 => "ieee80211",
            InterfaceType::ARPHRD_IEEE80211_PRISM => "ieee80211_prism",
            InterfaceType::ARPHRD_IEEE80211_RADIOTAP => "ieee80211_radiotap",
            InterfaceType::ARPHRD_IEEE802154 => "ieee802154",
            InterfaceType::ARPHRD_VOID => "void",
            InterfaceType::ARPHRD_NONE => "none",
        }
        .to_owned();
    }
}

impl fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

#[derive(Debug)]
pub struct Interface {
    pub interface_type: InterfaceType,   /* Device type */
    pub interface_index: u32,            /* Interface index */
    pub interface_flags: InterfaceFlags, /* Device flags  */
    pub change_mask: u32,
    pub rtattrs: InterfaceRoutingAttributes,
}

impl Interface {
    pub fn from_raw_messages(info_msg: InterfaceInfoMessage, rtattrs: InterfaceRoutingAttributes) -> Interface {
        return Interface {
            interface_type: info_msg.interface_type,
            interface_index: info_msg.interface_index,
            interface_flags: info_msg.interface_flags,
            change_mask: info_msg.change_mask,
            rtattrs: rtattrs,
        };
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

#[derive(TryFromPrimitive, Debug, Clone, Copy)]
#[repr(u8)]
pub enum OperationalState {
    Unknown,        // IF_OPER_UNKNOWN
    NotPresent,     // IF_OPER_NOTPRESENT
    Down,           // IF_OPER_DOWN
    LowerLayerDown, // IF_OPER_LOWERLAYERDOWN
    Testing,        // IF_OPER_TESTING
    Dormant,        // IF_OPER_DORMANT
    Up,             // IF_OPER_UP
}

impl OperationalState {
    fn to_string(&self) -> String {
        return match self {
            OperationalState::Unknown => "UNKNOWN",
            OperationalState::NotPresent => "NOTPRESENT",
            OperationalState::Down => "DOWN",
            OperationalState::LowerLayerDown => "LOWERLAYERDOWN",
            OperationalState::Testing => "TESTING",
            OperationalState::Dormant => "DORMANT",
            OperationalState::Up => "UP",
        }
        .to_owned();
    }
}

impl fmt::Display for OperationalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

#[derive(TryFromPrimitive, Debug, Clone, Copy)]
#[repr(u8)]
pub enum LinkMode {
    Default, // IF_LINK_MODE_DEFAULT
    Dormant, // IF_LINK_MODE_DORMANT
    Testing, // IF_LINK_MODE_TESTING
}

impl LinkMode {
    fn to_string(&self) -> String {
        return match self {
            LinkMode::Default => "DEFAULT",
            LinkMode::Dormant => "DORMANT",
            LinkMode::Testing => "TESTING",
        }
        .to_owned();
    }
}

impl fmt::Display for LinkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub struct BridgeID {
    id: u16,
    addr: MacAddress,
}

impl BridgeID {
    fn to_string(&self) -> String {
        return format!("{:02o}.{}", self.id, self.addr);
    }
}

impl From<[u8; 8]> for BridgeID {
    fn from(data: [u8; 8]) -> Self {
        return BridgeID {
            id: (data[1] as u16) << 8 | (data[0] as u16),
            addr: MacAddress(data[2..8].try_into().unwrap()),
        };
    }
}

impl TryFrom<&[u8]> for BridgeID {
    type Error = NetLinkError;
    fn try_from(data: &[u8]) -> Result<BridgeID, Self::Error> {
        let data: [u8; 8] = data.try_into()?;
        return Ok(BridgeID::from(data));
    }
}

impl fmt::Display for BridgeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}

#[derive(Debug, Clone, Default)]
pub struct BridgeLinkData {
    forward_delay: Option<u32>,
    hello_time: Option<u32>,
    max_age: Option<u32>,
    ageing_time: Option<u32>,
    stp_state: Option<u32>,
    priority: Option<u16>,
    vlan_filtering: Option<bool>,
    vlan_protocol: Option<u16>, // TODO: Make this an enum
    group_fwd_mask: Option<u16>,
    root_id: Option<BridgeID>,
    bridge_id: Option<BridgeID>,
    root_port: Option<u16>,
    root_path_cost: Option<u32>,
    topology_change: Option<bool>,
    topology_change_detected: Option<bool>,
    hello_timer: Option<u64>,
    tcn_timer: Option<u64>,
    topology_change_timer: Option<u64>,
    gc_timer: Option<u64>,
    group_address: Option<MacAddress>,
    fdb_flush: Option<bool>,
    multicast_router: Option<u8>,
    multicast_snooping: Option<u8>,
    multicast_query_use_ifaddr: Option<u8>,
    multicast_querier: Option<u8>,
    multicast_hash_elasticity: Option<u32>,
    multicast_hash_max: Option<u32>,
    multicast_last_member_count: Option<u32>,
    multicast_startup_query_count: Option<u32>,
    multicast_last_member_interval: Option<u64>,
    multicast_membership_interval: Option<u64>,
    multicast_querier_interval: Option<u64>,
    multicast_query_interval: Option<u64>,
    multicast_query_response_interval: Option<u64>,
    multicast_startup_query_interval: Option<u64>,
    nf_call_iptables: Option<u8>,
    nf_call_ip6tables: Option<u8>,
    nf_call_arptables: Option<u8>,
    vlan_default_pvid: Option<u16>,
    //Pad
    vlan_stats_enabled: Option<bool>,
    multicast_stats_enabled: Option<bool>,
    multicast_igmp_version: Option<u8>,
    multicast_mld_version: Option<u8>,
    multicast_vlan_stats_per_port: Option<u8>,
    multicast_multi_boolopt: Option<u64>,
}

impl BridgeLinkData {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn read_new_attr(&mut self, data_reader: &mut BufferReader) -> Result<(), NetLinkError> {
        let (attr_type, data_buffer) = read_new_attr(data_reader)?;
        let mut data_reader = BufferReader::new(&data_buffer);

        match attr_type {
            1 => self.forward_delay = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_FORWARD_DELAY
            2 => self.hello_time = Some(read_u32(&mut data_reader, &Endianness::Little)?),    // IFLA_BR_HELLO_TIME
            3 => self.max_age = Some(read_u32(&mut data_reader, &Endianness::Little)?),       // IFLA_BR_MAX_AGE
            4 => self.ageing_time = Some(read_u32(&mut data_reader, &Endianness::Little)?),   // IFLA_BR_AGEING_TIME
            5 => self.stp_state = Some(read_u32(&mut data_reader, &Endianness::Little)?),     // IFLA_BR_STP_STATE
            6 => self.priority = Some(read_u16(&mut data_reader, &Endianness::Little)?),      // IFLA_BR_PRIORITY
            7 => self.vlan_filtering = Some(read_u8(&mut data_reader)? != 0),                 // IFLA_BR_VLAN_FILTERING
            8 => self.vlan_protocol = Some(read_u16(&mut data_reader, &Endianness::Little)?), // IFLA_BR_VLAN_PROTOCOL
            9 => self.group_fwd_mask = Some(read_u16(&mut data_reader, &Endianness::Little)?), // IFLA_BR_GROUP_FWD_MASK
            10 => self.root_id = Some(data_buffer[..].try_into()?),                           // IFLA_BR_ROOT_ID
            11 => self.bridge_id = Some(data_buffer[..].try_into()?),                         // IFLA_BR_BRIDGE_ID
            12 => self.root_port = Some(read_u16(&mut data_reader, &Endianness::Little)?),    // IFLA_BR_ROOT_PORT
            13 => self.root_path_cost = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_ROOT_PATH_COST
            14 => self.topology_change = Some(read_u8(&mut data_reader)? != 0),               // IFLA_BR_TOPOLOGY_CHANGE
            15 => self.topology_change_detected = Some(read_u8(&mut data_reader)? != 0),      // IFLA_BR_TOPOLOGY_CHANGE_DETECTED
            16 => self.hello_timer = Some(read_u64(&mut data_reader, &Endianness::Little)?),  // IFLA_BR_HELLO_TIMER
            17 => self.tcn_timer = Some(read_u64(&mut data_reader, &Endianness::Little)?),    // IFLA_BR_TCN_TIMER
            18 => self.topology_change_timer = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_TOPOLOGY_CHANGE_TIMER
            19 => self.gc_timer = Some(read_u64(&mut data_reader, &Endianness::Little)?),     // IFLA_BR_GC_TIMER
            20 => self.group_address = Some(MacAddress(data_buffer[..].try_into()?)),         // IFLA_BR_GROUP_ADDR
            21 => self.fdb_flush = Some(read_u8(&mut data_reader)? != 0),                     // IFLA_BR_FDB_FLUSH
            22 => self.multicast_router = Some(read_u8(&mut data_reader)?),                   // IFLA_BR_MCAST_ROUTER
            23 => self.multicast_snooping = Some(read_u8(&mut data_reader)?),                 // IFLA_BR_MCAST_SNOOPING
            24 => self.multicast_query_use_ifaddr = Some(read_u8(&mut data_reader)?),         // IFLA_BR_MCAST_QUERY_USE_IFADDR
            25 => self.multicast_querier = Some(read_u8(&mut data_reader)?),                  // IFLA_BR_MCAST_QUERIER
            26 => self.multicast_hash_elasticity = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_HASH_ELASTICITY
            27 => self.multicast_hash_max = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_HASH_MAX
            28 => self.multicast_last_member_count = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_LAST_MEMBER_CNT
            29 => self.multicast_startup_query_count = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_STARTUP_QUERY_CNT
            30 => self.multicast_last_member_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_LAST_MEMBER_INTVL
            31 => self.multicast_membership_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_MEMBERSHIP_INTVL
            32 => self.multicast_querier_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_QUERIER_INTVL
            33 => self.multicast_query_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?),   // IFLA_BR_MCAST_QUERY_INTVL
            34 => self.multicast_query_response_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_QUERY_RESPONSE_INTVL
            35 => self.multicast_startup_query_interval = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MCAST_STARTUP_QUERY_INTVL
            36 => self.nf_call_iptables = Some(read_u8(&mut data_reader)?), // IFLA_BR_NF_CALL_IPTABLES
            37 => self.nf_call_ip6tables = Some(read_u8(&mut data_reader)?), // IFLA_BR_NF_CALL_IP6TABLES
            38 => self.nf_call_arptables = Some(read_u8(&mut data_reader)?), // IFLA_BR_NF_CALL_ARPTABLES,
            39 => self.vlan_default_pvid = Some(read_u16(&mut data_reader, &Endianness::Little)?), // IFLA_BR_VLAN_DEFAULT_PVID
            // Pad
            41 => self.vlan_stats_enabled = Some(read_u8(&mut data_reader)? != 0), // IFLA_BR_VLAN_STATS_ENABLED
            42 => self.multicast_stats_enabled = Some(read_u8(&mut data_reader)? != 0), // IFLA_BR_MCAST_STATS_ENABLED
            43 => self.multicast_igmp_version = Some(read_u8(&mut data_reader)?),  // IFLA_BR_MCAST_IGMP_VERSION
            44 => self.multicast_mld_version = Some(read_u8(&mut data_reader)?),   // IFLA_BR_MCAST_MLD_VERSION
            45 => self.multicast_vlan_stats_per_port = Some(read_u8(&mut data_reader)?), // IFLA_BR_VLAN_STATS_PER_PORT
            46 => self.multicast_multi_boolopt = Some(read_u64(&mut data_reader, &Endianness::Little)?), // IFLA_BR_MULTI_BOOLOPT
            _ => {
                return Err(NetLinkError::InvalidEnumPrimitive(attr_type as u64));
            }
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, Default)]
pub struct TunnelLinkData {
    // TODO: Figure out whether these are actually u32s
    owner: Option<u32>,
    group: Option<u32>,
    tun_type: Option<u8>, // TODO: Make this an Enum
    pi: Option<bool>,
    vnet_hdr: Option<bool>,
    persist: Option<bool>,
    multi_queue: Option<bool>,
    num_queues: Option<u32>,
    num_disabled_queues: Option<u32>,
}

impl TunnelLinkData {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn read_new_attr(&mut self, data_reader: &mut BufferReader) -> Result<(), NetLinkError> {
        let (attr_type, data_buffer) = read_new_attr(data_reader)?;
        let mut data_reader = BufferReader::new(&data_buffer);

        match attr_type {
            0 => {}                                                                                 // IFLA_TUN_UNSPEC
            1 => self.owner = Some(read_u32(&mut data_reader, &Endianness::Little)?),               // IFLA_TUN_OWNER
            2 => self.group = Some(read_u32(&mut data_reader, &Endianness::Little)?),               // IFLA_TUN_GROUP
            3 => self.tun_type = Some(read_u8(&mut data_reader)?),                                  // IFLA_TUN_TYPE
            4 => self.pi = Some(read_u8(&mut data_reader)? != 0),                                   // IFLA_TUN_PI
            5 => self.vnet_hdr = Some(read_u8(&mut data_reader)? != 0),                             // IFLA_TUN_VNET_HDR
            6 => self.persist = Some(read_u8(&mut data_reader)? != 0),                              // IFLA_TUN_PERSIST
            7 => self.multi_queue = Some(read_u8(&mut data_reader)? != 0),                          // IFLA_TUN_MULTI_QUEUE
            8 => self.num_queues = Some(read_u32(&mut data_reader, &Endianness::Little)?),          // IFLA_TUN_NUM_QUEUES
            9 => self.num_disabled_queues = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_TUN_NUM_DISABLED_QUEUES
            _ => {
                return Err(NetLinkError::InvalidEnumPrimitive(attr_type as u64));
            }
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, Default)]
pub struct LinkInfo {
    pub kind: Option<String>,
    pub slave_kind: Option<String>,

    pub bridge_link_data: Option<BridgeLinkData>,
    pub bridge_slave_data: Option<BridgeLinkData>,

    pub tunnel_link_data: Option<TunnelLinkData>,
    pub tunnel_slave_data: Option<TunnelLinkData>,
}

impl LinkInfo {
    pub fn new() -> LinkInfo {
        return Self::default();
    }

    pub fn read_new_attr(&mut self, data_reader: &mut BufferReader) -> Result<(), NetLinkError> {
        let (attr_type, data_buffer) = read_new_attr(data_reader)?;
        let mut data_reader = BufferReader::new(&data_buffer);
        match attr_type {
            libc::IFLA_INFO_UNSPEC => {}
            libc::IFLA_INFO_KIND => self.kind = Some(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
            libc::IFLA_INFO_SLAVE_KIND => self.slave_kind = Some(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
            libc::IFLA_INFO_XSTATS => {} // TODO
            libc::IFLA_INFO_DATA => match self.kind.as_deref() {
                Some("bridge") => {
                    if self.bridge_link_data.is_none() {
                        self.bridge_link_data = Some(BridgeLinkData::new());
                    }

                    if let Some(link_data) = self.bridge_link_data.as_mut() {
                        while data_reader.has_more() {
                            link_data.read_new_attr(&mut data_reader)?;
                        }
                    }
                }
                Some("tun") => {
                    if self.tunnel_link_data.is_none() {
                        self.tunnel_link_data = Some(TunnelLinkData::new());
                    }

                    if let Some(link_data) = self.tunnel_link_data.as_mut() {
                        while data_reader.has_more() {
                            link_data.read_new_attr(&mut data_reader)?;
                        }
                    }
                }
                Some(kind) => {
                    println!("Unhandled link type: {}", kind);
                }
                _ => {}
            },
            libc::IFLA_INFO_SLAVE_DATA => {} // TODO: Parse this
            _ => {
                println!("{} {:?}", attr_type, data_buffer);
            }
        }

        return Ok(());
    }
}

#[derive(Debug, Default, Clone)]
pub struct InterfaceRoutingAttributes {
    pub address: Option<MacAddress>,
    pub broadcast: Option<MacAddress>,
    pub interface_name: Option<String>,
    pub mtu: Option<u32>,
    pub link: Option<u32>,
    pub qdisc: Option<String>,
    pub stats: Option<LinkStats>,
    pub master_id: Option<u32>,
    pub txqueue_len: Option<u32>,
    pub oper_state: Option<OperationalState>,
    pub link_mode: Option<LinkMode>,
    pub stats64: Option<LinkStats64>,
    pub group: Option<u32>,
    pub promiscuity: Option<u32>,
    pub num_tx_queues: Option<u32>,
    pub num_rx_queues: Option<u32>,
    pub carrier: Option<bool>,
    pub carrier_changes: Option<u32>,
    pub net_ns_id: Option<u32>,
    pub proto_down: Option<bool>,
    pub gso_max_segs: Option<u32>,
    pub gso_max_size: Option<u32>,
    pub xdp: Option<u64>,
    pub carrier_up_count: Option<u32>,
    pub carrier_down_count: Option<u32>,
    pub min_mtu: Option<u32>,
    pub max_mtu: Option<u32>,

    pub link_info: Option<LinkInfo>,
    pub perm_address: Option<MacAddress>,

    pub unknowns: Vec<(u16, Vec<u8>)>,
}

impl InterfaceRoutingAttributes {
    pub fn new() -> InterfaceRoutingAttributes {
        return Self::default();
    }

    pub fn read_new_attr<T: Read>(&mut self, reader: &mut T) -> Result<(), NetLinkError> {
        let (attr_type, data_buffer) = read_new_attr(reader)?;
        let mut data_reader = BufferReader::new(&data_buffer);
        match attr_type {
            libc::IFLA_UNSPEC => {}
            libc::IFLA_ADDRESS => self.address = Some(MacAddress(data_buffer[..].try_into()?)),
            libc::IFLA_BROADCAST => self.broadcast = Some(MacAddress(data_buffer[..].try_into()?)),
            libc::IFLA_IFNAME => self.interface_name = Some(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
            libc::IFLA_MTU => self.mtu = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_LINK => self.link = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_QDISC => self.qdisc = Some(CStr::from_bytes_with_nul(&data_buffer)?.to_str()?.to_owned()),
            libc::IFLA_STATS => self.stats = Some(LinkStats::read(&mut data_reader)?),
            libc::IFLA_MASTER => self.master_id = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_TXQLEN => self.txqueue_len = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_OPERSTATE => self.oper_state = Some(read_u8(&mut data_reader)?.try_into()?),
            libc::IFLA_LINKMODE => self.link_mode = Some(read_u8(&mut data_reader)?.try_into()?),
            libc::IFLA_MAP => {} // Explicitly Drop these ones for now... Handling them is more difficult
            libc::IFLA_LINKINFO => {
                if self.link_info.is_none() {
                    self.link_info = Some(LinkInfo::new());
                }

                if let Some(link_info) = self.link_info.as_mut() {
                    while data_reader.has_more() {
                        link_info.read_new_attr(&mut data_reader)?;
                    }
                }
            }
            libc::IFLA_STATS64 => self.stats64 = Some(LinkStats64::read(&mut data_reader)?),
            libc::IFLA_AF_SPEC => {} // TODO: Parse this (VLAN Info)
            libc::IFLA_GROUP => self.group = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_PROMISCUITY => self.promiscuity = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_NUM_TX_QUEUES => self.num_tx_queues = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_NUM_RX_QUEUES => self.num_rx_queues = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_CARRIER => self.carrier = Some(read_u8(&mut data_reader)? != 0),
            libc::IFLA_CARRIER_CHANGES => self.carrier_changes = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_LINK_NETNSID => self.net_ns_id = Some(read_u32(&mut data_reader, &Endianness::Little)?),
            libc::IFLA_PROTO_DOWN => self.proto_down = Some(read_u8(&mut data_reader)? != 0),
            40 => self.gso_max_segs = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_GSO_MAX_SEGS
            41 => self.gso_max_size = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_GSO_MAX_SIZE
            43 => self.xdp = Some(read_u64(&mut data_reader, &Endianness::Little)?),          // IFLA_XDP
            47 => self.carrier_up_count = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_CARRIER_UP_COUNT
            48 => self.carrier_down_count = Some(read_u32(&mut data_reader, &Endianness::Little)?), // IFLA_CARRIER_DOWN_COUNT
            50 => self.min_mtu = Some(read_u32(&mut data_reader, &Endianness::Little)?),      // IFLA_MIN_MTU
            51 => self.max_mtu = Some(read_u32(&mut data_reader, &Endianness::Little)?),      // IFLA_MAX_MTU
            54 => self.perm_address = Some(MacAddress(data_buffer[..].try_into()?)),          // IFLA_PERM_ADDR
            _ => {
                self.unknowns.push((attr_type, data_buffer));
            }
        };

        return Ok(());
    }
}
