use super::registry::TaskRegistry;
use super::task::ServiceInstance;

use nix::unistd::Pid;

use std::collections::HashMap;

pub enum TaskState {
    Started(Option<Pid>),
    Failed(String),
    Starting,
    Stopped,
}

pub struct TaskStatusRegistry<'a> {
    task_registry: &'a TaskRegistry,
    statuses: HashMap<ServiceInstance, TaskState>
}

impl<'a> TaskStatusRegistry<'a> {
    pub fn new(registry: &'a TaskRegistry) -> TaskStatusRegistry<'a> {
        return TaskStatusRegistry {
            task_registry: registry,
            statuses: HashMap::new()
        }
    }

    pub fn execute_task(&mut self, name: &str, args: &HashMap<String, String>) -> bool {
        let task = match self.task_registry.get_task(name) {
            None => {
                return false;
            },
            Some(task) => task
        };

        return task.execute(args, self).is_ok();
    }

    pub fn set_status(&mut self, instance: ServiceInstance, status: TaskState) {
        self.statuses.insert(instance, status);
    }
}