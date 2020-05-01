use std::io::Write;
use nix::sys::signal;
use std::collections::HashMap;
use builtins;
use std::os::unix::io::RawFd;
use std::ffi::{CStr, CString};
use nix::unistd::{fork, ForkResult, Pid, tcsetpgrp, tcgetpgrp, pipe, execvp, dup2, close, getpid, setpgid, getpgrp, isatty};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use libq::io::{STDIN_FD, STDOUT_FD, STDERR_FD};
use std::env;

pub struct IOTriple {
    /// An IOTriple represents the 3 standard IO streams of a Unix process
    stdin: RawFd,
    stdout: RawFd,
    stderr: RawFd,
}

impl IOTriple {
    /// Generates a default IOTriple, pointing to the system stdin, out and err */
    pub fn new() -> IOTriple {
        return IOTriple {
            stdin: STDIN_FD,
            stdout: STDOUT_FD,
            stderr: STDERR_FD
        };
    }
}

/// Represents a process in the shell, with the appropriate state 
/// Until the process has been started with `execute`, only the proc_name and argv
/// are valid
pub struct Process {
    /// The process name to run. Generally always argv[0]
    proc_name: String,
    /// The arguments to pass to the process
    argv: Vec<String>,
    /// An Option containing a Pid. Will exist if the process has been `execute`d
    pid: Option<Pid>,
    /// true if `execute` or `try_execute_as_builtin` has been called
    started: bool,
    /// True iff the process has been started, and has completed
    completed: bool,
    /// True iff the process has been started, but stopped and can be continued
    stopped: bool,
    /// An Option containing the exit code of the process, if it has exitted, or been stopped
    status: Option<i32>,
    /// Whether or not this process is in the foreground
    foreground: bool,
}

impl Process {
    /// Generates a new Process with the given proc_name and argv
    pub fn new(proc_name: String, argv: Vec<String>, foreground: bool) -> Process{
        return Process {
            proc_name: proc_name,
            argv: argv,
            pid: None,
            started: false,
            completed: false,
            stopped: false,
            status: None,
            foreground: foreground,
        };
    }

    /// Attempts to execute this process as a builtin (Which _should not_ be in a forked child)
    /// If the builtin exists, returns Ok(exit_code), otherwise, returns Err(())
    pub fn try_execute_as_builtin(&mut self, shell: &mut Shell, streams: &IOTriple) -> Result<i32, ()>{
        if shell.is_builtin(&self.proc_name) {
            let exit_code = shell.run_builtin(&self.proc_name, &self.argv, streams);
            self.status = Some(exit_code);
            self.started = true;
            self.completed = true;
            return Ok(exit_code);
        }

        return Err(());
    }

    fn cement_args(&mut self, shell: &Shell) -> Result<(), String>{
        for index in 0..self.argv.len() {
            self.argv[index] = do_string_interpolation(&self.argv[index], shell)?;
        }
        self.proc_name = self.argv[0].clone();
        return Ok(());
    }

    /// Executes this process, in the context of the given shell, in the given process group
    /// We expect a `fork` to have happened before this process runs so any persistance inside here
    /// doesn't actually work because we've diverged from the shell process (So any accounting _must_
    /// be done before we enter here)
    /// Because this process exevp's, or panics in the event that doesn't work, this function can actually
    /// never exit
    pub fn execute(&self, shell: &mut Shell, group: Option<Pid>, streams: &IOTriple) -> i32{
        if shell.is_interactive() {
            // If we're interactive, then we start a new process group (If one isn't given),
            // put ourselves into it, and take the foreground from the shell
            let pid = getpid();
            let pgid = match group {
                None => pid,
                Some(group_id) => group_id
            };
            setpgid(pid, pgid).expect("Failed to open a new process group");
            if self.foreground {
                tcsetpgrp(shell.terminal_fd, pid).expect("Failed to go into foreground");
            }

            // The shell ignores most of these signals, so here we restore the default handlers
            unsafe {
                signal::signal(signal::SIGINT, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGQUIT, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTSTP, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTTIN, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGTTOU, signal::SigHandler::SigDfl).unwrap();
                signal::signal(signal::SIGCHLD, signal::SigHandler::SigDfl).unwrap();
            }
        }

        // Duplicate the IO streams into the system standard paths
        // Note: We _must_ close the IOTriple streams here if they're non standard to avoid keeping pipes open
        // This means the IOTriple isn't valid after this execution
        if streams.stdin != STDIN_FD {
            dup2(streams.stdin, STDIN_FD).expect("Failed to duplicate input stream");
            close(streams.stdin).unwrap();
        }

        if streams.stdout != STDOUT_FD {
            dup2(streams.stdout, STDOUT_FD).expect("Failed to duplicate output stream");
            close(streams.stdout).unwrap();
        }

        if streams.stderr != STDERR_FD {
            dup2(streams.stderr, STDERR_FD).expect("Failed to duplicate error stream");
            close(streams.stderr).unwrap();
        }

        // Blastoff! Here we do some mangling to turn the UTF-8 Strings we have as args into CStrings
        let c_path = CString::new(self.proc_name.as_str()).unwrap();
        let cstr_argv: Vec<Vec<u8>> = self.argv.iter().map(|arg| CString::new(arg.as_str()).unwrap().into_bytes_with_nul()).collect();
        let argv = &cstr_argv.iter().map(|arg| CStr::from_bytes_with_nul(arg).unwrap()).collect::<Vec<&CStr>>()[..];
        match execvp(&c_path, argv) {
            Ok(_) => {
            },
            Err(e) => {
                panic!("Failed to exec: {}", e);
            }
        }
        return 0; // This just makes the compiler happy. We shouldn't ever get here
    }
}

struct VariableBuilder {
    build: String,
    in_braces: bool,
    done: bool,
}

impl VariableBuilder {
    fn new() -> VariableBuilder {
        return VariableBuilder{
            build: String::new(),
            in_braces: false,
            done: false
        }
    }

    fn ingest_char(&mut self, c: char) -> Result<(), String>{
        if self.done || 
           (c == '$' && self.build.len() > 0) || 
           (c == '?' && self.build.len() > 0) ||
           (c == '{' && self.build.len() > 0) ||
           (c == '}' && !self.in_braces) || 
           (c == '{' && self.in_braces){
            return Err(format!("Invalid char: {}", c));
        }

        if c == '{' {
            self.in_braces = true;
        }
        else if c == '}' {
            self.in_braces = false;
            self.done = true;
        }
        else if c == '$' || c == '?' {
            self.build.push(c);
            self.done = true;
        }
        else {
            self.build.push(c);
        }

        return Ok(());
    }

    fn could_be_done(&self) -> bool{
        return !self.in_braces || self.done
    }
}

fn do_string_interpolation(token: &String, shell: &Shell) -> Result<String, &'static str> {
    let mut build = String::new();
    let mut in_quotes_char = '\0';
    let mut var_build: Option<VariableBuilder> = None;
    for chr in token.chars() {
        if chr == '\'' || chr == '\"' {
            in_quotes_char = match in_quotes_char {
                '\0' => chr,
                _ if chr == in_quotes_char => '\0',
                _ => {
                    build.push(chr);
                    in_quotes_char
                }
            };
        }
        else if chr == '$' && in_quotes_char != '\'' && var_build.is_none(){
            var_build = Some(VariableBuilder::new());
        }
        else {
            match &mut var_build {
                Some(builder) => {
                    match builder.ingest_char(chr) {
                        Ok(()) => {},
                        Err(_err) => {
                            return Err("Substitution Error")
                        }
                    }

                    if builder.done {
                        build.push_str(&shell.get_variable(&builder.build));
                        var_build = None;
                    }
                },
                None => {
                    build.push(chr);
                }
            }
        }
    }

    match var_build {
        Some(builder) => {
            if builder.could_be_done() {
                build.push_str(&shell.get_variable(&builder.build));
            }
            else {
                return Err("Unclosed variable substitution");
            }
        },
        None => {}
    }

    return Ok(build);
}

pub struct Job {
    /// Represents a pipeline of processes where the stdout of each is `pipe`d into the stdin of the next

    /// pgid represents the process group that all the child processes of this pipeline are in. Only valid once we've started running it
    pgid: Option<Pid>,

    // A Vector of all the processes in this pipeline
    processes: Vec<Process>
}

impl Job {
    // Creates a new Job from the given processes, default the pgid to None as we haven't executed yet
    pub fn new(processes: Vec<Process>) -> Job {
        return Job{
            processes: processes,
            pgid: None
        };
    }

    // Waits for all the jobs in this pipeline to either stop, or complete
    fn wait(&mut self) {
        while {
            !self.handle_status_update(waitpid(None, Some(WaitPidFlag::__WALL | WaitPidFlag::WUNTRACED))) && 
            !self.is_stopped() && !self.is_completed()
        }{};
    }

    // Handles updates from `wait` events, doing some accounting around the child processes
    // Returns true if we've hit an irrecoverable error, or false if we've managed to successfully do the accounting
    fn handle_status_update(&mut self, status: nix::Result<WaitStatus>) -> bool{
        match status {
            Ok(event) => {
                match event {
                    WaitStatus::Exited(pid, code) => {
                        match self.find_process_by_pid(pid) {
                            Some(process) => {
                                process.completed = true;
                                process.status = Some(code);
                                return false;
                            }
                            None => {
                                eprintln!("Failed to find child by pid: {}", pid);
                                return true;
                            }
                        }
                    },
                    WaitStatus::Signaled(pid, signal, _) => {
                        match self.find_process_by_pid(pid) {
                            Some(process) => {
                                process.completed = true;
                                process.status = Some(signal as i32);
                                return false;
                            }
                            None => {
                                eprintln!("Failed to find child by pid: {}", pid);
                                return true;
                            }
                        }
                    },
                    WaitStatus::Stopped(pid, signal) => {
                        match self.find_process_by_pid(pid) {
                            Some(process) => {
                                process.stopped = true;
                                process.status = Some(signal as i32);
                                return false;
                            }
                            None => {
                                eprintln!("Failed to find child by pid: {}", pid);
                                return true;
                            }
                        }
                    },
                    _ => {
                        println!("Fallthrough");
                        return false;
                    }
                }
            },
            Err(err) => {
                eprintln!("Failed to wait: {}", err);
                return true;
            }
        }
    }

    // Searches the proceses in this Pipeline for one with the given pid
    fn find_process_by_pid(&mut self, pid: Pid) -> Option<&mut Process> {
        return self.processes.iter_mut().find(|process| {process.pid == Some(pid)});
    }

    // Returns true if all the processes in this pipeline are stopped or completed
    fn is_stopped(&self) -> bool{
        return self.processes.iter().all(|process| {!process.started || process.stopped || process.completed});
    }

    // Returns true if all the processes in this pipeline are completed
    fn is_completed(&self) -> bool {
        return self.processes.iter().all(|process| {!process.started || process.completed});
    }

    pub fn execute(&mut self, shell: &mut Shell, group: Option<Pid>, streams: &IOTriple) -> i32{
        if self.processes.len() == 0 {
            return 0; // Short circuit in the naive case where we have an empty Job
        }

        let mut iter = self.processes.iter_mut().peekable();
        let mut infile = streams.stdin;
        let mut outfile: RawFd;
        let mut pipe_source: RawFd;
        self.pgid = group;
        let mut broken = false;
        while let Some(process) = iter.next() {
            if broken {
                break;
            }
            if iter.peek().is_some() {
                match pipe() {
                    Ok((src, dest)) => {
                        pipe_source = src;
                        outfile = dest;
                    },
                    Err(err) => panic!("Failed to create pipe: {}", err)
                }
            }
            else {
                pipe_source = streams.stdin;
                outfile = streams.stdout;
            }

            match process.try_execute_as_builtin(shell, &IOTriple{
                stdin: infile,
                stdout: outfile,
                stderr: streams.stderr
            }) {
                Ok(_exit_code) => {},
                Err(()) => {
                    match process.cement_args(shell) {
                        Err(err) => {
                            eprintln!("{}", err);
                            broken = true;
                        }
                        Ok(()) => {
                            process.started = true;
                            match fork() {
                                Ok(ForkResult::Parent { child, .. }) => {
                                    if shell.is_interactive() {
                                        self.pgid = match self.pgid {
                                            None => Some(child),
                                            Some(_) => self.pgid
                                        };
                                        process.pid = Some(child);
                
                                        setpgid(child, self.pgid.unwrap()).expect("Failed to start new process group");
                                    }
                                }
                                Ok(ForkResult::Child) => {
                                    process.execute(shell, group, &IOTriple{
                                        stdin: infile,
                                        stdout: outfile,
                                        stderr: streams.stderr,
                                    });
                                },
                                Err(_) => {
                                    broken = true;
                                    process.started = false;
                                    eprintln!("Fork failed");
                                }
                            }
                        }
                    }
                
                    if infile != streams.stdin {
                        close(infile).unwrap();
                    }
        
                    if outfile != streams.stdout {
                        close(outfile).unwrap();
                    }
                }
            }

            infile = pipe_source;
        }

        if !shell.is_interactive() {
            self.wait();
        }
        else if self.processes[0].foreground {
            shell.put_job_in_foreground(self);
        }
        else {
            shell.put_job_in_background(self);
        }

        return self.processes[0].status.unwrap();
    }
}

pub struct Variable {
    /// Represents a variable in the shell, with a name and a value

    /// The name of the variable, without the $ set by e.g. `cats=value`
    name: String,

    /// The Value of the variable
    value: String,

    /// Whether this variable should be passed onto child processes
    environment: bool,
}

impl Variable {
    pub fn new(name: String, value: String, environment: bool) -> Variable{
        return Variable{
            name: name,
            value: value,
            environment: environment
        };
    }
}

pub struct Shell {
    is_repl: bool,
    is_interactive: bool,
    pub parent_pgid: Pid,
    pub terminal_fd: i32,
    builtins: HashMap<&'static str, builtins::Builtin>,
    exitcode: Option<u8>,
    variables: HashMap<String, Variable>
}

impl Shell {
    pub fn new(is_repl: bool) -> Shell {
        let is_interactive = match isatty(libq::io::STDIN_FD) {
            Ok(tty) => tty,
            Err(errno) => {
                panic!("STDIN is being weird: {}", errno);
            }
        };
    
        let my_pgid = getpgrp();
        if is_interactive {
            let mut fg_pgid = match tcgetpgrp(libq::io::STDIN_FD) {
                Ok(is_fg) => is_fg,
                Err(errno) => {
                    panic!("STDIN is being weird: {}", errno);
                }
            };
    
            while fg_pgid != my_pgid {
                signal::kill(my_pgid, signal::SIGTTIN).unwrap();
                fg_pgid = match tcgetpgrp(libq::io::STDIN_FD) {
                    Ok(is_fg) => is_fg,
                    Err(errno) => {
                        panic!("STDIN is being weird: {}", errno);
                    }
                };
            }
    
            unsafe {
                signal::signal(signal::SIGINT, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGQUIT, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTSTP, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTTIN, signal::SigHandler::SigIgn).unwrap();
                signal::signal(signal::SIGTTOU, signal::SigHandler::SigIgn).unwrap();
            }
    
            let my_pid = getpid();
            setpgid(my_pid, my_pid).expect("Failed to set PGID for shell");
            tcsetpgrp(libq::io::STDIN_FD, my_pid).expect("Failed to become the foreground process");
        }

        let mut variables_map = HashMap::new();
        for (key, value) in env::vars() {
            variables_map.insert(key.clone(), Variable{
                name: key.clone(),
                value: value,
                environment: true
            });
        }

        return Shell {
            is_repl: is_repl,
            is_interactive: is_interactive,
            parent_pgid: my_pgid,
            terminal_fd: libq::io::STDIN_FD,
            builtins: builtins::get_builtin_registry(),
            exitcode: None,
            variables: variables_map
        }
    }

    pub fn put_job_in_foreground(&self, job: &mut Job) {
        match job.pgid {
            // Some jobs don't have PGIDs (e.g. pipelines with builtins)
            Some(pgid) => {
                tcsetpgrp(self.terminal_fd, pgid).expect("Failed to put job into foreground");
                job.wait();
                tcsetpgrp(self.terminal_fd, self.parent_pgid).expect("Failed to get job control back from child process");
            },
            None => {}
        }
    }

    pub fn put_job_in_background(&self, _job: &Job) {
        
    }

    pub fn is_interactive(&self) -> bool {
        return self.is_interactive;
    }

    pub fn is_builtin(&self, name: &str) -> bool{
        return self.builtins.contains_key(name);
    }

    pub fn set_variable(&mut self, var: Variable) {
        if var.environment {
            env::set_var(var.name.clone(), var.value.clone());
        }
        self.variables.insert(var.name.clone(), var);
    }

    pub fn get_variable(&self, name: &str) -> &str {
        return match self.variables.get(name) {
            Some(var) => var.value.as_str(),
            None => ""
        };
    }

    pub fn run_builtin(&mut self, name: &str, argv: &Vec<String>, streams: &IOTriple) -> i32 {
        return self.builtins.get(name).unwrap()(self, argv, streams);
    }

    pub fn exit(&mut self, code: u8) {
        self.exitcode = Some(code);
    }

    pub fn write(&self, text: &str) {
        if self.is_interactive() && self.is_repl {
            print!("{}", text);
        }
        std::io::stdout().flush().expect("Failed writing to stdout");
    }

    pub fn has_exitted(&self) -> Option<u8> {
        return self.exitcode;
    }
}