use super::task::{Stage, Service, Task};
use super::serde::{ServiceDef, StageDef};
use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashMap;
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
            logger: logger
        });
    }

    pub fn len(&self) -> usize {
        return self.tasks.len();
    }

    pub fn get_task(&self, name: &str) -> Option<&Box<dyn Task>> {
        return self.tasks.get(name);
    }

    pub fn execute_task(&self, name: &str, args: &HashMap<String, String>) -> bool {
        let task = match self.tasks.get(name) {
            None => {
                return false;
            },
            Some(task) => task
        };

        let mut log = logger::with_name_as_json("TaskRegistry");
        log.info().with_string("task", name.to_string()).msg(format!("Starting task"));

        return task.execute(args, self).is_ok();
    }
}
