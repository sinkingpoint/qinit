extern crate clap;
extern crate libq;
extern crate libsudo;

use libsudo::sudoers::{Identity, Sudoers};

use libq::logger;

use clap::{App, Arg};

fn main() {
    let logger = logger::with_name_as_console("sudo");
    let config = match Sudoers::read_from_disk() {
        Some(config) => config,
        None => {
            logger.info().smsg("Syntax error reading sudoers file");
            return;
        }
    };

    let args = App::new("sudo")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Execute a command as another user")
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .takes_value(true)
                .help("Run the command as a user other than root"),
        )
        .arg(
            Arg::with_name("group")
                .short("g")
                .long("group")
                .takes_value(true)
                .help("Run the command with the primary group as something other than root"),
        )
        .arg(
            Arg::with_name("command")
                .takes_value(true)
                .multiple(true)
                .help("The command to run"),
        )
        .get_matches();

    let user = match args.value_of("user").unwrap_or("#1") {
        arg if arg.starts_with("#") => {
            let id: u32 = match arg[1..].parse() {
                Ok(id) => id,
                Err(_) => {
                    logger.info().smsg("Failed to parse user as numerical ID");
                    return;
                }
            };
            Identity::Uid(id)
        }
        arg => Identity::User(arg.to_owned()),
    };

    let group = match args.value_of("group").unwrap_or("#1") {
        arg if arg.starts_with("#") => {
            let id: u32 = match arg[1..].parse() {
                Ok(id) => id,
                Err(_) => {
                    logger.info().smsg("Failed to parse group as numerical ID");
                    return;
                }
            };
            Identity::Gid(id)
        }
        arg => Identity::Group(arg.to_owned()),
    };

    let command = args.values_of("command").unwrap().collect();
    let (allowed, options) = config.is_allowed(command, user, group);

    if allowed {
        println!("Allowed!");
    } else {
        println!("Not allowed!");
    }
}
