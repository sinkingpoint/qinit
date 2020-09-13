extern crate clap;
extern crate libq;

use clap::{App, AppSettings, Arg, SubCommand};
use libq::netlink::rtnetlink::RTNetlink;
use libq::netlink::{NetLinkSocket, SockProtocol};
use libq::tabulate::Table;

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
                .subcommand(SubCommand::with_name("show").about("Show Interface details")),
        )
        .get_matches();

    let mut socket = NetLinkSocket::new(SockProtocol::NetlinkRoute).unwrap();
    let links = socket.get_links().unwrap();
    let mut table = Table::new(&["Index", "Name", "Flags", "MTU", "QDisc", "Master", "State", "Mode", "Group", "QLen", "Type", "Address", "Broadcast"]);

    for link in links {
        let link = link.unwrap();
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
        table.add_values(vec![index, name, flags, mtu, qdisc, master, state, mode, group, qlen, link_type, addr, broadcast]).unwrap();
    }

    println!("{}", table);
}
