use super::super::mem::{int_from_bytes_little_endian, long_from_bytes_little_endian, short_from_bytes_little_endian};
use nix::unistd::getpid;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::CStr;
use std::fmt;
use std::io::{self, Read};

libc_bitflags! {
    #[allow(non_camel_case_types, dead_code)]
    pub struct NLMsgFlags : u16 {
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
pub struct NLMsgHeader {
    pub length: u32,           /* Length of message including header */
    pub msg_type: MessageType, /* Type of message content */
    pub flags: NLMsgFlags,     /* Additional flags */
    pub sequence_number: u32,  /* Sequence number */
    pub pid: u32,              /* Sending process PID */
}

impl NLMsgHeader {
    pub fn new(msg_type: MessageType, sequence_number: u32, length: u32, flags: NLMsgFlags) -> NLMsgHeader {
        return NLMsgHeader {
            length: length,
            msg_type: msg_type,
            flags: flags,
            sequence_number: sequence_number,
            pid: getpid().as_raw() as u32,
        };
    }

    pub unsafe fn from_slice(data: &[u8; 16]) -> NLMsgHeader {
        let length = (data[3] as u32) << 24 | (data[2] as u32) << 16 | (data[1] as u32) << 8 | (data[0] as u32);
        let msg_type = (data[5] as u16) << 8 | data[4] as u16;
        let flags = (data[7] as u16) << 8 | data[6] as u16;
        let sequence_number = (data[11] as u32) << 24 | (data[10] as u32) << 16 | (data[9] as u32) << 8 | (data[8] as u32);
        let pid = (data[15] as u32) << 24 | (data[14] as u32) << 16 | (data[13] as u32) << 8 | (data[12] as u32);

        return NLMsgHeader {
            length: length,
            msg_type: MessageType::try_from(msg_type).unwrap(),
            flags: NLMsgFlags::from_bits(flags).unwrap(),
            sequence_number: sequence_number,
            pid: pid,
        };
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IFInfoMsg {
    _ifi_family: u8,
    _ifi_pad: u8,
    pub interface_type: InterfaceType,   /* Device type */
    pub interface_index: i32,            /* Interface index */
    pub interface_flags: InterfaceFlags, /* Device flags  */
    _ifi_change: u32,
}

impl IFInfoMsg {
    pub fn new(msg_type: InterfaceType, index: i32, flags: InterfaceFlags) -> IFInfoMsg {
        return IFInfoMsg {
            _ifi_family: 0,
            _ifi_pad: 0,
            interface_type: msg_type,
            interface_index: index,
            interface_flags: flags,
            _ifi_change: 0xFFFFFFFF,
        };
    }

    pub fn empty() -> IFInfoMsg {
        return IFInfoMsg::new(InterfaceType::ARPHRD_NONE, 0, InterfaceFlags::empty());
    }
}

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

pub type IPv4Addr = [u8; 4];

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

impl TryFrom<Vec<u8>> for LinkStats {
    type Error = &'static str;
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() != 96 {
            return Err("Length isn't 96 bytes");
        }

        return Ok(LinkStats {
            rx_packets: int_from_bytes_little_endian(vec[0], vec[1], vec[2], vec[3]),
            u16tx_packets: int_from_bytes_little_endian(vec[4], vec[5], vec[6], vec[7]),
            u16rx_bytes: int_from_bytes_little_endian(vec[8], vec[9], vec[10], vec[11]),
            u16tx_bytes: int_from_bytes_little_endian(vec[12], vec[13], vec[14], vec[15]),
            u16rx_errors: int_from_bytes_little_endian(vec[16], vec[17], vec[18], vec[19]),
            u16tx_errors: int_from_bytes_little_endian(vec[20], vec[21], vec[22], vec[23]),
            u16rx_dropped: int_from_bytes_little_endian(vec[24], vec[25], vec[26], vec[27]),
            u16tx_dropped: int_from_bytes_little_endian(vec[28], vec[29], vec[30], vec[31]),
            u16multicast: int_from_bytes_little_endian(vec[32], vec[33], vec[34], vec[35]),
            u16collisions: int_from_bytes_little_endian(vec[36], vec[37], vec[38], vec[39]),
            u16rx_length_errors: int_from_bytes_little_endian(vec[40], vec[41], vec[42], vec[43]),
            u16rx_over_errors: int_from_bytes_little_endian(vec[44], vec[45], vec[46], vec[47]),
            u16rx_crc_errors: int_from_bytes_little_endian(vec[48], vec[49], vec[50], vec[51]),
            u16rx_frame_errors: int_from_bytes_little_endian(vec[52], vec[53], vec[54], vec[55]),
            u16rx_fifo_errors: int_from_bytes_little_endian(vec[56], vec[57], vec[58], vec[59]),
            u16rx_missed_errors: int_from_bytes_little_endian(vec[60], vec[61], vec[62], vec[63]),
            u16tx_aborted_errors: int_from_bytes_little_endian(vec[64], vec[65], vec[66], vec[67]),
            u16tx_carrier_errors: int_from_bytes_little_endian(vec[68], vec[69], vec[70], vec[71]),
            u16tx_fifo_errors: int_from_bytes_little_endian(vec[72], vec[73], vec[74], vec[75]),
            u16tx_heartbeat_errors: int_from_bytes_little_endian(vec[76], vec[77], vec[78], vec[79]),
            u16tx_window_errors: int_from_bytes_little_endian(vec[80], vec[81], vec[82], vec[83]),
            u16rx_compressed: int_from_bytes_little_endian(vec[84], vec[85], vec[86], vec[87]),
            u16tx_compressed: int_from_bytes_little_endian(vec[88], vec[89], vec[90], vec[91]),
            u16rx_nohandler: int_from_bytes_little_endian(vec[92], vec[93], vec[94], vec[95]),
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

impl TryFrom<Vec<u8>> for LinkStats64 {
    type Error = &'static str;
    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() != 192 {
            return Err("Length isn't 96 bytes");
        }

        return Ok(LinkStats64 {
            rx_packets: long_from_bytes_little_endian(vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7]),
            u16tx_packets: long_from_bytes_little_endian(vec[8], vec[9], vec[10], vec[11], vec[12], vec[13], vec[14], vec[15]),
            u16rx_bytes: long_from_bytes_little_endian(vec[16], vec[17], vec[18], vec[19], vec[20], vec[21], vec[22], vec[23]),
            u16tx_bytes: long_from_bytes_little_endian(vec[24], vec[25], vec[26], vec[27], vec[28], vec[29], vec[30], vec[31]),
            u16rx_errors: long_from_bytes_little_endian(vec[32], vec[33], vec[34], vec[35], vec[36], vec[37], vec[38], vec[39]),
            u16tx_errors: long_from_bytes_little_endian(vec[40], vec[41], vec[42], vec[43], vec[44], vec[45], vec[46], vec[47]),
            u16rx_dropped: long_from_bytes_little_endian(vec[48], vec[49], vec[50], vec[51], vec[52], vec[53], vec[54], vec[55]),
            u16tx_dropped: long_from_bytes_little_endian(vec[56], vec[57], vec[58], vec[59], vec[60], vec[61], vec[62], vec[63]),
            u16multicast: long_from_bytes_little_endian(vec[64], vec[65], vec[66], vec[67], vec[68], vec[69], vec[70], vec[71]),
            u16collisions: long_from_bytes_little_endian(vec[72], vec[73], vec[74], vec[75], vec[76], vec[77], vec[78], vec[79]),
            u16rx_length_errors: long_from_bytes_little_endian(vec[80], vec[81], vec[82], vec[83], vec[84], vec[85], vec[86], vec[87]),
            u16rx_over_errors: long_from_bytes_little_endian(vec[88], vec[89], vec[90], vec[91], vec[92], vec[93], vec[94], vec[95]),
            u16rx_crc_errors: long_from_bytes_little_endian(vec[96], vec[97], vec[98], vec[99], vec[100], vec[101], vec[102], vec[103]),
            u16rx_frame_errors: long_from_bytes_little_endian(
                vec[104], vec[105], vec[106], vec[107], vec[108], vec[109], vec[110], vec[111],
            ),
            u16rx_fifo_errors: long_from_bytes_little_endian(
                vec[112], vec[113], vec[114], vec[115], vec[116], vec[117], vec[118], vec[119],
            ),
            u16rx_missed_errors: long_from_bytes_little_endian(
                vec[120], vec[121], vec[122], vec[123], vec[124], vec[125], vec[126], vec[127],
            ),
            u16tx_aborted_errors: long_from_bytes_little_endian(
                vec[128], vec[129], vec[130], vec[131], vec[132], vec[133], vec[134], vec[135],
            ),
            u16tx_carrier_errors: long_from_bytes_little_endian(
                vec[136], vec[137], vec[138], vec[139], vec[140], vec[141], vec[142], vec[143],
            ),
            u16tx_fifo_errors: long_from_bytes_little_endian(
                vec[144], vec[145], vec[146], vec[147], vec[148], vec[149], vec[150], vec[151],
            ),
            u16tx_heartbeat_errors: long_from_bytes_little_endian(
                vec[152], vec[153], vec[154], vec[155], vec[156], vec[157], vec[158], vec[159],
            ),
            u16tx_window_errors: long_from_bytes_little_endian(
                vec[160], vec[161], vec[162], vec[163], vec[164], vec[165], vec[166], vec[167],
            ),
            u16rx_compressed: long_from_bytes_little_endian(vec[168], vec[169], vec[170], vec[171], vec[172], vec[173], vec[174], vec[175]),
            u16tx_compressed: long_from_bytes_little_endian(vec[176], vec[177], vec[178], vec[179], vec[180], vec[181], vec[182], vec[183]),
            u16rx_nohandler: long_from_bytes_little_endian(vec[184], vec[185], vec[186], vec[187], vec[188], vec[189], vec[190], vec[191]),
        });
    }
}

#[derive(Debug)]
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
    pub fn read<T>(data: &mut T) -> Result<(RoutingAttribute, u32), io::Error>
    where
        T: Read,
    {
        const ALIGN_TO: u16 = 4;
        let mut meta_buffer = [0; 4];
        data.read_exact(&mut meta_buffer)?;

        let length: u16 = short_from_bytes_little_endian(meta_buffer[0], meta_buffer[1]);
        let attr_type: u16 = short_from_bytes_little_endian(meta_buffer[2], meta_buffer[3]);
        let padding_length: u32 = (((length + ALIGN_TO - 1) & !(ALIGN_TO - 1)) - length) as u32;

        let mut data_buffer = vec![0; length as usize - 4];
        data.read_exact(&mut data_buffer);

        let mut padding_buffer = vec![0; padding_length as usize];
        data.read_exact(&mut padding_buffer);

        return Ok((
            match attr_type {
                1 => RoutingAttribute::Address(MacAddress(data_buffer[..].try_into().unwrap())),
                2 => RoutingAttribute::Broadcast(MacAddress(data_buffer[..].try_into().unwrap())),
                3 => RoutingAttribute::InterfaceName(CStr::from_bytes_with_nul(&data_buffer).unwrap().to_str().unwrap().to_owned()),
                4 => RoutingAttribute::Mtu(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                5 => RoutingAttribute::Link(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                6 => RoutingAttribute::QDisc(CStr::from_bytes_with_nul(&data_buffer).unwrap().to_str().unwrap().to_owned()),
                7 => RoutingAttribute::Stats(LinkStats::try_from(data_buffer).unwrap()),
                8 => RoutingAttribute::Cost(data_buffer),
                9 => RoutingAttribute::Priority(data_buffer),
                10 => RoutingAttribute::Master(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                11 => RoutingAttribute::Wireless(data_buffer),
                12 => RoutingAttribute::ProtInfo(data_buffer),
                13 => RoutingAttribute::TXQLen(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                14 => RoutingAttribute::Map(data_buffer),
                15 => RoutingAttribute::Weight(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                16 => RoutingAttribute::OperationalState(data_buffer[0]),
                17 => RoutingAttribute::LinkMode(data_buffer[0]),
                18 => RoutingAttribute::LinkInfo(data_buffer),
                19 => RoutingAttribute::NetNSPid(data_buffer),
                20 => RoutingAttribute::Alias(CStr::from_bytes_with_nul(&data_buffer).unwrap().to_str().unwrap().to_owned()),
                21 => RoutingAttribute::NumVfs(data_buffer),
                22 => RoutingAttribute::VfInfoList(data_buffer),
                23 => RoutingAttribute::Stats64(LinkStats64::try_from(data_buffer).unwrap()),
                24 => RoutingAttribute::VfPorts(data_buffer),
                25 => RoutingAttribute::PortSelf(data_buffer),
                26 => RoutingAttribute::AfSpec(data_buffer),
                27 => RoutingAttribute::Group((&data_buffer[..]).try_into().unwrap()),
                28 => RoutingAttribute::NetNsFd(data_buffer),
                29 => RoutingAttribute::ExtMask(data_buffer),
                30 => RoutingAttribute::Promiscuity(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                31 => RoutingAttribute::NumTxQueues(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                32 => RoutingAttribute::NumRxQueues(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                33 => RoutingAttribute::Carrier(data_buffer[0]),
                34 => RoutingAttribute::PhysPortId(data_buffer),
                35 => RoutingAttribute::CarrierChanges(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                36 => RoutingAttribute::PhysSwitchId(data_buffer),
                37 => RoutingAttribute::LinkNetNsId(data_buffer),
                38 => RoutingAttribute::PhysPortName(data_buffer),
                39 => RoutingAttribute::ProtoDown(data_buffer[0]),
                40 => RoutingAttribute::GsoMaxSegs(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                41 => RoutingAttribute::GsoMaxSize(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                42 => RoutingAttribute::Pad(data_buffer),
                43 => RoutingAttribute::Xdp(data_buffer),
                44 => RoutingAttribute::Event(data_buffer),
                45 => RoutingAttribute::NewNetNsId(data_buffer),
                46 => RoutingAttribute::IfNetNsId(data_buffer),
                47 => RoutingAttribute::CarrierUpCount(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                48 => RoutingAttribute::CarrierDownCount(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                49 => RoutingAttribute::NewIfIndex(data_buffer),
                50 => RoutingAttribute::MinMtu(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                51 => RoutingAttribute::MaxMtu(int_from_bytes_little_endian(
                    data_buffer[0],
                    data_buffer[1],
                    data_buffer[2],
                    data_buffer[3],
                )),
                52 => RoutingAttribute::PropList(data_buffer),
                53 => RoutingAttribute::AltIfName(data_buffer),
                54 => RoutingAttribute::PermAddress(data_buffer),
                _ => RoutingAttribute::Unknown(attr_type, data_buffer),
            },
            length as u32 + padding_length,
        ));
    }
}
