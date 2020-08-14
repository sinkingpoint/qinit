use std::convert::TryInto;
use std::fmt;
use serde_derive::Deserialize;
use serde::de::{self, Deserialize, Visitor, Deserializer};
use std::collections::HashMap;

/// Represents a User/Group Identifier in a config file
/// Can either be the numeric (user/group) ID, or the name
/// Names that don't exist on the system are fine - they don't
/// get turned into IDs until they're used (e.g. the service gets started)
#[derive(Debug, PartialEq)]
pub enum Identifier {
    /// Represents a Numeric User or Group ID
    NumericID(u64),

    /// Represents a User/Group Name
    Name(String)
}

impl Default for Identifier {
    fn default() -> Self {
        // The default value of an Identifier is (Insecure, and) the root user/group
        return Identifier::NumericID(0);
    }
}

struct IdentifierVisitor;
impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return f.write_str("either a UID or a username");
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        return Ok(Identifier::NumericID(value.try_into().unwrap()));
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 {
            return Err(E::custom(format!("UID out of range: {}", value)));
        }
        return Ok(Identifier::NumericID(value.try_into().unwrap()));
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        return Ok(Identifier::Name(value.to_owned()));
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Identifier, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdentifierVisitor)
    }
}

/// The RestartMode of a Task or State defines what happens when a Task exits
#[derive(Debug, PartialEq, Deserialize)]
pub enum RestartMode {
    /// The Task should always restart, even if manually stopped. Should be used for services
    /// that should never exit (Critical daemons etc)
    Always,
    /// The Task should be restarted if it exists, no matter the exit code, unless it was manually
    /// stopped through QInit
    UnlessStopped,
    /// The Task should be restarted if it exits with an exit code != 0, unless it was manually
    /// stopped through QInit
    OnError,
    /// The Task should never be restarted, no matter how it exits
    Never
}

impl Default for RestartMode {
    fn default() -> Self {
        return RestartMode::OnError;
    }
}

/// A DependencyDef represents a dependency link between one sphere and another
#[derive(Debug, Deserialize, PartialEq)]
pub struct DependencyDef {
    /// The name of the Sphere that this dependency references
    pub name: String,

    /// The args of the Sphere. This has different effects, depending on whether the sphere
    /// is exclusive (Tasks) or Inclusive (Stages)
    pub args: HashMap<String, String>
}
