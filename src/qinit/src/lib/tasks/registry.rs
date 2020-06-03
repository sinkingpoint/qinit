use super::task::{Stage, Service, Task};
use super::serde::{ServiceDef, StageDef};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Read};
use libq::logger::{self, Logger, JSONRecordWriter};

pub struct TaskRegistry {
    tasks: HashMap<String, Box<dyn Task>>,
    logger: Logger<JSONRecordWriter>
}

impl TaskRegistry {
    pub fn load_from_disk(task_def_folders: &Vec<PathBuf>) -> Result<TaskRegistry, io::Error>{
        let logger = {
            let mut l = logger::with_name_as_json("TaskRegistry");
            l.set_debug_mode(true);
            l
        };
        let mut tasks: HashMap<String, Box<dyn Task>> = HashMap::new();
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
                            if tasks.contains_key(&service.name) {
                                logger.info().msg(format!("Found duplicate defintion of {}. Skipping", &service.name));
                            }
                            tasks.insert(service.name.clone(), Box::new(Service::from(service)));
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
                            tasks.insert(stage.name.clone(), Box::new(Stage::from(stage)));
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

        let mut registry = TaskRegistry {
            tasks: tasks,
            logger: logger
        };
        
        registry.validate();

        return Ok(registry);
    }

    /// Goes through all the tasks in the registry and validates their dependencies, removing
    /// the tasks that have unknown ones
    fn validate(&mut self) {
        let mut to_remove = Vec::new();
        for (service_name, task) in self.tasks.iter() {
            for dep in task.get_deps().iter() {
                if !self.tasks.contains_key(dep) {
                    self.logger.info().msg(format!("Failed to load {} - dependency {} not found", service_name, dep));
                    to_remove.push(dep.clone());
                    break;
                }
            }
        }

        for service_name in to_remove.iter() {
            self.tasks.remove(service_name);
        }
    }

    pub fn len(&self) -> usize {
        return self.tasks.len();
    }
}
