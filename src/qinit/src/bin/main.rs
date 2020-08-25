extern crate accountant;

use accountant::tasks::{SphereRegistry, DependencyDef};
use std::path::Path;
use std::sync::{Arc, Mutex};

fn main() {
    let registry = SphereRegistry::read_from_disk(vec![&Path::new("/etc/qinit/tasks"), &Path::new("src/rootfs/tasks"), &Path::new("/cats")]).unwrap();
    let registry_lock = Arc::new(Mutex::new(registry));

    let mut registry = registry_lock.lock().unwrap();
    registry.start(DependencyDef::new("multiusermode".to_owned(), None));
}
