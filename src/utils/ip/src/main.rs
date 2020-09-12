extern crate clap;
extern crate libq;

use clap::{App, AppSettings, Arg, SubCommand};
use libq::netlink::rtnetlink::RTNetlink;
use libq::netlink::{NetLinkSocket, SockProtocol};

fn main() {
    let args = App::new("ip")
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

    for link in links {
        let link = link.unwrap();
        print!("{}: ", link.interface_index);
        if let Some(name) = link.rtattrs.interface_name {
            print!("{}: ", name);
        }

        print!("<{}> ", link.interface_flags.to_string());

        if let Some(mtu) = link.rtattrs.mtu {
            print!("mtu {} ", mtu);
        }

        if let Some(qdisc) = link.rtattrs.qdisc {
            print!("qdisc {} ", qdisc);
        }

        if let Some(master_id) = link.rtattrs.master_id {
            print!("master {} ", master_id);
        }

        if let Some(state) = link.rtattrs.oper_state {
            print!("state {} ", state);
        }

        if let Some(mode) = link.rtattrs.link_mode {
            print!("mode {} ", mode);
        }

        if let Some(group) = link.rtattrs.group {
            print!("group {} ", group);
        }

        if let Some(qlen) = link.rtattrs.txqueue_len {
            if qlen != 0 {
                print!("qlen {} ", qlen);
            }
        }

        println!("");

        print!("    link/{} ", link.interface_type);

        if let Some(addr) = link.rtattrs.address {
            print!("{} ", addr);
        }

        if let Some(addr) = link.rtattrs.broadcast {
            print!("brd {}  ", addr);
        }

        println!("");
    }
}
