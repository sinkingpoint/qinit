use super::task::{Stage, Service, Task, ServiceInstance, TaskStatus};
use super::serde::{ServiceDef, StageDef, RestartMode};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Read};
use libq::logger::{self, Logger, JSONRecordWriter};

use nix::unistd::Pid;

pub struct TaskRegistry {
    tasks: HashMap<String, Box<dyn Task + Send>>,
    logger: Logger<JSONRecordWriter>,
    statuses: HashMap<ServiceInstance, TaskStatus>
}

impl TaskRegistry {
    pub fn load_from_disk(task_def_folders: &Vec<PathBuf>) -> Result<TaskRegistry, io::Error>{
        let logger = {
            let mut l = logger::with_name_as_json("TaskRegistry");
            l.set_debug_mode(true);
            l
        };

        let mut tasks: HashMap<String, Box<dyn Task + Send>> = HashMap::new();
        for loc in task_def_folders.iter() {
            if !loc.exists() || !loc.is_dir() {
                logger.debug().msg(format!("{} either doesn't exist or isn't a directory. Either way, skipping it for task definitions", loc.display()));
                continue;
            }

            logger.debug().msg(format!("Checking {} for tasks", loc.display()));

            for child in loc.read_dir()? {
                if child.is_err() {
                    continue;
                }
    
                let child = child.unwrap();
                let path = child.path();
                let mut contents = String::new();
                let mut file = File::open(child.path())?;
                file.read_to_string(&mut contents)?;

                let extension = path.extension().unwrap().to_string_lossy();

                if extension == "service" {
                    match toml::from_str::<ServiceDef>(&contents) {
                        Ok(service) => {
                            println!("{:?}", service);
                            if tasks.contains_key(&service.name) {
                                logger.info().msg(format!("Found duplicate defintion of {}. Skipping", &service.name));
                            }
                            let service = Service::from(service);
                            tasks.insert(service.get_name().clone(), Box::new(service));
                        },
                        Err(err) => {
                            logger.info().with_string("error", format!("{}", err)).msg(format!("Failed to load task definition {} as service definition", child.path().display()));
                        }
                    }
                }
                else if extension == "stage" {
                    match toml::from_str::<StageDef>(&contents) {
                        Ok(stage) => {
                            if tasks.contains_key(&stage.name) {
                                logger.info().msg(format!("Found duplicate defintion of {}. Skipping", &stage.name));
                            }
                            let stage = Stage::from(stage);
                            tasks.insert(stage.get_name().clone(), Box::new(stage));
                        },
                        Err(err) => {
                            logger.info().with_string("error", format!("{}", err)).msg(format!("Failed to load task definition {} as service definition", child.path().display()));
                        }
                    }
                }
                else {
                    logger.info().msg(format!("Encountered unknown task type: {}", extension));
                }
            }
        }

        return Ok(TaskRegistry {
            tasks: tasks,
            logger: logger,
            statuses: HashMap::new()
        });
    }

    pub fn len(&self) -> usize {
        return self.tasks.len();
    }

    pub fn get_task(&self, name: &str) -> Option<&Box<dyn Task + Send>> {
        return self.tasks.get(name);
    }

    fn execute_task_dep_tree(&self, service: &ServiceInstance) -> Result<Vec<(ServiceInstance, TaskStatus)>, ()> {
        let mut to_start = Vec::new();
        let mut to_process_deps = Vec::new();
        let mut new_states = Vec::new();

        to_process_deps.push(service);

        while to_process_deps.len() > 0 {
            let new_dep = to_process_deps.pop().unwrap();
            match self.get_task(new_dep.get_name()) {
                Some(task) => {
                    // TODO: Deduplication, failing for circular deps
                    to_start.push((task, new_dep.get_args(), new_dep.clone()));

                    for dep in task.get_deps() {
                        to_process_deps.push(dep);
                    }
                },
                None => {
                    return Err(());
                }
            }
        }

        to_start.reverse();

        for (task, args, instance) in to_start.into_iter() {
            if let Some(TaskStatus::Running(_)) = self.statuses.get(&instance) {
                continue;
            }
            let result = task.execute(args);
            new_states.push((instance, result));
        }

        return Ok(new_states);
    }

    pub fn execute_task(&mut self, name: &str, args: &HashMap<String, String>) -> Result<(), ()> {
        match self.execute_task_dep_tree(&ServiceInstance::new(name.to_owned(), args.clone())) {
            Ok(statuses) => {
                for (instance, state) in statuses.into_iter() {
                    self.statuses.insert(instance, state);
                }
            },
            Err(()) => {
                // TODO: Stop all started deps
                return Err(());
            }
        }

        return Ok(());
    }

    pub fn set_status_with_pid(&mut self, child_pid: Pid, new_state: TaskStatus) {
        let mut found_data = None;
        for (instance, state) in self.statuses.iter_mut() {
            if let TaskStatus::Running(Some(test_pid)) = state {
                if  test_pid == &child_pid {
                    *state = new_state.clone();
                    found_data = Some((instance.get_name().clone(), instance.get_args().clone()));
                    break;
                }
            }
        }

        // TODO: This is real hacky and does a couple of extra moves than necessary
        // We should rewrite this

        if found_data.is_none() {
            return;
        }

        let (name, args) = found_data.unwrap();

        let restart_mode = self.get_task(&name).unwrap().get_restart_mode();

        if restart_mode.is_some() && restart_mode.unwrap() != &RestartMode::Never {
            match new_state {
                TaskStatus::Stopped(_) | TaskStatus::Failed => {
                    match self.execute_task(&name, &args) {
                        Ok(_) => {},
                        Err(_) => {}
                    }
                },
                _ => {}
            }
        }
    }
}
