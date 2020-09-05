use std::fs::{create_dir, File};
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process;

pub enum PidFileError {
    IOError(io::Error),
    ProcessAlreadyRunning,
}

pub fn write_pid_file(path: &Path) -> Result<(), PidFileError> {
    // TODO: I think there's a TOCTOU bug here, where two processes can skip this if statement
    // and then go on and try and create the PID file. Need some form of locking between the processes
    if path.exists() {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return Err(PidFileError::IOError(err));
            }
        };

        let buf_reader = BufReader::new(file);
        match buf_reader.lines().next() {
            None => {}
            Some(pid) => {
                let proc_pathbuf: PathBuf = match pid {
                    Err(err) => {
                        return Err(PidFileError::IOError(err));
                    }
                    Ok(pid) => ["/proc", pid.trim()].iter().collect(),
                };

                if proc_pathbuf.exists() {
                    // The PID file points to a still running service
                    return Err(PidFileError::ProcessAlreadyRunning);
                }
            }
        };
    }

    if let Some(parent_dir) = &path.parent() {
        if !parent_dir.exists() || !parent_dir.is_dir() {
            match create_dir(parent_dir) {
                Ok(_) => {}
                Err(err) => {
                    return Err(PidFileError::IOError(err));
                }
            }
        }
    }

    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(err) => {
            return Err(PidFileError::IOError(err));
        }
    };

    match file.write_all(&format!("{}", process::id()).into_bytes()[..]) {
        Ok(_) => {}
        Err(err) => {
            return Err(PidFileError::IOError(err));
        }
    }

    return Ok(());
}
