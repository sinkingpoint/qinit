use super::serde::ServiceDef;
use super::Identifier;
use std::convert::From;

#[derive(Debug)]
pub enum TaskState {
    Stopped,
    Started,
    Stopping,
    Stating
}

pub struct Service {
    name: String,
    description: Option<String>,
    user: Option<Identifier>,
    group: Option<Identifier>,
    pub requirements: Vec<String>, // A list of Units that should be started _before_ this one
    command: String
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
            command: item.command
        }
    }
}
