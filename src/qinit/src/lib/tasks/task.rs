use super::serde::{ServiceDef, StageDef};
use super::Identifier;
use std::convert::From;

#[derive(Debug)]
pub enum TaskState {
    Stopped,
    Started,
    Stopping,
    Starting,
    Failed
}

/// A trait that defines the minimum requirements for a task to be run by QInit
pub trait Task {
    fn get_name(&self) -> &String;
    fn get_deps(&self) -> &Vec<String>;
}

/// A struct that represents a service/daemon being run in the system
/// `Service`s each get their own CGroup and optionally have a list of services
/// that must have started successfully before this one
pub struct Service {
    name: String,
    description: Option<String>,
    user: Option<Identifier>,
    group: Option<Identifier>,
    requirements: Vec<String>, // A list of Units that should be started _before_ this one
    command: String,
    state: TaskState,
}

impl From<ServiceDef> for Service {
    fn from(item: ServiceDef) -> Self {
        return Service {
            name: item.name,
            description: item.description,
            user: item.user,
            group: item.group,
            requirements: match item.requirements {
                None => Vec::new(),
                Some(reqs) => reqs
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

    fn get_deps(&self) -> &Vec<String> {
        return &self.requirements;
    }
}

/// A Stage is a convienient way to link a number of `Service`s. It can be thought
/// of as a `Service` that doesn't have a command, only dependencies
pub struct Stage {
    name: String,
    description: Option<String>,
    pub steps: Vec<String>,
    state: TaskState,
}

impl From<StageDef> for Stage {
    fn from(item: StageDef) -> Self {
        return Stage {
            name: item.name,
            description: item.description,
            steps: item.steps,
            state: TaskState::Stopped
        };
    }
}

impl Task for Stage {
    fn get_name(&self) -> &String {
        return &self.name;
    }

    fn get_deps(&self) -> &Vec<String> {
        return &self.steps;
    }
}