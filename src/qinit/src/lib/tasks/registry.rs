use super::task::Service;
use super::serde::ServiceDef;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::io::{self, Read};
use libq::logger::{self, Logger, JSONRecordWriter};

pub struct TaskRegistry {
    services: HashMap<String, Service>,
    logger: Logger<JSONRecordWriter>
}

impl TaskRegistry {
    pub fn load_from_disk(task_def_folders: &Vec<PathBuf>) -> Result<TaskRegistry, io::Error>{
        let logger = {
            let mut l = logger::with_name_as_json("TaskRegistry");
            l.set_debug_mode(true);
            l
        };
        let mut services = HashMap::new();
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
                let mut contents = String::new();
                let mut file = File::open(child.path())?;
                file.read_to_string(&mut contents)?;
    
                match toml::from_str::<ServiceDef>(&contents) {
                    Ok(service) => {
                        if services.contains_key(&service.name) {
                            logger.info().msg(format!("Found duplicate defintion of {}. Skipping", &service.name));
                        }
                        services.insert(service.name.clone(), Service::from(service));
                    },
                    Err(err) => {
                        logger.info().with_string("error", format!("{}", err)).msg(format!("Failed to load task definition {} as service definition", child.path().display()));
                    }
                }
            }
        }

        let mut registry = TaskRegistry {
            services: services,
            logger: logger
        };
        
        registry.validate();

        return Ok(registry);
    }

    /// Goes through all the tasks in the registry and validates their dependencies, removing
    /// the tasks that have unknown ones
    fn validate(&mut self) {
        let mut to_remove = Vec::new();
        let services = &mut self.services;
        for (service_name, service) in services.iter() {
            for dep in service.requirements.iter() {
                if !services.contains_key(dep) {
                    self.logger.info().msg(format!("Failed to load {} - dependency {} not found", service_name, dep));
                    to_remove.push(dep.clone());
                    break;
                }
            }
        }

        for service_name in to_remove.iter() {
            services.remove(service_name);
        }
    }

    pub fn len(&self) -> usize {
        return self.services.len();
    }
}
