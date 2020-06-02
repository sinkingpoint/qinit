extern crate libqinit;
extern crate libq;

use libq::logger::{self};
use libqinit::tasks::TaskRegistry;
use std::path::PathBuf;

fn main() {
    let logger = logger::with_name_as_json("test");
    let task_registry = TaskRegistry::load_from_disk(&vec![
        PathBuf::from("./tasks")
    ]).expect("Failed to load tasks");

    logger.info().msg(format!("Loaded {} tasks", task_registry.len()));
}