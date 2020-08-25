use tasks::serde::{TaskDef, DependencyDef, Stage};
use super::process::fork_process;

/// SphereType is an enum containing a generic interface over all the different types
/// of sphere that exist. Why we do this over a trait and trait objects is a bit arbitrary
#[derive(Debug)]
pub enum SphereType {
    Task(TaskDef),
    Stage(Stage)
}

pub trait Startable {
    fn start(&self, current_state: Option<&SphereState>) -> Option<SphereStatus>;
}

impl Startable for SphereType {
    fn start(&self, current_state: Option<&SphereState>) -> Option<SphereStatus> {
        return match self {
            SphereType::Task(def) => def.start(current_state),
            SphereType::Stage(def) => def.start(current_state)
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
    Stopped,
    Stopping,
    PreStarting,
    Starting,
    Started,
    Exited(u32),
    NotStarted,
    FailedToStart
}

impl Startable for Stage {
    fn start(&self, _current_state: Option<&SphereState>) -> Option<SphereStatus> {
        return Some(SphereStatus {
            state: SphereState::Started,
            pid: None
        });
    }
}

impl Startable for TaskDef {
    fn start(&self, current_state: Option<&SphereState>) -> Option<SphereStatus> {
        let pid;
        let new_state;
        match current_state {
            None | Some(SphereState::Stopped) | Some(SphereState::Exited(_)) | Some(SphereState::NotStarted) | Some(SphereState::FailedToStart) => {
                if let Some(cmd) = &self.init_command {
                    // We haven't started anything, and there's an init command to run
                    pid = fork_process(&cmd.split_whitespace().collect());
                    new_state = SphereState::PreStarting;
                }
                else {
                    pid = fork_process(&self.start_command.split_whitespace().collect());
                    new_state = SphereState::Starting;
                }
            },
            Some(SphereState::PreStarting) => {
                pid = fork_process(&self.start_command.split_whitespace().collect());
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