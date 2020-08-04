extern crate clap;
extern crate nix;

use clap::{App, Arg};
use std::ffi::{CStr, CString};
use std::path::Path;

use nix::fcntl::{open, OFlag};
use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::sys::stat::{stat, Mode};
use nix::unistd::{chdir, chroot, close, execv};

fn switchroot(new_root: &Path) -> Result<(), ()> {
    if !new_root.exists() {
        eprintln!("New root location `{}` doesn't exist!", new_root.display());
        return Err(());
    }

    let new_root_stat = stat(new_root).expect("Failed to stat new root. Does it exist?");

    for mount_dev in ["dev", "proc", "sys", "run"].iter() {
        let old_mount_path_str = format!("/{}", mount_dev);
        let old_mount_path = Path::new(&old_mount_path_str);
        let new_mount_path = Path::new(new_root).join(mount_dev).into_boxed_path();
        println!("Moving {} to {:?}", old_mount_path.display(), new_mount_path);
        match stat(&*new_mount_path) {
            Ok(new_mount_stat) => {
                if new_mount_stat.st_dev != new_root_stat.st_dev {
                    eprintln!(
                        "System Filesystem {} is not on the current root. Skipping it.",
                        new_mount_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to stat new mount path {}: {}. Skipping it", new_mount_path.display(), e);
                continue;
            }
        };

        if let Err(_) = mount::<Path, Path, str, str>(Some(&*old_mount_path), &*new_mount_path, None, MsFlags::MS_MOVE, None) {
            eprintln!(
                "Failed to move {} to {}, forcing umount of {}",
                old_mount_path.display(),
                new_mount_path.display(),
                old_mount_path.display()
            );
            umount2(old_mount_path, MntFlags::MNT_FORCE).expect("Failed to unmount old fs");
        }
    }

    chdir(new_root).expect("Failed to change to new root");

    let old_root_fd = open("/", OFlag::O_RDONLY, Mode::empty()).expect("Failed to open old root dir");

    if let Err(_) = mount::<Path, str, str, str>(Some(new_root), "/", None, MsFlags::MS_MOVE, None) {
        close(old_root_fd).unwrap();
        eprintln!("Failed to mount moving {} to /", new_root.display());
        return Err(());
    }

    if let Err(_) = chroot(".") {
        close(old_root_fd).unwrap();
        eprintln!("Failed to change into root");
        return Err(());
    }

    close(old_root_fd).unwrap();

    //TODO: Remove the old file system

    return Ok(());
}

fn main() {
    let args = App::new("switch_root")
        .version("0.1")
        .author("Colin D. <colin@quirl.co.nz>")
        .about("Switch the root file system to another path, moving all mounted system fs' and deleting the old one")
        .arg(
            Arg::with_name("newroot")
                .takes_value(true)
                .value_name("NEWROOT")
                .index(1)
                .required(true)
                .help("The new root directory"),
        )
        .arg(
            Arg::with_name("init")
                .takes_value(true)
                .value_name("INIT")
                .index(2)
                .required(true)
                .help("The init script to run"),
        )
        .arg(
            Arg::with_name("initargs")
                .takes_value(true)
                .multiple(true)
                .value_name("INITARGS")
                .index(3)
                .required(true)
                .help("The args to give to the init script"),
        )
        .get_matches();

    let new_root = args.value_of("newroot").unwrap();
    if let Err(_) = switchroot(&Path::new(new_root)) {
        std::process::exit(1);
    }

    let init = args.value_of("init").unwrap();
    let mut init_args: Vec<&str> = args.values_of("initargs").unwrap().collect();
    init_args.insert(0, init);

    let c_path = CString::new(init).unwrap();
    let cstr_argv: Vec<Vec<u8>> = init_args
        .iter()
        .map(|arg| CString::new(*arg).unwrap().into_bytes_with_nul())
        .collect();
    let argv = &cstr_argv
        .iter()
        .map(|arg| CStr::from_bytes_with_nul(arg).unwrap())
        .collect::<Vec<&CStr>>()[..];
    execv(&c_path, argv).expect("Failed executing init script");
}
