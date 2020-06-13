use serde_derive::Deserialize;

use serde::de::{self, Visitor, Deserializer, Deserialize};
use std::convert::TryInto;
use std::fmt;
use std::collections::HashMap;

use super::Identifier;

#[derive(Debug, Deserialize, PartialEq)]
pub enum RestartMode {
    Always,
    OnCrash,
    Never
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct ServiceDef {
    pub name: String,
    pub description: Option<String>,
    pub user: Option<Identifier>,
    pub group: Option<Identifier>,
    pub args: Option<Vec<String>>,
    pub restart: Option<RestartMode>,
    pub requirements: Option<Vec<DependencyDef>>, // A list of Units that should be started _before_ this one
    pub command: String
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct DependencyDef {
    pub name: String,
    pub args: Option<HashMap<String, String>>
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct StageDef {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<DependencyDef>
}

/// Struct used in serde to Deserialise user/group definitions
struct IdentifierVisitor;
impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return f.write_str("either a UID or a username");
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> where E: de::Error {
        return Ok(Identifier::ID(value.try_into().unwrap()));
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> where E: de::Error {
        if value < 0 {
            return Err(E::custom(format!("UID out of range: {}", value)))
        }
        return Ok(Identifier::ID(value.try_into().unwrap()));
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
        return Ok(Identifier::Name(value.to_owned()));
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Identifier, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_any(IdentifierVisitor)
    }
}