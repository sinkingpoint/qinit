use super::serde::{TaskDef, Stage, DependencyDef};
use super::sphere::{SphereStatus, SphereType, SphereState, Startable};
use super::super::dtr::Graph;
use std::collections::HashMap;
use std::fs::{File, read_dir};
use std::io::{self, Read};
use std::path::Path;
use libq::logger;

pub struct SphereRegistry {
    /// The templates that can be used to create new spheres using `start`
    sphere_templates: HashMap<String, SphereType>,

    /// All the hard spheres that have been started at some point in the past
    running_spheres: HashMap<DependencyDef, SphereStatus>,

    /// All the hard spheres pending starting, based on some RunningSphere which has not yet `Started`
    pending_graph: Graph<DependencyDef, u32>
}

fn read_from_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    return Ok(buffer);
}

impl SphereRegistry {
    pub fn start(&mut self, sphere: DependencyDef) {
        match self.plan(sphere) {
            Some(plan) => {
                self.pending_graph.extend(plan);
                self.leaf_sweep();
            },
            None => {
                // We hit an error during planning. Bail
            }
        }
    }

    /// leaf_sweep is responsible for going through all the leaf nodes of the pending tree
    /// and starting them (If they're not already being started)
    fn leaf_sweep(&mut self) {
        let logger = logger::with_name_as_json("sphere_registry;leaf_sweep");
        for leaf in self.pending_graph.iter_leaves() {
            // If the leaf hasn't been marked as starting, then we need to start it
            if !self.running_spheres.contains_key(leaf) || self.running_spheres.get(leaf).unwrap().state != SphereState::Starting {
                // Start the leaf sphere!
                let sphere = match self.sphere_templates.get(&leaf.name.to_lowercase()) {
                    Some(sphere) => sphere,
                    None => {
                        logger.info().with_str("name", &leaf.name).with_map("args", &leaf.args.as_ref().unwrap_or(&HashMap::new())).smsg("Failed to find matching sphere");
                        continue;
                    }
                };

                if let Some(state) = sphere.start(self.running_spheres.get(leaf).and_then(|s| Some(&s.state))) {
                    logger.info().msg(format!("Moved {} into state {:?}", leaf.name, state));
                    self.running_spheres.insert(leaf.clone(), state);
                }
            }
        }
    }

    pub fn plan(&self, sphere: DependencyDef) -> Option<Graph<DependencyDef, u32>> {
        let logger = logger::with_name_as_json("sphere_registry;plan");
        let mut graph = Graph::new();
        let mut to_process = Vec::new();
        graph.add_node(sphere.clone());
        to_process.push(sphere);

        // ambiguities will store all the dependencies we encounter that don't describe a "hard" sphere
        // i.e. a dependency that doesn't specify all the arguments required by a sphere
        // We store these for later to avoid races where the hard sphere this "soft" sphere matches hasn't been encountered yet
        let mut ambiguities = Vec::new();

        while to_process.len() > 0 {
            let parent = to_process.pop().unwrap();
            logger.debug().with_str("name", &parent.name).smsg("Processing Dep");

            // Attempt to find the sphere that matches the given name
            let sphere = match self.sphere_templates.get(&parent.name.to_lowercase()) {
                Some(sphere) => sphere,
                None => {
                    logger.info().with_str("name", &parent.name).with_map("args", &parent.args.unwrap_or(HashMap::new())).smsg("Failed to find matching sphere");
                    return None;
                }
            };

            // If the dependency definition does not completely describe the sphere at this point,
            // then we're at an error - we have a node with no parent (and thus no "hard" spheres in this tree)
            // but is ambiguous. Thus we need to just bail because we have an unresolved ambiguity
            if !sphere.completely_describes(&parent) {
                return None;
            }

            let children = sphere.get_deps();

            if children.is_none() {
                // We've reached a leaf node, continue
                continue;
            }

            let children = children.unwrap();

            for child in children.iter() {
                // Merge the parents arguments and the childs arguments (This means that arguments can propagate down the dependency chain)
                let args = match (&parent.args, &child.args) {
                    (Some(parent_args), Some(child_args)) => Some(parent_args.iter().chain(child_args.iter()).map(|(k, v)| (k.clone(), v.clone())).collect()),
                    (Some(args), None) | (None, Some(args)) => Some(args.clone()),
                    (None, None) => None
                };

                // Resolve the child name to a child sphere
                let child_sphere = match self.sphere_templates.get(&child.name.to_lowercase()) {
                    Some(sphere) => sphere,
                    None => {
                        logger.info().with_str("name", &parent.name).with_map("args", &parent.args.unwrap_or(HashMap::new())).smsg("Failed to find matching sphere");
                        return None;
                    }
                };

                if !child_sphere.completely_describes(child) {
                    // There's ambiguity here that we need to resolve later - add it to the ambiguous list for now
                    ambiguities.push((parent.clone(), child));
                    continue;
                }

                let new_dep = DependencyDef::new(child.name.clone(), args);
                graph.add_node(new_dep.clone());
                match graph.add_edge(&parent, &new_dep, 1) {
                    Some(_) => {},
                    None => {
                        logger.debug().with_str("parent", &parent.name).with_str("child", &new_dep.name).smsg("Failed to add dependency");
                        return None;
                    }
                }
                logger.debug().with_str("parent", &parent.name).with_str("child", &new_dep.name).smsg("Adding Dependency");
                to_process.push(new_dep);
            }
        }

        // For all the ambiguities, we need to resolve them to hard spheres that have already been
        // created as dependencies of other hard spheres. Note that because these soft spheres are ambiguous by nature,
        // if multiple hard spheres match this soft sphere then we add the dependency as if it resolves to the first encountered
        // hard sphere
        for (parent, child) in ambiguities.iter() {
            let parent_index = graph.get_index_for_node(&parent)?;
            let child_index = match graph.find_node_index(|dep| child.partial_match(dep)) {
                Some(idx) => idx,
                None => {
                    logger.debug().with_str("name", &child.name).with_map("args", &child.args.as_ref().unwrap_or(&HashMap::new())).smsg("Failed to find matching sphere for ambiguous dependency");
                    return None;
                }
            };
            graph.add_edge_by_index(parent_index, child_index, 1);
        }

        return Some(graph);
    }

    /// Reads Sphere Definitions from all the directories in the given Vec
    pub fn read_from_disk(file_paths: Vec<&Path>) -> Result<SphereRegistry, io::Error>{
        let mut sphere_templates = HashMap::new();
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
                            Some("task") => {
                                match toml::from_str::<TaskDef>(&read_from_file(&path)?) {
                                    Ok(task) => {
                                        logger.debug().with_str("name", &task.name).smsg("Loaded Task");
                                        sphere_templates.insert(task.name.clone().to_lowercase(), SphereType::Task(task));
                                    },
                                    Err(e) => {
                                        logger.info().with_string("error", e.to_string()).with_string("path", path.to_string_lossy().to_string()).smsg("Failed to read Task definition");
                                    }
                                }
                            },
                            Some("stage") => {
                                match toml::from_str::<Stage>(&read_from_file(&path)?) {
                                    Ok(stage) => {
                                        logger.debug().with_str("name", &stage.name).smsg("Loaded Stage");
                                        sphere_templates.insert(stage.name.clone().to_lowercase(), SphereType::Stage(stage));
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
            sphere_templates: sphere_templates,
            pending_graph: Graph::new(),
            running_spheres: HashMap::new()
        });
    }
}