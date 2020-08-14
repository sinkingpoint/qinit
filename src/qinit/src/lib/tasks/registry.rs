use super::serde::{TaskDef, Stage};
use std::fs::{File, read_dir};
use std::io::{self, Read};
use std::path::Path;
use libq::logger;

pub struct SphereRegistry {
    tasks: Vec<TaskDef>,
    stages: Vec<Stage>
}

fn read_from_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    return Ok(buffer);
}

impl SphereRegistry {
    /// Reads Sphere Definitions from all the directories in the given Vec
    pub fn read_from_disk(file_paths: Vec<&Path>) -> Result<SphereRegistry, io::Error>{
        let mut tasks = Vec::new();
        let mut stages = Vec::new();
        let logger = logger::with_name_as_json("sphere_registry");
        
        for dir in file_paths.iter() {
            if !dir.exists() || !dir.is_dir() {
                logger.info().with_string("path", dir.to_string_lossy().to_string()).smsg("Failed to read directory. Skipping it.");
                continue;
            }

            let iter = match read_dir(dir) {
                Ok(iter) => iter,
                Err(e) => {
                    logger.info().with_string("error", e.to_string()).with_string("path", dir.to_string_lossy().to_string()).smsg("Failed to read directory. Skipping it");
                    continue;
                }
            };

            for file in iter {
                let file = file?;
                let path = file.path();
                match path.extension() {
                    Some(ext) => {
                        match ext.to_str() {
                            Some("service") => {
                                match toml::from_str::<TaskDef>(&read_from_file(&path)?) {
                                    Ok(task) => {
                                        tasks.push(task);
                                    },
                                    Err(e) => {
                                        logger.info().with_string("error", e.to_string()).with_string("path", path.to_string_lossy().to_string()).smsg("Failed to read Task definition");
                                    }
                                }
                            },
                            Some("stage") => {
                                match toml::from_str::<Stage>(&read_from_file(&path)?) {
                                    Ok(stage) => {
                                        stages.push(stage);
                                    },
                                    Err(e) => {
                                        logger.info().with_string("error", e.to_string()).with_string("path", path.to_string_lossy().to_string()).smsg("Failed to read Stage definition");
                                    }
                                }
                            }
                            Some(_) | None => {
                                logger.info().with_string("path", path.to_string_lossy().to_string()).smsg("Unknown Sphere Type");
                            }
                        }
                    },
                    None => {
                        logger.info().with_string("path", path.to_string_lossy().to_string()).smsg("Unknown Sphere Type");
                    }
                }
            }
        }

        return Ok(SphereRegistry {
            tasks: tasks,
            stages: stages
        });
    }
}