use std::fs::File;
use libq::logger;
use libq::strings::Tokenizer;
use libq::passwd::{GroupEntry, PasswdEntry, Groups};
use std::collections::HashMap;
use std::io::{BufReader, BufRead};
use std::path::PathBuf;

use nix::unistd::getuid;

/// An Enum representing all the Options that we know how
/// to handle. 
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum SudoersOptions {
    EchoPassword,
    AlwaysSetHome,
    MatchGroupByGid,
    EnvReset,
    EnvKeep,
    NoPasswd,
    SecurePath,
    AlwaysQueryGroupPlugin
}

/// An Enum representing the _value_ of a given option
/// in the configuration. 
#[derive(Debug, Clone)]
pub enum OptionsType {
    Flag(bool),
    Valued(bool, Vec<String>)
}

impl SudoersOptions {
    /// Constructs a Tuple of (type, value) from a given string, returning None if the
    /// Option isn't supported, or its value is incorrect, e.g. trying to append values to a flag
    fn from_sudoers_line(line: &[&str]) -> Option<(SudoersOptions, OptionsType)> {
        // Handle the flag and !flag case first
        let (arg, is_true) = {
            if line[0].starts_with("!") {
                (line[0].trim_start_matches("!"), false)
            }
            else {
                (line[0], true)
            }
        };

        let flag = match arg.to_lowercase().as_ref() {
            "visiblepw" => Some(SudoersOptions::EchoPassword),
            "always_set_home" => Some(SudoersOptions::AlwaysSetHome),
            "match_group_by_gid" => Some(SudoersOptions::MatchGroupByGid),
            "env_reset" => Some(SudoersOptions::EnvReset),
            "always_query_group_plugin" => Some(SudoersOptions::AlwaysQueryGroupPlugin),
            "nopasswd" => Some(SudoersOptions::NoPasswd),
            _ => None
        };

        if flag.is_some() {
            return Some((flag.unwrap(), OptionsType::Flag(is_true)));
        }

        // If it's not a known flag, it might be a value option, so we check it for assignments
        if line.len() < 3 || (line[1] != "=" && line[1] != "+=") {
            return None;
        }

        let append = line[1] == "+=";
        let values = line.iter().skip(2).map(|part| part.trim_start_matches("\"").trim_end_matches("\"")).map(|part| part.clone().to_owned()).collect::<Vec<String>>();

        return match arg {
            "env_keep" => Some((SudoersOptions::EnvKeep, OptionsType::Valued(append, values))),
            "secure_path" => Some((SudoersOptions::SecurePath, OptionsType::Valued(append, values))),
            _ => None
        };
    }
}

/// An Enum representing a user or group specification in a Sudoers file
#[derive(Debug, Clone)]
pub enum Identity {
    Uid(u32),
    User(String),
    Gid(u32),
    Group(String),
    All
}

impl PartialEq<Identity> for Identity {
    /// Defines the equality for an Identity. Two Identies are equal if they're the same type (e.g. both Uid/User or Gid/Group)
    /// and they resolve to the same ID. For String based Identities (User, Group), this means that we resolve them to an ID
    /// through /etc/passwd and /etc/group in this process
    fn eq(&self, other: &Identity) -> bool {
        let id_a = match self.normalize_to_id() {
            Some(Identity::All) => {
                return true;
            },
            Some(id) => id,
            None => {
                return false;
            }
        };

        let id_b = match other.normalize_to_id() {
            Some(Identity::All) => {
                return true;
            },
            Some(id) => id,
            None => {
                return false;
            }
        };

        if id_a.is_user() != id_b.is_user() {
            return false;
        }

        return match id_a {
            Identity::Uid(id_a) | Identity::Gid(id_a) => {
                match id_b {
                    Identity::Uid(id_b) | Identity::Gid(id_b) => id_a == id_b,
                    _ => false
                }
            },
            _ => false
        }
    }
}

impl Identity {
    /// Returns true if this is a User identity, i.e. User or UID
    fn is_user(&self) -> bool {
        return match self {
            Identity::Uid(_) | Identity::User(_) => true,
            _ => false
        }
    }

    /// Normalises this Identity to a u32 Identifier of the right type
    /// Nominally this means that Group becomes Gid and User becomes Uid
    /// Gid, Uid, and All Identities aren't changed. 
    /// This is done by reading the appropriate files (/etc/passwd, etc/group)
    /// So if this is a named entry, and an appropriate entry doesn't exist in those
    /// files, this returns None
    fn normalize_to_id(&self) -> Option<Identity> {
        return Some(match self {
            Identity::Uid(id) => Identity::Uid(*id),
            Identity::Gid(id) => Identity::Gid(*id),
            Identity::User(name) => {
                let id = match PasswdEntry::by_username(name) {
                    Some(passwd_entry) => passwd_entry.uid,
                    None => {
                        return None;
                    }
                };

                Identity::Uid(id)
            },
            Identity::Group(name) => {
                let id = match GroupEntry::by_groupname(name) {
                    Some(group) => group.gid,
                    None => {
                        return None;
                    }
                };

                Identity::Gid(id)
            },
            Identity::All => Identity::All
        });
    }

    /// Parses an Identity from a string, returning None
    /// if the string is malformed. This handles five different
    /// formats - "name", "#uid", "%groupname", "%#gid", and "ALL"
    pub fn from_str(s: &str) -> Option<Identity> {
        if s.starts_with("#") || s.starts_with("%#") {
            let id = match s {
                s if s.starts_with("#") => &s[1..],
                s if s.starts_with("%#") => &s[2..],
                s => s
            };

            let id = match id.parse() {
                Ok(id) => id,
                Err(_) => {
                    return None;
                }
            };

            if s.starts_with("#") {
                return Some(Identity::Uid(id));
            }
            else {
                return Some(Identity::Gid(id));
            }
        }
        else if s.starts_with("%") {
            return Some(Identity::Group(String::from(&s[1..])));
        }
        else {
            if s == "ALL" {
                return Some(Identity::All);
            }

            return Some(Identity::User(String::from(s)));
        }
    }
}

/// A struct representing a Command that a user might be allowed to run
/// Derived from the sudoers line "(user:group) /bin/foo bar"
/// which would mean "Can run /bin/foo with arg "bar" as user and group"
#[derive(Debug)]
struct Command {
    /// The User Identity that this command would be allowed to be run as
    as_user: Identity,

    /// The group Identity that this command would be allowed to be run as
    as_group: Identity,

    /// The nae of the command to be allowed to run (`argv[0]`)
    name: String,

    /// The arguments of the command to be allowed to run with. len() == 0 means any args, len() == 1 && args[0] == "" means no args
    args: Vec<String>
}

/// A struct representing a Permission granted in a sudoers file. Can be read as 
/// Users who match `id` can run `commands` on `machine` with `options
#[derive(Debug)]
pub struct Permission {
    /// The Identity that this Permission grants itself to
    id: Identity,

    /// The machine name that this matches. Traditionally `ALL`
    machine: String,

    /// The command that this permission allows the `id` to run
    commands: Vec<Command>,

    /// The options that are to be applied when this permission is used
    options: HashMap<SudoersOptions, OptionsType>,
}

impl Permission {
    pub fn is_allowed(&self, argv: &Vec<&str>, who: &Identity, as_user: &Identity, as_group: &Identity) -> bool {
        if &self.id != who {
            return false;
        }

        for command in self.commands.iter() {
            if &command.as_user == as_user && &command.as_group == as_group {
                if command.name == "ALL" {
                    return true;
                }
                else if command.name == argv[0] && command.args.len() == 0 {
                    return true;
                }
                else if command.name == argv[0] && command.args.len() == 2 && command.args[1] == "" && argv.len() == 1{
                    return true;
                }
                else if command.name == argv[0] && argv.len() == command.args.len() + 1 {
                    let mut matches = true;
                    for i in 1..argv.len() {
                        if command.args[i-1] != argv[i] {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    pub fn from_str(line: &str) -> Option<Permission> {
        let mut tokenizer = Tokenizer::new(line, vec!['\n', ';', '|', '=', ',', '(', ')', '&']);
        let tokens = match tokenizer.try_tokenize() {
            Ok(tokens) => tokens,
            Err(_) => {
                return None;
            }
        };

        let mut tokens_iter = tokens.into_iter().peekable();
        let user = Identity::from_str(&tokens_iter.next()?)?;
        let host = tokens_iter.next()?;
        if tokens_iter.next()? != "=" {
            return None;
        }

        let mut commands = Vec::new();
        let mut options = HashMap::new();
        while tokens_iter.peek().is_some() {
            if tokens_iter.next()? != "(" {
                println!("{:?}", tokens_iter.collect::<Vec<String>>());
                return None;
            }

            let user = Identity::from_str(&tokens_iter.next()?)?;
            let group;

            if tokens_iter.peek()? == ":" {
                tokens_iter.next()?;
                group = Identity::from_str(&tokens_iter.next()?)?;
            }
            else {
                group = Identity::from_str("ALL")?;
            }

            if tokens_iter.next()? != ")" {
                return None;
            }

            if tokens_iter.peek().is_some() && tokens_iter.peek()?.ends_with(":") {
                let option = tokens_iter.next()?;
                match SudoersOptions::from_sudoers_line(&[&option.trim_end_matches(":")[..]]) {
                    Some((option_type, option_value)) => {
                        options.insert(option_type, option_value);
                    },
                    None => {
                        return None;
                    }
                }
            }

            while tokens_iter.peek().is_some() && tokens_iter.peek()? != "(" {
                let command = tokens_iter.next()?;
                let mut args = Vec::new();
                while tokens_iter.peek().is_some() && tokens_iter.peek()? != "," && tokens_iter.peek()? != "(" {
                    args.push(tokens_iter.next()?);
                }

                if tokens_iter.peek().is_some() && tokens_iter.peek()? == "," {
                    tokens_iter.next(); // Skip the ,
                }
    
                commands.push(Command{
                    as_user: user.clone(),
                    as_group: group.clone(),
                    name: command,
                    args: args
                });
            }
        }

        if tokens_iter.peek().is_some() {
            return None;
        }

        return Some(Permission {
            id: user,
            machine: host,
            commands: commands,
            options: options
        });
    }
}

#[derive(Debug)]
pub struct Sudoers {
    defaults: HashMap<SudoersOptions, OptionsType>,
    permissions: Vec<Permission>
}

impl Sudoers {
    pub fn new() -> Sudoers {
        return Sudoers {
            defaults: HashMap::new(),
            permissions: Vec::new()
        };
    }

    pub fn is_allowed(&self, argv: Vec<&str>, as_user: Identity, as_group: Identity) -> (bool, Option<HashMap<SudoersOptions, OptionsType>>) {
        let mut options = self.defaults.clone();
        let mut identities = Vec::new();
        let user = PasswdEntry::by_uid(getuid().as_raw());
        if user.is_none() {
            return (false, None);
        }
        let user = user.unwrap();
        identities.push(Identity::Uid(user.uid));
        let groups = Groups::new().filter(|group| group.users.contains(&user.username)).map(|g| Identity::Gid(g.gid));
        identities.append(&mut groups.collect());

        for perm in self.permissions.iter() {
            for id in identities.iter() {
                if perm.is_allowed(&argv, id, &as_user, &as_group) {
                    options.extend(perm.options.clone());
                    return (true, Some(options));
                }
            }
        }

        return (false, None);
    }

    pub fn process_line(&mut self, line: &str) -> Result<(), ()>{
        if line.trim() == "" {
            return Ok(());
        }

        let parts = line.trim().split_whitespace().collect::<Vec<&str>>();
        if parts[0].starts_with("#") && !(parts[0] == "#include" || parts[0] == "#includedir" || (parts[0].len() > 1 && parts[0].chars().nth(1).unwrap().is_numeric())) {
            return Ok(()); // Comment
        }

        match parts[0] {
            "Defaults" => {
                let (flag_name, value) = match SudoersOptions::from_sudoers_line(&parts[1..]) {
                    Some(v) => v,
                    None => {
                        return Err(());
                    }
                };

                if let OptionsType::Flag(_) = value {
                    self.defaults.insert(flag_name, value);
                }
                else if let OptionsType::Valued(append, mut values) = value {
                    if self.defaults.contains_key(&flag_name) && append {
                        if let OptionsType::Valued(_, old_values) = self.defaults.get_mut(&flag_name).unwrap() {
                            old_values.append(&mut values);
                        }
                    }
                    else {
                        self.defaults.insert(flag_name, OptionsType::Valued(append, values));
                    }
                }
            },
            "#includedir" => {

            },
            _ => {
                let permission = match Permission::from_str(line) {
                    Some(perm) => perm,
                    None => {
                        return Err(());
                    }
                };
                self.permissions.push(permission);
            }
        }

        return Ok(());
    }

    pub fn read_from_disk() -> Option<Sudoers> {
        let logger = logger::with_name_as_console("sudo");

        let file = match File::open(PathBuf::from("/etc/sudoers")) {
            Ok(f) => f,
            Err(e) => {
                logger.info().with_string("error", e.to_string()).smsg("Failed to open /etc/sudoers");
                return None;
            }
        };

        let reader = BufReader::new(file);
        let mut config = Sudoers::new();

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    logger.info().with_string("error", e.to_string()).smsg("Failed to read /etc/sudoers");
                    return None;
                }
            };

            match config.process_line(&line) {
                Ok(()) => {},
                Err(()) => {
                    logger.info().msg(format!("Invalid configuration line: {}", line));
                }
            }
        }

        return Some(config);
    }
}
