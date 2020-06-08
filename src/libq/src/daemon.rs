use std::fs::{create_dir, File};
use std::io::{BufReader,BufRead};
use std::path::PathBuf;
use std::process;
use std::io::Write;

pub fn write_pid_file(path: PathBuf) -> Result<(), String> {
    if path.exists() {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                return Err(format!("Failed to open pid file `{}`: {}", path.display(), err));
            }
        };

        let buf_reader = BufReader::new(file);
        match buf_reader.lines().next() {
            None => {},
            Some(pid) => {
                let proc_pathbuf: PathBuf = match pid {
                    Err(err) => {
                        return Err(format!("Failed to read from pidfile: {}", err));
                    },
                    Ok(pid) => ["/proc", pid.trim()].iter().collect()
                };

                if proc_pathbuf.exists() {
                    // The PID file points to a still running service
                    return Err(format!("Previous process is still running"));
                }
            }
        };
    }

    if let Some(parent_dir) = &path.parent() {
        if !parent_dir.exists() || !parent_dir.is_dir() {
            if create_dir(parent_dir).is_err() {
                return Err(format!("Failed to create directory {}", parent_dir.display()));
            }
        }
    }
    
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(err) => {
            return Err(format!("Failed to open pid file `{}`: {}", path.display(), err));
        }
    };

    if file.write_all(&format!("{}", process::id()).into_bytes()[..]).is_err() {
        return Err(format!("Failed writing PID to {}", &path.display()));
    }

    return Ok(());
}