extern crate libq;

use libq::blkid;
use std::path::PathBuf;

fn main() {
    let dev = blkid::Device::from_path(PathBuf::from("./cats")).unwrap();
    if let Some(probe_result) = dev.probe() {
        println!("{}", probe_result);
    } else {
        println!("Failed to probe device");
    }
}
