extern crate accountant;

use accountant::tasks::{SphereRegistry, DependencyDef, listen_for_children, CourthouseBuilder};
use std::path::Path;
use std::sync::{Arc, Mutex};

fn main() {
    let registry = SphereRegistry::read_from_disk(vec![&Path::new("/etc/qinit/tasks"), &Path::new("src/rootfs/tasks"), &Path::new("/cats")]).unwrap();
    let registry_lock = Arc::new(Mutex::new(registry));
    let send = registry_lock.clone();
    let handle = std::thread::spawn(move || listen_for_children(send));

    {
        let mut registry = registry_lock.lock().unwrap();
        registry.set_courthouse_builder(CourthouseBuilder::new(registry_lock.clone()));    
        registry.start(DependencyDef::new("multiusermode".to_owned(), None));
    }

    handle.join().expect("Expected children reaper to never exit");
}
