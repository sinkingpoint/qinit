extern crate accountant;

use accountant::tasks::SphereRegistry;
use std::path::Path;

fn main() {
    let registry = SphereRegistry::read_from_disk(vec![&Path::new("src/rootfs/tasks"), &Path::new("/cats")]);
}
