extern crate clap;
extern crate libq;
extern crate nix;

use clap::{App, Arg};

use nix::unistd::{getegid, geteuid, getgid, getuid};

use libq::passwd::{GroupEntry, Groups, PasswdEntry};

fn main() {
    let args = App::new("id")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Get user/group id information")
        .arg(
            Arg::with_name("user")
                .short("u")
                .help("Only print the user ID")
                .conflicts_with_all(&["group", "groups"]),
        )
        .arg(
            Arg::with_name("group")
                .short("g")
                .help("Only print the group ID")
                .conflicts_with_all(&["user", "groups"]),
        )
        .arg(
            Arg::with_name("groups")
                .short("G")
                .help("Print all group IDs")
                .conflicts_with_all(&["group", "user"]),
        )
        .arg(Arg::with_name("name").short("n").help("Print names instead of just numerical IDs"))
        .get_matches();

    let name = args.is_present("name");
    let group = args.is_present("group");
    let groups = args.is_present("groups");
    let user = args.is_present("user");

    if name && !(group || groups || user) {
        eprintln!("-n _must_ be used with one of -ugG");
        return;
    }

    let uid = PasswdEntry::by_uid(getuid().as_raw()).unwrap();
    let gid = GroupEntry::by_gid(getgid().as_raw()).unwrap();
    let euid = PasswdEntry::by_uid(geteuid().as_raw()).unwrap();
    let egid = GroupEntry::by_gid(getegid().as_raw()).unwrap();

    if group {
        if name {
            println!("{}", gid.name);
        } else {
            println!("{}", gid.gid);
        }
    } else if user {
        if name {
            println!("{}", uid.username);
        } else {
            println!("{}", uid.uid);
        }
    } else if groups {
        let groups = Groups::new().filter(|group| group.users.contains(&uid.username));
        if name {
            println!("{} {}", gid.name, groups.map(|group| group.name).collect::<Vec<String>>().join(" "));
        } else {
            println!(
                "{} {}",
                gid.gid,
                groups.map(|group| group.gid.to_string()).collect::<Vec<String>>().join(" ")
            );
        }
    } else {
        print!("uid={}({}) gid={}({}) ", uid.uid, uid.username, gid.gid, gid.name);
        if uid.uid != euid.uid || gid.gid != egid.gid {
            print!("euid={}({}) egid={}({}) ", euid.uid, euid.username, egid.gid, egid.name);
        }
        print!("\n");
    }
}
