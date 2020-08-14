use super::common::{Identifier, RestartMode, DependencyDef, StartConditions};
use serde_derive::Deserialize;

/// A Task Represents a Service that can be started, stopped, restarted and reloaded
#[derive(Debug, PartialEq, Deserialize)]
pub struct TaskDef {
    /// The name of the Task. This is what will be used when creating dependencies, or generally referring to this Task
    pub name: String,

    /// The description of this Task. Used to provide context in status information
    pub description: String,

    /// The user that processes under this task will run as
    pub user: Option<Identifier>,

    /// The group that processes under this task will run as
    pub group: Option<Identifier>,

    /// The command to be run _before_ starting this Task
    pub init_command: Option<String>,

    /// The command to be run in order to start this Task
    pub start_command: String,

    /// The command to be run in order to reload this Task (Defaults to a sighup to the primary process)
    pub reload_command: Option<String>,

    /// A list of valid argument names when starting this Task
    pub args: Option<Vec<String>>,

    /// How this process gets restarted
    pub restart_mode: Option<RestartMode>,

    /// The dependencies of this task. Tasks are inclusive Spheres - any non specified arguments
    /// default to allowing any value
    pub requires: Option<Vec<DependencyDef>>,

    /// The Conditions that must be met, after this Task is `exec`d, before it is considered "Started" 
    pub conditions: Option<StartConditions>
}

/// A Stage represents a collection of tasks that can be stopped, started, or restarted as a bundle
#[derive(Debug, PartialEq, Deserialize)]
pub struct Stage {
    /// The name of the Task. This is what will be used when creating dependencies, or generally referring to this Task
    pub name: String,

    /// The description of this Task. Used to provide context in status information
    pub description: String,
    
    /// The dependencies of this stage. Stages are exclusive spheres - if any arguments are not specified
    /// on any of the tasks then loading this Stage fails
    pub tasks: Vec<DependencyDef>
}
