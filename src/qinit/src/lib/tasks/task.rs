use super::serde::{ServiceDef, StageDef, DependencyDef, RestartMode};
use super::Identifier;
use std::convert::From;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fmt;

#[derive(Debug)]
pub enum TaskState {
    Stopped,
    Started,
    Stopping,
    Starting,
    Failed
}

pub struct Dependency {
    name: String,
    args: HashMap<String, String>
}

impl Dependency {
    pub fn get_name(&self) -> &String {
        return &self.name;
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (key, value) in self.args.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut arg_string = String::new();
        for (key, value) in self.args.iter() {
            arg_string.push_str(format!("{}={}", key, value).as_str());
        }
        write!(f, "Depency{{ {} {} }}", self.name, arg_string)?;
        return Ok(());
    }
}

impl From<DependencyDef> for Dependency {
    fn from(item: DependencyDef) -> Self {
        return Dependency {
            name: item.name.to_lowercase(),
            args: item.args
        }
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.args.len() != other.args.len() {
            return false;
        }

        for (key, value) in self.args.iter() {
            match other.args.get(key) {
                None => {
                    return false;
                },
                Some(other_val) if other_val != value => {
                    return false;
                },
                _ => {}
            }
        }

        return true;
    }
}

impl Eq for Dependency{}

/// A trait that defines the minimum requirements for a task to be run by QInit
pub trait Task {
    fn get_name(&self) -> &String;
    fn get_deps(&self) -> &Vec<Dependency>;
}

/// A struct that represents a service/daemon being run in the system
/// `Service`s each get their own CGroup and optionally have a list of services
/// that must have started successfully before this one
pub struct Service {
    name: String,
    description: Option<String>,
    user: Option<Identifier>,
    group: Option<Identifier>,
    args: Vec<String>,
    restart_mode: RestartMode,
    requirements: Vec<Dependency>, // A list of Units that should be started _before_ this one
    command: String,
    state: TaskState,
}

impl From<ServiceDef> for Service {
    fn from(item: ServiceDef) -> Self {
        return Service {
            name: item.name.to_lowercase(),
            description: item.description,
            user: item.user,
            group: item.group,
            args: match item.args {
                None => Vec::new(),
                Some(args) => args
            },
            restart_mode: item.restart_mode.unwrap_or(RestartMode::OnCrash),
            requirements: match item.requirements {
                None => Vec::new(),
                Some(reqs) => reqs.into_iter().map(|dep| Dependency::from(dep)).collect()
            },
            command: item.command,
            state: TaskState::Stopped
        }
    }
}

impl Task for Service {
    fn get_name(&self) -> &String {
        return &self.name;
    }

    fn get_deps(&self) -> &Vec<Dependency> {
        return &self.requirements;
    }
}

/// A Stage is a convienient way to link a number of `Service`s. It can be thought
/// of as a `Service` that doesn't have a command, only dependencies
pub struct Stage {
    name: String,
    description: Option<String>,
    pub steps: Vec<Dependency>,
    state: TaskState,
}

impl From<StageDef> for Stage {
    fn from(item: StageDef) -> Self {
        return Stage {
            name: item.name.to_lowercase(),
            description: item.description,
            steps: item.steps.into_iter().map(|dep| Dependency::from(dep)).collect(),
            state: TaskState::Stopped
        };
    }
}

impl Task for Stage {
    fn get_name(&self) -> &String {
        return &self.name;
    }

    fn get_deps(&self) -> &Vec<Dependency> {
        return &self.steps;
    }
}