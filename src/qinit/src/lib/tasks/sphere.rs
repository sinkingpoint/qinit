use tasks::serde::{TaskDef, DependencyDef, Stage};
use std::collections::HashMap;

/// SphereType is an enum containing a generic interface over all the different types
/// of sphere that exist. Why we do this over a trait and trait objects is a bit arbitrary
#[derive(Debug)]
pub enum SphereType {
    Task(TaskDef),
    Stage(Stage)
}

impl SphereType {
    pub fn get_deps(&self) -> Option<&Vec<DependencyDef>> {
        match self {
            SphereType::Task(def) => def.requires.as_ref(),
            SphereType::Stage(def) => Some(def.tasks.as_ref())
        }
    }

    /// Returns true if the given dependency def has all the args required
    /// for this sphere, and no more
    pub fn completely_describes(&self, dep: &DependencyDef) -> bool {
        match self {
            SphereType::Stage(def) => {
                // Stages have no args
                return dep.args == None;
            },
            SphereType::Task(def) => {
                match (&def.args, &dep.args) {
                    (None, None) => {
                        return true;
                    },
                    (Some(_), None) | (None, Some(_)) => {
                        return false;
                    },
                    (Some(self_args), Some(dep_args)) => {
                        for arg in self_args.iter() {
                            if !dep_args.contains_key(arg) {
                                return false;
                            }
                        }

                        return self_args.len() == dep_args.len();
                    }
                }
            }
        }
    }
}

pub struct RunningSphere {
    task_details: DependencyDef,
    state: SphereStatus
}

pub struct SphereStatus {
    state: SphereState,
    pid: Option<u32>
}

#[allow(dead_code)]
pub enum SphereState {
    Stopped,
    Stopping,
    Starting,
    Started,
    Exited(u32),
    NotStarted,
    FailedToStart
}
