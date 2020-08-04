extern crate libq;

use libq::netlink::NLSocket;

fn main() {
    let socket = NLSocket::new();
    let interfaces = socket.get_interfaces();
    for interface in interfaces.iter() {
        println!(
            "{}: {}: <{:?}> mtu {} qdisc {} state {} group {:?} qlen {}",
            interface.index,
            interface.name,
            interface.flags,
            interface.mtu,
            interface.qdisc,
            interface.state,
            interface.group,
            interface.queue_length
        );
        println!(
            "\t{:?} {} brd {}",
            interface.int_type, interface.mac_address, interface.broadcast_address
        );
    }
}
