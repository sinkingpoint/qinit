use nix::sys::inotify::{Inotify, InitFlags, AddWatchFlags, WatchDescriptor};
use tasks::serde::DependencyDef;
use patient::{Status, FreudianClient};

use std::thread;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::registry::SphereRegistry;


/// A Courthouse is responsible for using inotify to listen for a set of events to 
/// occur on a set of files, and informing a given registry when all the changes have been seen
pub struct Courthouse {
    /// The inotify client we use to listen for events
    inotify_client: Inotify,

    /// A Map from watch_id -> directory
    watchers: HashMap<WatchDescriptor, PathBuf>,

    /// The list of events we're still waiting on for each file
    waiting_list: HashMap<PathBuf, AddWatchFlags>,

    /// The list of freudian topics we expect to be created
    freudian_topic_list: HashSet<String>,

    /// The registry that this courthouse will report its findings to
    registry: Arc<Mutex<SphereRegistry>>,

    /// The DependencyDef that this Courthouse is waiting for filings for
    dependency: DependencyDef
}

pub struct MonitorRequest {
    path: PathBuf,
    events: AddWatchFlags
}

pub struct CourthouseBuilder {
    /// The registry that this courthouse will report its findings to
    registry: Arc<Mutex<SphereRegistry>>
}

impl CourthouseBuilder {
    pub fn new(registry: Arc<Mutex<SphereRegistry>>) -> CourthouseBuilder {
        return CourthouseBuilder {
            registry: registry
        }
    }

    pub fn build(&self, dep: DependencyDef, requests: Vec<MonitorRequest>, freudian_topics: HashSet<String>) -> Result<Courthouse, nix::Error> {
        return Courthouse::new(dep, requests, freudian_topics, self.registry.clone());
    }
}

impl MonitorRequest {
    pub fn new(path: PathBuf, events: AddWatchFlags) -> MonitorRequest {
        return MonitorRequest {
            path: path,
            events: events
        }
    }
}

impl Courthouse {
    fn new(dep: DependencyDef, requests: Vec<MonitorRequest>, freudian_topics: HashSet<String>, registry: Arc<Mutex<SphereRegistry>>) -> Result<Courthouse, nix::Error> {
        let mut req_map = HashMap::new();
        let mut watchers = HashMap::new();

        let inotify_client = Inotify::init(InitFlags::empty())?;
        for request in requests.into_iter() {
            let parent = request.path.parent().unwrap();
            watchers.insert(inotify_client.add_watch(parent, AddWatchFlags::all())?, PathBuf::from(parent));
            req_map.insert(request.path, request.events);
        }

        return Ok(Courthouse {
            inotify_client: inotify_client,
            waiting_list: req_map,
            watchers: watchers,
            registry: registry,
            freudian_topic_list: freudian_topics,
            dependency: dep
        });
    }

    pub fn start(self) {
        std::thread::spawn(move || {
            self.main_loop()
        });
    }

    fn main_loop(mut self) {
        while self.waiting_list.len() > 0 {
            match self.inotify_client.read_events() {
                Ok(events) => {
                    for event in events.iter() {
                        let mut parts = Vec::new();
                        parts.push(self.watchers.get(&event.wd).unwrap().clone());
                        if event.name.is_some() {
                            let name = event.name.as_ref().unwrap().clone();
                            parts.push(PathBuf::from(name.into_string().unwrap()));
                        }
                        let full_path: PathBuf = parts.into_iter().collect();

                        if self.waiting_list.contains_key(&full_path) {
                            if (*self.waiting_list.get(&full_path).unwrap() & event.mask).bits() != 0 {
                                self.waiting_list.remove(&full_path);
                            }
                        }
                    }
                },
                Err(_e) => {}
            }
        }

        if self.freudian_topic_list.len() > 0 {
            if let Ok(mut client) = FreudianClient::new(&Path::new("/run/freudian/socket")) {
                while self.freudian_topic_list.len() > 0 {
                    match client.get_topics() {
                        Ok((names, status)) => {
                            if status == Status::Ok {
                                let names: HashSet<String> = names.unwrap().into_iter().collect();
                                self.freudian_topic_list = &self.freudian_topic_list - &names;
                            }

                            // TODO: Handle this error - tell the registry that the courthouse says no
                        },
                        Err(_err) => {
                            // TODO: Handle this error - tell the registry that the courthouse says no
                        }
                    }

                    // Sleep for 100ms. This is pretty arbitrary
                    thread::sleep(Duration::new(0, 1000));
                }
            }
        }

        let mut registry = self.registry.lock().unwrap();
        registry.handle_court_completion(self.dependency);
    }
}
