use super::serde::{ServiceDef, StageDef, DependencyDef, RestartMode};
use super::status_registry::{TaskStatusRegistry, TaskState};
use super::Identifier;
use std::convert::From;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::ffi::{CStr, CString};

use nix::unistd::{fork, ForkResult, execv};
use nix::errno::Errno;

use libq::logger;

pub struct ServiceInstance {
    name: String,
    args: HashMap<String, String>
}

impl ServiceInstance {
    pub fn new(name: String, args: HashMap<String, String>) -> ServiceInstance {
        return ServiceInstance {
            name: name,
            args: args
        };
    }

    pub fn get_name(&self) -> &String {
        return &self.name;
    }

    pub fn get_args(&self) -> &HashMap<String, String> {
        return &self.args;
    }
}

impl Hash for ServiceInstance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (key, value) in self.args.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl fmt::Display for ServiceInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut arg_string = String::new();
        for (key, value) in self.args.iter() {
            arg_string.push_str(format!("{}={}", key, value).as_str());
        }
        write!(f, "Depency{{ {} {} }}", self.name, arg_string)?;
        return Ok(());
    }
}

impl From<DependencyDef> for ServiceInstance {
    fn from(item: DependencyDef) -> Self {
        return ServiceInstance {
            name: item.name.to_lowercase(),
            args: match item.args {
                None => HashMap::new(),
                Some(a) => a
            }
        }
    }
}

impl PartialEq for ServiceInstance {
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

impl Eq for ServiceInstance{}

/// A trait that defines the minimum requirements for a task to be run by QInit
pub trait Task {
    fn get_name(&self) -> &String;
    fn get_deps(&self) -> &Vec<ServiceInstance>;
    fn execute(&self, &HashMap<String, String>, &mut TaskStatusRegistry) -> Result<(), ()>;
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
    requirements: Vec<ServiceInstance>, // A list of Units that should be started _before_ this one
    command: String,
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
                Some(reqs) => reqs.into_iter().map(|dep| ServiceInstance::from(dep)).collect()
            },
            command: item.command
        }
    }
}

impl Task for Service {
    fn get_name(&self) -> &String {
        return &self.name;
    }

    fn get_deps(&self) -> &Vec<ServiceInstance> {
        return &self.requirements;
    }

    fn execute(&self, args: &HashMap<String, String>, registry: &mut TaskStatusRegistry) -> Result<(), ()> {
        // Validate args
        if args.len() != self.args.len() {
            // Args are incomplete
                return Err(());
        }

        for arg in self.args.iter() {
            if !args.contains_key(arg) {
                // Arg is missing
                return Err(());
            }
        }

        // TODO: Do templating on command line string, split into argv, fork and record state
        let mut replaced_command = self.command.clone();
        for (key, value) in args.iter() {
            let template = format!("${{{}}}", key);
            replaced_command = replaced_command.replace(template.as_str(), value.as_str());
        }
        let logger = logger::with_name_as_json("qinit;task");

        let argv: Vec<&str> = replaced_command.split_whitespace().collect();
        let argv: Vec<Vec<u8>> = argv.into_iter().map(|arg| CString::new(arg).unwrap().into_bytes_with_nul()).collect();
        let argv = &argv.iter().map(|arg| CStr::from_bytes_with_nul(arg).unwrap()).collect::<Vec<&CStr>>()[..];

        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                registry.set_status(ServiceInstance::new(self.name.clone(), args.clone()), TaskState::Started(Some(child)));
            },
            Ok(ForkResult::Child) => {
                match execv(argv[0], argv) {
                    Ok(_) => {} // We should never get here. A sucessful execvp will never get here as it will be running the other program
                    Err(err) => {
                        if let Some(errno) = err.as_errno() {
                            if errno == Errno::ENOENT {
                                std::process::exit(127);
                            }
                        }
                        else {
                            logger.debug().with_string("error", err.to_string()).smsg("Failed to exec process");
                        }
                    }
                }
            },
            Err(_) => {
                eprintln!("Fork failed");
            }
        }
        return Err(());
    }
}

/// A Stage is a convienient way to link a number of `Service`s. It can be thought
/// of as a `Service` that doesn't have a command, only dependencies
pub struct Stage {
    name: String,
    description: Option<String>,
    steps: Vec<ServiceInstance>,
}

impl From<StageDef> for Stage {
    fn from(item: StageDef) -> Self {
        return Stage {
            name: item.name.to_lowercase(),
            description: item.description,
            steps: item.steps.into_iter().map(|dep| ServiceInstance::from(dep)).collect(),
        };
    }
}

impl Task for Stage {
    fn get_name(&self) -> &String {
        return &self.name;
    }

    fn get_deps(&self) -> &Vec<ServiceInstance> {
        return &self.steps;
    }

    fn execute(&self, args: &HashMap<String, String>, registry: &mut TaskStatusRegistry) -> Result<(), ()> {
        for step in self.steps.iter() {
            if !registry.execute_task(step.get_name(), step.get_args()) {
                registry.set_status(ServiceInstance::new(self.name.clone(), args.clone()), TaskState::Failed(format!("Failed to start {}", step.get_name())));
                return Err(());
            }
        }

        registry.set_status(ServiceInstance::new(self.name.clone(), args.clone()), TaskState::Started(None));
        
        return Ok(());
    }
}