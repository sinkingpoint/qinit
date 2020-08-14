use super::serde::TaskDef;
use super::serde::{Identifier, RestartMode, DependencyDef};

struct Task {
    /// The name of the Task. This is what will be used when creating dependencies, or generally referring to this Task
    pub name: String,

    /// The description of this Task. Used to provide context in status information
    pub description: String,

    /// The user that processes under this task will run as
    pub user: Identifier,

    /// The group that processes under this task will run as
    pub group: Identifier,

    /// The command to be run _before_ starting this Task
    pub init_command: Option<String>,

    /// The command to be run in order to start this Task
    pub start_command: String,

    /// The command to be run in order to reload this Task (Defaults to a sighup to the primary process)
    pub reload_command: Option<String>,

    /// A list of valid argument names when starting this Task
    pub args: Vec<String>,

    /// How this process gets restarted
    pub restart_mode: RestartMode,

    /// The dependencies of this task. Tasks are inclusive Spheres - any non specified arguments
    /// default to allowing any value
    pub requires: Vec<DependencyDef>
}