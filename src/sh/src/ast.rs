use shell;
use nix::unistd::Pid;

pub trait ASTNode {
    fn execute(&self, &mut shell::Shell, Option<Pid>, &shell::IOTriple) -> i32;
    fn ingest_token(&mut self, &String) -> bool;
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

pub fn parse_into_ast(tokens: &Vec<String>) -> Result<Box<dyn ASTNode>, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError{
            error: "Empty Statement".to_string(),
            continuable: false,
        });
    }

    let mut head = Box::new(ASTHead{
        nodes: Vec::new()
    });
    let mut current_node: Box<dyn ASTNode> = Box::new(PipelineNode::new());
    
    for token in tokens {
        match token.as_str() {
            "if" => {}, // TODO: If Statements
            "while" => {}, // TODO: While Statements 
            "&&" => {}, // TODO: And statements
            "||" => {}, // TODO: Or Statements
            _ => {
                if !current_node.ingest_token(&parse_out_quotes(token)) {
                    head.nodes.push(current_node);
                    current_node = Box::new(PipelineNode::new());
                }
            },
        }
    }

    head.nodes.push(current_node);

    return Ok(head);
}

struct ASTHead {
    nodes: Vec<Box<dyn ASTNode>>
}

impl ASTNode for ASTHead {
    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let mut exit_codes = Vec::new();
        for node in self.nodes.iter() {
            exit_codes.push(node.execute(shell, pgid, streams));
        }

        return match exit_codes.last() {
            None => 0,
            Some(code) => *code
        };
    }

    fn ingest_token(&mut self, _token: &String) -> bool {
        return false;
    }
}

struct OrNode {
    left_side: Box<dyn ASTNode>,
    right_side: Box<dyn ASTNode>
}

impl ASTNode for OrNode {
    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let left_status = self.left_side.execute(shell, pgid, streams);
        if left_status != 0 {
            return self.right_side.execute(shell, pgid, streams);
        }
        return left_status;
    }

    fn ingest_token(&mut self, _token: &String) -> bool {
        return true;
    }
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

    fn ingest_token(&mut self, token: &String) -> bool{
        match token.as_str() {
            "|" => {
                return false
            },
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
        return true;
    }
}

struct PipelineNode {
    processes: Vec<ProcessNode>,
    finished: bool,
}

impl PipelineNode {
    fn new() -> PipelineNode{
        return PipelineNode {
            processes: Vec::new(),
            finished: true
        };
    }
}

impl ASTNode for PipelineNode {
    fn execute(&self, shell: &mut shell::Shell, group: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let processes = self.processes.iter().map(|process| { process.to_process() }).collect();
        let mut job = shell::Job::new(processes);
        return job.execute(shell, group, streams);
    }

    fn ingest_token(&mut self, token: &String) -> bool {
        match token.as_str() {
            "\n" | ";" => {
                return false; // If we hit a ; or a \n, we're at the end of the whole pipeline
            },
            _ => {}
        }

        if self.finished {
            let mut node = ProcessNode::new();
            if node.ingest_token(token) {
                self.processes.push(node);
                self.finished = false;
                return true;
            }
        }
        else {
            self.finished = !self.processes.last_mut().unwrap().ingest_token(token);
        }

        return true;
    }
}

