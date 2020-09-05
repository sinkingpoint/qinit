#[derive(Debug)]
#[repr(C)]
pub struct InterfaceInfoMessage {
    pub interface_type: InterfaceType,   /* Device type */
    pub interface_index: i32,            /* Interface index */
    pub interface_flags: InterfaceFlags, /* Device flags  */
}

impl InterfaceInfoMessage {
    pub fn new(msg_type: InterfaceType, index: i32, flags: InterfaceFlags) -> InterfaceInfoMessage {
        return InterfaceInfoMessage {
            interface_type: msg_type,
            interface_index: index,
            interface_flags: flags,
        };
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
    fn read<T: Read>(mut reader: &mut T) -> Result<LinkStats, NetLinkError> {
        let endianness = &Endianness::Little;
        return Ok(LinkStats {
            rx_packets: read_u32(&mut reader, endianness)?,
            u16tx_packets: read_u32(&mut reader, endianness)?,
            u16rx_bytes: read_u32(&mut reader, endianness)?,
            u16tx_bytes: read_u32(&mut reader, endianness)?,
            u16rx_errors: read_u32(&mut reader, endianness)?,
            u16tx_errors: read_u32(&mut reader, endianness)?,
            u16rx_dropped: read_u32(&mut reader, endianness)?,
            u16tx_dropped: read_u32(&mut reader, endianness)?,
            u16multicast: read_u32(&mut reader, endianness)?,
            u16collisions: read_u32(&mut reader, endianness)?,
            u16rx_length_errors: read_u32(&mut reader, endianness)?,
            u16rx_over_errors: read_u32(&mut reader, endianness)?,
            u16rx_crc_errors: read_u32(&mut reader, endianness)?,
            u16rx_frame_errors: read_u32(&mut reader, endianness)?,
            u16rx_fifo_errors: read_u32(&mut reader, endianness)?,
            u16rx_missed_errors: read_u32(&mut reader, endianness)?,
            u16tx_aborted_errors: read_u32(&mut reader, endianness)?,
            u16tx_carrier_errors: read_u32(&mut reader, endianness)?,
            u16tx_fifo_errors: read_u32(&mut reader, endianness)?,
            u16tx_heartbeat_errors: read_u32(&mut reader, endianness)?,
            u16tx_window_errors: read_u32(&mut reader, endianness)?,
            u16rx_compressed: read_u32(&mut reader, endianness)?,
            u16tx_compressed: read_u32(&mut reader, endianness)?,
            u16rx_nohandler: read_u32(&mut reader, endianness)?,
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
    fn read<T: Read>(mut reader: &mut T) -> Result<LinkStats64, NetLinkError> {
        let endianness = &Endianness::Little;
        return Ok(LinkStats64 {
            rx_packets: read_u64(&mut reader, endianness)?,
            u16tx_packets: read_u64(&mut reader, endianness)?,
            u16rx_bytes: read_u64(&mut reader, endianness)?,
            u16tx_bytes: read_u64(&mut reader, endianness)?,
            u16rx_errors: read_u64(&mut reader, endianness)?,
            u16tx_errors: read_u64(&mut reader, endianness)?,
            u16rx_dropped: read_u64(&mut reader, endianness)?,
            u16tx_dropped: read_u64(&mut reader, endianness)?,
            u16multicast: read_u64(&mut reader, endianness)?,
            u16collisions: read_u64(&mut reader, endianness)?,
            u16rx_length_errors: read_u64(&mut reader, endianness)?,
            u16rx_over_errors: read_u64(&mut reader, endianness)?,
            u16rx_crc_errors: read_u64(&mut reader, endianness)?,
            u16rx_frame_errors: read_u64(&mut reader, endianness)?,
            u16rx_fifo_errors: read_u64(&mut reader, endianness)?,
            u16rx_missed_errors: read_u64(&mut reader, endianness)?,
            u16tx_aborted_errors: read_u64(&mut reader, endianness)?,
            u16tx_carrier_errors: read_u64(&mut reader, endianness)?,
            u16tx_fifo_errors: read_u64(&mut reader, endianness)?,
            u16tx_heartbeat_errors: read_u64(&mut reader, endianness)?,
            u16tx_window_errors: read_u64(&mut reader, endianness)?,
            u16rx_compressed: read_u64(&mut reader, endianness)?,
            u16tx_compressed: read_u64(&mut reader, endianness)?,
            u16rx_nohandler: read_u64(&mut reader, endianness)?,
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
    pub fn read<T: Read>(mut data: &mut T) -> Result<(RoutingAttribute, u32), NetLinkError> {
        let length: u16 = read_u16(&mut data, &Endianness::Little)?;
        let attr_type: u16 = read_u16(&mut data, &Endianness::Little)?;
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
                27 => RoutingAttribute::Group((&data_buffer[..]).try_into()?),
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
