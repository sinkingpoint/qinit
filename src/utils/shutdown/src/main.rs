extern crate nix;

use nix::sys::reboot::{reboot, RebootMode};

fn main() {
    reboot(RebootMode::RB_POWER_OFF).expect("Failed to issue shutdown command");
}