use shell;
use nix::unistd::Pid;

pub trait ASTNode {
    fn execute(&self, &mut shell::Shell, Option<Pid>, &shell::IOTriple) -> i32;
    fn ingest_token(&mut self, &String);
}

pub struct ParseError {
    pub error: String,
    pub continuable: bool, // If an error is continuable, we will continue reading until we hit a valid and complete statement, or a non continable error
}

/// Given a token, removes outer matching quote pairs from it
/// ## Examples:
/// | Token           | Return        |
/// |-----------------|---------------|
/// | "abcd"          | abcd          |
/// | "abcd'efg'"     | abcd'efg'     |
/// | dogs' and cats' | dogs and cats |
fn parse_out_quotes(token: &String) -> String {
    let mut build = String::new();
    let mut in_quotes_char = '\0';
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
        else {
            build.push(chr);
        }
    }
    return build;
}

pub fn parse_into_ast(tokens: &Vec<String>) -> Result<impl ASTNode, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError{
            error: "Empty Statement".to_string(),
            continuable: false,
        });
    }

    let mut pipeline: PipelineNode = PipelineNode::new();
    let mut current_process: ProcessNode = ProcessNode::new();
    
    for token in tokens {
        match token.as_str() {
            "if" => {}, // TODO: If Statements
            "while" => {}, // TODO: While Statements 
            "&&" => {}, // TODO: And statements
            "||" => {}, // TODO: Or Statements
            "|" => {
                pipeline.processes.push(current_process);
                current_process = ProcessNode::new();
            },
            "&" => {},  // TODO: Background processes
            _ => {
                current_process.ingest_token(&parse_out_quotes(token));
            },
        }
    }
    pipeline.processes.push(current_process);
    return Ok(pipeline);
}

struct ProcessNode {
    proc_name: String,
    argv: Vec<String>,
    foreground: bool,
}

impl ProcessNode {
    fn new() -> ProcessNode{
        return ProcessNode{
            proc_name: String::new(),
            argv: Vec::new(),
            foreground: true,
        };
    }

    fn to_process(&self) -> shell::Process {
        return shell::Process::new(self.proc_name.clone(), self.argv.iter().map(|a| { a.to_owned().clone() }).collect(), self.foreground);
    }
}

impl ASTNode for ProcessNode {
    fn execute(&self, shell: &mut shell::Shell, group: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let process = self.to_process();
        return process.execute(shell, group, streams);
    }

    fn ingest_token(&mut self, token: &String) {
        match token.as_str() {
            "&" => {
                self.foreground = false
            },
            _ => {
                if self.proc_name == "" {
                    self.proc_name = token.clone();
                }
                self.argv.push(token.clone());
            }
        }
    }
}

struct PipelineNode {
    processes: Vec<ProcessNode>,
}

impl PipelineNode {
    fn new() -> PipelineNode{
        return PipelineNode {
            processes: Vec::new(),
        };
    }
}

impl ASTNode for PipelineNode {
    fn execute(&self, shell: &mut shell::Shell, group: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let processes = self.processes.iter().map(|process| { process.to_process() }).collect();
        let mut job = shell::Job::new(processes);
        return job.execute(shell, group, streams);
    }

    fn ingest_token(&mut self, _token: &String) {
        
    }
}

