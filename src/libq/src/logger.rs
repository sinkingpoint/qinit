use std::collections::HashMap;
use std::io::{self, Write};
use super::terminal::{TerminalColor, set_foreground_color, set_foreground_color_raw};

/// An enum representing the level of a log. 
/// Very opinionatedly we only offer two levels here, where a Debug log is a message to a developer
/// and an info log is a message to a user
#[derive(PartialEq)]
enum Level {
    /// The Debug level
    /// 
    /// Designates messages to a developer who needs to debug the application
    Debug,

    /// The Info Level
    /// 
    /// Designates messages to a user who is using the tool
    Info
}
impl Level {
    fn to_str(&self) -> &str {
        return match self {
            Level::Debug => "debug",
            Level::Info => "info"
        };
    }
}

/// A trait representing the ability to output a log record in some format
pub trait RecordWriter {
    /// Takes a record and writes it to `out` in some format. Optionally includes the logger name in the payload
    fn write_record<T>(&self, out: &mut T, logger_name: &str, record: &Record<Self>) -> Result<(), io::Error> where T: Write, Self: Sized;
}

/// A struct which represents a `RecordWriter` that outputs logs in ndjson format
/// i.e. logs are in JSON format, one per line
/// For any daemons, or non interactive software where humans aren't going to be directly reading the logs
/// this is the one to use
pub struct JSONRecordWriter{}
impl RecordWriter for JSONRecordWriter {
    fn write_record<T>(&self, out: &mut T, logger_name: &str, record: &Record<Self>) -> Result<(), io::Error> where T: Write, Self: Sized{
        out.write(&b"{"[..])?; // {
        for (key, value) in &record.kvs {
            out.write(&b"\""[..])?;
            out.write(key.as_bytes())?;
            out.write(&b"\": \""[..])?;
            out.write(value.as_bytes())?;
            out.write(&b"\", "[..])?;
        }
        
        out.write(&b"\"level\": \""[..])?;
        out.write(record.level.to_str().as_bytes())?;
        out.write(&b"\", "[..])?;
        out.write(&b"\"logger_name\": \""[..])?;
        out.write(logger_name.as_bytes())?;
        out.write(&b"\", "[..])?;
        out.write(&b"\"message\": \""[..])?;
        let message = record.message.as_ref().unwrap();
        out.write(&message.as_bytes())?;
        out.write(&b"\""[..])?;
        out.write(&b"}\n"[..])?; // }
        out.flush()?;
        return Ok(());
    }
}

/// A struct which represents a `RecordWriter` that outputs logs in a human readable format
/// This should be used for user facing applications where humans are expected to read the output
pub struct ConsoleRecordWriter{}
impl RecordWriter for ConsoleRecordWriter {
    fn write_record<T>(&self, out: &mut T, _logger_name: &str, record: &Record<Self>) -> Result<(), io::Error> where T: Write, Self: Sized{
        match record.level {
            Level::Info => {
                set_foreground_color(out, TerminalColor::Green)?;
                out.write(&b"INF "[..])?;
            },
            Level::Debug => {
                set_foreground_color(out, TerminalColor::Yellow)?;
                out.write(&b"DBG "[..])?;
            }
        }

        set_foreground_color(out, TerminalColor::BrightWhite)?;
        let message = record.message.as_ref().unwrap();
        out.write(&message.as_bytes())?;
        for (key, value) in &record.kvs {
            out.write(&b" "[..])?;
            set_foreground_color_raw(out, 160, 160, 160)?;
            out.write(key.as_bytes())?;
            out.write(&b"="[..])?;
            set_foreground_color(out, TerminalColor::BrightWhite)?;
            out.write(value.as_bytes())?;
            out.write(&b" "[..])?;
        }
        out.write(&b"\n"[..])?;

        set_foreground_color(out, TerminalColor::Reset)?;
        return Ok(());
    }
}

/// A struct which has the ability to create records and filter them by a given level. 
pub struct Logger<T> where T: RecordWriter{
    name: String,
    debug_enabled: bool,
    writer: T,
}

/// A struct which holds all the information of a log message to be outputted. 
pub struct Record<'a, T> where T: RecordWriter {
    logger: &'a Logger<T>,
    message: Option<String>,
    level: Level,
    kvs: HashMap<&'a str, String>
}

impl<'a, T> Record<'a, T> where T: RecordWriter{
    fn new(logger: &'a Logger<T>, level: Level) -> Record<'a, T>{
        return Record {
            logger: logger,
            message: None,
            level: level,
            kvs: HashMap::new()
        };
    }

    pub fn with_str(&'a mut self, key: &'a str, value: &'a str) -> &'a mut Record<T> {
        self.kvs.insert(key, value.to_owned());
        return self;
    }

    pub fn with_string(&'a mut self, key: &'a str, value: String) -> &'a mut Record<T> {
        self.kvs.insert(key, value);
        return self;
    }

    pub fn with_i8(&'a mut self, key: &'a str, value: i8) -> &'a mut Record<T> {
        return self.with_i64(key, value as i64);
    }

    pub fn with_i16(&'a mut self, key: &'a str, value: i16) -> &'a mut Record<T> {
        return self.with_i64(key, value as i64);
    }

    pub fn with_i32(&'a mut self, key: &'a str, value: i32) -> &'a mut Record<T> {
        return self.with_i64(key, value as i64);
    }

    pub fn with_i64(&'a mut self, key: &'a str, value: i64) -> &'a mut Record<T> {
        self.kvs.insert(key, value.to_string());
        return self;
    }

    pub fn with_u8(&'a mut self, key: &'a str, value: u8) -> &'a mut Record<T> {
        return self.with_u64(key, value as u64);
    }

    pub fn with_u16(&'a mut self, key: &'a str, value: u16) -> &'a mut Record<T> {
        return self.with_u64(key, value as u64);
    }

    pub fn with_u32(&'a mut self, key: &'a str, value: u32) -> &'a mut Record<T> {
        return self.with_u64(key, value as u64);
    }

    pub fn with_u64(&'a mut self, key: &'a str, value: u64) -> &'a mut Record<T> {
        self.kvs.insert(key, value.to_string());
        return self;
    }

    pub fn msg(&'a mut self, msg: String) {
        self.message = Some(msg);
        self.logger.write_record(self).expect("Failed to write log");
    }

    pub fn smsg(&'a mut self, msg: &str) {
        self.message = Some(msg.to_owned());
        self.logger.write_record(self).expect("Failed to write log");
    }
}

pub fn with_name_as_json(name: &str) -> Logger<JSONRecordWriter> {
    return with_name_and_format(name, JSONRecordWriter{});
}

pub fn with_name_as_console(name: &str) -> Logger<ConsoleRecordWriter> {
    return with_name_and_format(name, ConsoleRecordWriter{});
}

pub fn with_name_and_format<T>(name: &str, format: T) -> Logger<T> where T: RecordWriter{
    return Logger {
        name: name.to_owned(),
        debug_enabled: false,
        writer: format
    };
}

impl<T> Logger<T> where T: RecordWriter {
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    pub fn info<'a>(&'a self) -> Record<'a, T> {
        return Record::new(self, Level::Info);
    }

    pub fn debug<'a>(&'a self) -> Record<'a, T> {
        return Record::new(self, Level::Debug);
    }

    pub fn write_record(&self, record: &Record<T>) -> Result<(), io::Error> {
        if record.level == Level::Debug && !self.debug_enabled {
            return Ok(());
        }

        if record.level == Level::Debug {
            self.writer.write_record(&mut io::stderr(), &self.name, record)?;
        }
        else {
            self.writer.write_record(&mut io::stdout(), &self.name, record)?;
        }

        return Ok(());
    }
}