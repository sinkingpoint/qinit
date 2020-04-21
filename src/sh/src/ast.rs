use shell;
use process;
use nix::unistd::{fork, ForkResult, Pid, tcsetpgrp};
use nix::sys::wait::{waitpid, WaitPidFlag};

pub trait ASTNode {
    fn execute(&self, &mut shell::Shell) -> i32;
    fn ingest_token(&mut self, &String);
}

pub struct ParseError {
    error: String,
    continuable: bool, // If an error is continuable, we will continue reading until we hit a valid and complete statement, or a non continable error
}

pub fn parse_into_ast(tokens: &Vec<String>, shell: &shell::Shell) -> Result<impl ASTNode, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError{
            error: "Empty Statement".to_string(),
            continuable: false,
        });
    }

    let mut pipeline: Vec<ProcessNode> = Vec::new();
    let mut current_process: ProcessNode = ProcessNode::new(shell.get_parent_pgid(), true);
    
    for token in tokens {
        match token.as_str() {
            "if" => {}, // TODO: If Statements
            "while" => {}, // TODO: While Statements 
            "&&" => {}, // TODO: And statements
            "||" => {}, // TODO: Or Statements
            "|" => {},  // TODO: Pipes
            "&" => {},  // TODO: Background processes
            _ => {
                current_process.ingest_token(token);
            },
        }
    }
    pipeline.push(current_process);
    return Ok(PipelineNode{
        processes: pipeline
    });
}

struct ProcessNode {
    process: process::Process,
    pgid: Pid,
    foreground: bool,
}

impl ProcessNode {
    fn new(pgid: Pid, foreground: bool) -> ProcessNode{
        return ProcessNode{
            process: process::Process::new(),
            pgid: pgid,
            foreground: foreground,
        };
    }

    fn is_builtin(&self, shell: &shell::Shell) -> bool {
        return shell.is_builtin(&self.process.proc_name)
    }
}

impl ASTNode for ProcessNode {
    fn execute(&self, shell: &mut shell::Shell) -> i32 {
        if self.is_builtin(shell) {
            return shell.run_builtin(&self.process.proc_name, &self.process.argv).unwrap();
        }
        else {
            // Launches a process using execve. Not responsible for forking or anything
            self.process.launch(shell, self.foreground);
            return 0; // process.launch execve's, so we shouldn't ever get here. 
        }
    }

    fn ingest_token(&mut self, token: &String) {
        match token.as_str() {
            "&" => {
                self.foreground = false
            },
            _ => {
                self.process.add_argv(&token);
            }
        }
    }
}

struct PipelineNode {
    processes: Vec<ProcessNode>
}

impl ASTNode for PipelineNode {
    fn execute(&self, shell: &mut shell::Shell) -> i32 {
        for process in &self.processes {
            if process.is_builtin(shell) {
                process.execute(shell);
                continue;
            }

            match fork() {
                Ok(ForkResult::Parent { child, .. }) => {
                    waitpid(child, Some(WaitPidFlag::WUNTRACED | WaitPidFlag::__WALL)).expect("Failed waiting for child");
                    tcsetpgrp(shell.terminal_fd, shell.parent_pgid);
                }
                Ok(ForkResult::Child) => {
                    process.execute(shell);
                },
                Err(_) => println!("Fork failed"),
            }
        }

        return 0;
    }

    fn ingest_token(&mut self, _token: &String) {
        
    }
}

