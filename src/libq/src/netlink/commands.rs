use super::api::{NLMsgHeader, IFInfoMsg, NLMsgFlags, MessageType, RoutingAttribute, InterfaceFlags, MacAddress, IPv4Addr, InterfaceType};
use std::mem::size_of;
use std::io::{self, Read};
use super::super::mem::read_struct;

pub trait NLCommand {
    fn as_bytes(&self) -> &[u8];
}

pub struct GetInterfacesCommand {
    header: NLMsgHeader,
    body: IFInfoMsg
}

impl GetInterfacesCommand {
    pub fn new(sequence_number: u32) -> GetInterfacesCommand {
        return GetInterfacesCommand {
            header: NLMsgHeader::new(MessageType::RTM_GETLINK, sequence_number, size_of::<GetInterfacesCommand>() as u32, NLMsgFlags::NLM_F_REQUEST | NLMsgFlags::NLM_F_MATCH),
            body: IFInfoMsg::empty()
        }
    }
}

#[derive(Debug)]
pub struct NewInterfaceCommand {
    header: NLMsgHeader,
    body: IFInfoMsg,
    attrs: Vec<RoutingAttribute>
}

impl NewInterfaceCommand {
    pub fn read_after_header<T>(header: NLMsgHeader, buffer: &mut T) -> Result<NewInterfaceCommand, io::Error> where T: Read{
        let body = read_struct(buffer).unwrap();
        let mut count = header.length - 32;
        let mut attrs = Vec::new();

        while count > 0 {
            let (attr, size) = RoutingAttribute::read(buffer)?;
            count -= size;
            attrs.push(attr);
        }

        return Ok(NewInterfaceCommand {
            header: header,
            body: body,
            attrs: attrs
        });
    }
}

pub struct Interface {
    pub flags: InterfaceFlags,
    pub name: String,
    pub mtu: u32,
    pub qdisc: String,
    pub state: u8,
    pub queue_length: u32,
    pub group: IPv4Addr,
    pub attrs: Vec<RoutingAttribute>,
    pub mac_address: MacAddress,
    pub broadcast_address: MacAddress,
    pub index: i32,
    pub int_type: InterfaceType
}

impl Into<Interface> for NewInterfaceCommand {
    fn into(self) -> Interface {
        let mut int = Interface {
            flags: self.body.interface_flags,
            name: String::new(),
            mtu: 65536,
            qdisc: String::new(),
            state: 0,
            queue_length: 0,
            group: [0, 0, 0, 0,],
            attrs: Vec::new(),
            mac_address: MacAddress([0, 0, 0, 0, 0, 0]),
            broadcast_address: MacAddress([0, 0, 0, 0, 0, 0]),
            index: self.body.interface_index,
            int_type: self.body.interface_type
        };

        for attr in self.attrs.into_iter() {
            match attr {
                RoutingAttribute::InterfaceName(name) => {
                    int.name = name;
                },
                RoutingAttribute::Mtu(mtu) => {
                    int.mtu = mtu;
                },
                RoutingAttribute::QDisc(qdisc) => {
                    int.qdisc = qdisc;
                },
                RoutingAttribute::OperationalState(state) => {
                    int.state = state;
                },
                RoutingAttribute::TXQLen(qlen) => {
                    int.queue_length = qlen;
                },
                RoutingAttribute::Group(addr) => {
                    int.group = addr;
                },
                RoutingAttribute::Address(addr) => {
                    int.mac_address = addr;
                },
                RoutingAttribute::Broadcast(addr) => {
                    int.broadcast_address = addr;
                },
                attr => {
                    int.attrs.push(attr);
                }
            }
        }

        return int;
    }
}