extern crate clap;
extern crate libq;

use clap::{App, AppSettings, Arg, SubCommand};
use libq::netlink::rtnetlink::{InterfaceFlags, OperationalState, RTNetlink};
use libq::netlink::{NetLinkSocket, SockProtocol};
use libq::tabulate::Table;

fn print_links(mut socket: NetLinkSocket) {
    let links = socket.get_links().unwrap();
    let mut table = Table::new(&[
        "Index",
        "Name",
        "Flags",
        "MTU",
        "QDisc",
        "Master",
        "State",
        "Mode",
        "Group",
        "QLen",
        "Type",
        "Address",
        "Broadcast",
    ]);

    for link in links {
        let index = format!("{}", link.interface_index);
        let name = link.rtattrs.interface_name.as_deref().unwrap_or("").to_owned();
        let flags = link.interface_flags.to_string();
        let mtu = format!("{}", link.rtattrs.mtu.unwrap_or(0));
        let qdisc = link.rtattrs.qdisc.as_deref().unwrap_or("").to_owned();
        let master = format!("{}", link.rtattrs.master_id.unwrap_or(0));
        let state = link.rtattrs.oper_state.and_then(|x| Some(x.to_string())).unwrap_or(String::new());
        let mode = link.rtattrs.link_mode.and_then(|x| Some(x.to_string())).unwrap_or(String::new());
        let group = format!("{}", link.rtattrs.group.unwrap_or(0));
        let qlen = format!("{}", link.rtattrs.txqueue_len.unwrap_or(0));
        let link_type = format!("link/{}", link.interface_type);
        let addr = link.rtattrs.address.and_then(|x| Some(x.to_string())).unwrap_or(String::new());
        let broadcast = link.rtattrs.broadcast.and_then(|x| Some(x.to_string())).unwrap_or(String::new());
        table
            .add_values(vec![
                index, name, flags, mtu, qdisc, master, state, mode, group, qlen, link_type, addr, broadcast,
            ])
            .unwrap();
    }

    println!("{}", table);
}

fn print_addrs(mut socket: NetLinkSocket) {
    let addrs = socket.get_addrs().unwrap();
    for addr in addrs {
        println!("{:?}", addr);
    }
}

fn set_link_state(mut socket: NetLinkSocket, link_name: &str, state: bool) {
    let link = socket
        .get_links()
        .unwrap()
        .into_iter()
        .find(|link| link.rtattrs.interface_name.as_deref().unwrap_or("") == link_name);

    if let Some(mut link) = link {
        link.clear();
        if state {
            link.interface_flags |= InterfaceFlags::IFF_UP;
        } else {
            link.interface_flags &= !InterfaceFlags::IFF_UP;
        }
        link.change_mask = 1;
        socket.set_link(&link).unwrap();
    } else {
        // TODO: Error here - link not found
        return;
    }
}

fn main() {
    let args = App::new("netc")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Show and Change Networking Info")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("link")
                .about("Show/Change Information about Links")
                .setting(AppSettings::SubcommandRequired)
                .subcommand(SubCommand::with_name("show").about("Show Interface details"))
                .subcommand(
                    SubCommand::with_name("up")
                        .about("Try and bring the link up")
                        .arg(Arg::with_name("dev").index(1).required(true).help("The device name to act on")),
                )
                .subcommand(
                    SubCommand::with_name("down")
                        .about("Try and bring the link down")
                        .arg(Arg::with_name("dev").index(1).required(true).help("The device name to act on")),
                ),
        )
        .subcommand(
            SubCommand::with_name("addr")
                .about("Show/Change Information about Addresses")
                .setting(AppSettings::SubcommandRequired)
                .subcommand(SubCommand::with_name("show").about("Show Address details")),
        )
        .get_matches();

    let socket = NetLinkSocket::new(SockProtocol::NetlinkRoute).unwrap();
    match args.subcommand() {
        ("link", Some(matches)) => match matches.subcommand() {
            ("show", Some(matches)) => {
                print_links(socket);
            }
            ("up", Some(matches)) => {
                set_link_state(socket, matches.value_of("dev").unwrap(), true);
            }
            ("down", Some(matches)) => {
                set_link_state(socket, matches.value_of("dev").unwrap(), false);
            }
            _ => {}
        },
        ("addr", Some(matches)) => match matches.subcommand() {
            ("show", Some(matches)) => {
                print_addrs(socket);
            }
            _ => {}
        },
        _ => {}
    }
}
