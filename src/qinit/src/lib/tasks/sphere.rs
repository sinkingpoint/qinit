use super::process::fork_process;
use super::super::strings::do_string_replacement;
use tasks::serde::{TaskDef, DependencyDef, Stage, RestartMode};
use tasks::court::MonitorRequest;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs::remove_file;
use nix::sys::inotify::AddWatchFlags;
use nix::unistd::mkdir;
use nix::sys::stat::Mode;

/// SphereType is an enum containing a generic interface over all the different types
/// of sphere that exist. Why we do this over a trait and trait objects is a bit arbitrary
#[derive(Debug)]
pub enum SphereType {
    Task(TaskDef),
    Stage(Stage)
}

pub trait Startable {
    fn start(&self, args: &Option<&HashMap<String, String>>, current_state: Option<&SphereState>) -> Option<SphereStatus>;
}

impl Startable for SphereType {
    fn start(&self, args: &Option<&HashMap<String, String>>, current_state: Option<&SphereState>) -> Option<SphereStatus> {
        return match self {
            SphereType::Task(def) => def.start(args, current_state),
            SphereType::Stage(def) => def.start(args, current_state)
        }
    }
}

impl SphereType {
    pub fn get_deps(&self) -> Option<&Vec<DependencyDef>> {
        match self {
            SphereType::Task(def) => def.requires.as_ref(),
            SphereType::Stage(def) => Some(def.tasks.as_ref())
        }
    }

    pub fn get_restart_mode(&self) -> RestartMode {
        match self {
            SphereType::Task(def) => def.restart_mode.unwrap_or(RestartMode::OnError),
            SphereType::Stage(_def) => RestartMode::OnError
        }
    }

    /// Gets all the required events that need to happen to start this sphere
    pub fn get_monitor_requests(&self, args: &Option<&HashMap<String, String>>) -> (Vec<MonitorRequest>, HashSet<String>) {
        match self {
            SphereType::Task(def) => {
                let mut requests = Vec::new();
                let mut freudian_topics = HashSet::new();
                if let Some(conditions) = &def.conditions {
                    // Get Unix Sockets / Files
                    if let Some(sockets) = &conditions.unixsocket {
                        for socket in sockets.iter() {
                            let path = PathBuf::from(do_string_replacement(args, &socket.path));
                            if !path.is_absolute() {
                                // Skip it as invalid
                                // TODO: Handle this case better
                                continue;
                            }
                            
                            if let Some(parent) = path.parent(){
                                if !parent.exists() {
                                    // TODO: Derive the bits here better
                                    match mkdir(parent, Mode::from_bits(0o755).unwrap()) {
                                        Ok(_) => {},
                                        Err(_) => {
                                            continue;
                                        }
                                    }
                                }
                            }

                            if path.exists() {
                                // We need to kill the old, leftover socket
                                match remove_file(&path) {
                                    Ok(_) => {},
                                    Err(_) => {
                                        continue;
                                    }
                                }
                            }

                            requests.push(MonitorRequest::new(PathBuf::from(path), AddWatchFlags::IN_CREATE));
                        }
                    }

                    // Get Freudian Topics
                    if let Some(topics) = &conditions.freudian_topic {
                        for topic in topics.iter() {
                            freudian_topics.insert(topic.name.clone());
                        }
                    }
                }

                return (requests, freudian_topics);
            },
            SphereType::Stage(_def) => (Vec::new(), HashSet::new())
        }
    }

    /// Returns true if the given dependency def has all the args required
    /// for this sphere, and no more
    pub fn completely_describes(&self, dep: &DependencyDef) -> bool {
        match self {
            SphereType::Stage(_) => {
                // Stages have no args
                return dep.args == None;
            },
            SphereType::Task(def) => {
                match (&def.args, &dep.args) {
                    (None, None) => {
                        return true;
                    },
                    (Some(_), None) | (None, Some(_)) => {
                        return false;
                    },
                    (Some(self_args), Some(dep_args)) => {
                        for arg in self_args.iter() {
                            if !dep_args.contains_key(arg) {
                                return false;
                            }
                        }

                        return self_args.len() == dep_args.len();
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SphereStatus {
    pub state: SphereState,
    pub pid: Option<u32>
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum SphereState {
    /// Indicates the Sphere was stopped by a user, and is no longer running
    Stopped,

    /// Indicates the Sphere was stopped by the user, and we've sent a SIGINT to it
    Stopping,

    /// Indicates the Sphere's init script is running
    PreStarting,

    /// Indicates the Sphere's init script has finished, but the main process has not been started
    StartPending,

    /// Indicates that the Sphere has been started, but has start conditions that have not been met yet
    Starting,

    /// Indicates that the Sphere has been started, and is currently running
    Started,

    /// Indicates that the Sphere exitted normally, with the given exit code
    Exited(u32),

    /// Indicates that the Sphere failed to start for some reason
    FailedToStart
}

impl Startable for Stage {
    fn start(&self, _args: &Option<&HashMap<String, String>>, _current_state: Option<&SphereState>) -> Option<SphereStatus> {
        return Some(SphereStatus {
            state: SphereState::Started,
            pid: None
        });
    }
}

impl Startable for TaskDef {
    fn start(&self, args: &Option<&HashMap<String, String>>, current_state: Option<&SphereState>) -> Option<SphereStatus> {
        let pid;
        let new_state;
        match current_state {
            None | Some(SphereState::Stopped) | Some(SphereState::Exited(_)) | Some(SphereState::FailedToStart) => {
                if let Some(cmd) = &self.init_command {
                    // We haven't started anything, and there's an init command to run
                    pid = fork_process(&cmd.split_whitespace().map(|x| do_string_replacement(args, x)).collect());
                    new_state = SphereState::PreStarting;
                }
                else {
                    pid = fork_process(&self.start_command.split_whitespace().map(|x| do_string_replacement(args, x)).collect());
                    new_state = SphereState::Starting;
                }
            },
            Some(SphereState::StartPending) => {
                pid = fork_process(&self.start_command.split_whitespace().map(|x| do_string_replacement(args, x)).collect());
                new_state = SphereState::Starting;
            },
            Some(_) => {
                return None;
            }
        }

        return Some(SphereStatus {
            pid: pid,
            state: new_state
        });
    }
}