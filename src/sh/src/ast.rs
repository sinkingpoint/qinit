use shell;
use nix::unistd::Pid;
use std::fmt;

pub trait ASTNode: fmt::Display{
    fn execute(&self, &mut shell::Shell, Option<Pid>, &shell::IOTriple) -> i32;
    fn ingest_token(&mut self, &String) -> Result<bool, ParseError>;
    fn is_complete(&self) -> bool;
    fn ingest_node(&mut self, Box<dyn ASTNode>) -> Result<bool, ParseError>;
}

pub struct ParseError {
    pub error: String,
    pub continuable: bool, // If an error is continuable, we will continue reading until we hit a valid and complete statement, or a non continuable error
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

    let mut head: Box<dyn ASTNode> = Box::new(ASTHead::new());
    let mut current_node: Box<dyn ASTNode> = match tokens.first() {
        Some(_) => Box::new(PipelineNode::new()),
        None => Box::new(PipelineNode::new())
    };
    let mut current_block: Box<dyn ASTNode> = Box::new(ASTHead::new());
    let mut current_block_ref: &mut Box<dyn ASTNode> = &mut head;
    
    for token in tokens {
        match token.as_str() {
            "if" if current_node.as_ref().is_complete() => {
                current_block = Box::new(IfNode::new()) as Box<dyn ASTNode>;
                current_block_ref = &mut current_block;
            }, // TODO: If Statements
            "while" if current_node.as_ref().is_complete() => {}, // TODO: While Statements
            "then" | "fi" | "do" | "done" => {
                match current_block_ref.ingest_token(token) {
                    Ok(_) => {},
                    Err(err) if err.continuable => {
                        current_block_ref = &mut head;
                    },
                    Err(err) => {
                        return Err(err);
                    }
                }

                if current_block_ref.is_complete() {
                    head.ingest_node(current_block)?;
                    current_block = match tokens.first() {
                        Some(_) => Box::new(PipelineNode::new()),
                        None => Box::new(PipelineNode::new())
                    };
                    current_block_ref = &mut head;
                }

                current_node = Box::new(PipelineNode::new());
            },
            "&&" => {}, // TODO: And statements
            "||" => {}, // TODO: Or Statements
            _ => {
                match current_node.ingest_token(&parse_out_quotes(token)) {
                    Ok(_) => {},
                    Err(err) => {
                        if err.continuable {
                            current_block_ref.ingest_node(current_node)?;
                            current_node = Box::new(PipelineNode::new());
                        }
                        else {
                            return Err(err);
                        }
                    }
                }
            },
        }
    }

    match current_block_ref.ingest_node(current_node) {
        Ok(_) => {},
        Err(err) => {
            return Err(err);
        }
    }

    return Ok(head);
}

struct ASTHead {
    nodes: Vec<Box<dyn ASTNode>>
}

impl ASTHead {
    fn new() -> ASTHead {
        return ASTHead{
            nodes: Vec::new()
        };
    }
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

    fn ingest_token(&mut self, _token: &String) -> Result<bool, ParseError> {
        return Ok(false);
    }

    fn is_complete(&self) -> bool {
        return false;
    }

    fn ingest_node(&mut self, node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        self.nodes.push(node);
        return Ok(true);
    }
}

impl fmt::Display for ASTHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.nodes.iter() {
            match write!(f, "- {}", node) {
                Ok(_) => {},
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(());
    }
}

#[derive(Debug)]
enum ConditionalBuildState {
    Condition,
    Body,
    Done
}

struct IfNode {
    state: ConditionalBuildState,
    condition: Vec<Box<dyn ASTNode>>,
    body: Vec<Box<dyn ASTNode>>
}

impl IfNode {
    fn new() -> IfNode {
        return IfNode {
            state: ConditionalBuildState::Condition,
            condition: Vec::new(),
            body: Vec::new(),
        };
    }
}

impl ASTNode for IfNode {
    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let mut exit = 0;
        for node in &self.condition {
            exit = node.execute(shell, pgid, streams);
        }

        if exit == 0 {
            for node in &self.body {
                exit = node.execute(shell, pgid, streams);
            }
        }
        return exit;
    }

    fn ingest_token(&mut self, token: &String) -> Result<bool, ParseError> {
        self.state = match self.state {
            ConditionalBuildState::Condition if token == "then" => ConditionalBuildState::Body,
            ConditionalBuildState::Body if token == "fi" => ConditionalBuildState::Done,
            _ => {
                return Err(ParseError{
                    error: String::from(format!("Unexpected token: {} in state: {:?}", token, self.state)),
                    continuable: false
                });
            }
        };

        return Ok(true);
    }

    fn is_complete(&self) -> bool {
        return match self.state {
            ConditionalBuildState::Done => true,
            _ => false
        };
    }

    fn ingest_node(&mut self, node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        match self.state {
            ConditionalBuildState::Condition => {
                self.condition.push(node);
                return Ok(true);
            },
            ConditionalBuildState::Body => {
                self.body.push(node);
                return Ok(true);
            },
            ConditionalBuildState::Done => {
                return Err(ParseError{
                    error: String::from("Expected token `fi`"),
                    continuable: false,
                });
            }
        }
    }
}

impl fmt::Display for IfNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("if ")?;
        for node in &self.condition {
            write!(f, "{}; ", node)?;
        }
        f.write_str(" then\n")?;
        for node in &self.body {
            write!(f, "{}; ", node)?;
        }
        f.write_str("\ndone")?;
        return Ok(());
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

    fn ingest_token(&mut self, token: &String) -> Result<bool, ParseError>{
        match token.as_str() {
            "|" => {
                return Err(ParseError{
                    error: String::from("Pipe signals end of process ingestion"),
                    continuable: true
                });
            },
            "&" => {
                self.foreground = false;
                return Ok(true);
            },
            _ => {
                if self.proc_name == "" {
                    self.proc_name = token.clone();
                }
                self.argv.push(token.clone());
            }
        }
        return Ok(true);
    }

    fn is_complete(&self) -> bool {
        return false;
    }

    fn ingest_node(&mut self, _node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        return Err(ParseError{
            error: String::from("Process nodes can't contain other nodes"),
            continuable: false
        });
    }
}

impl fmt::Display for ProcessNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.argv[..].join(" "));
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

    fn ingest_token(&mut self, token: &String) -> Result<bool, ParseError> {
        match token.as_str() {
            "\n" | ";" => {
                return Err(ParseError{
                    error: String::from("End of Pipeline"),
                    continuable: true
                }); // If we hit a ; or a \n, we're at the end of the whole pipeline
            },
            _ => {}
        }

        if self.finished {
            let mut node = ProcessNode::new();
            match node.ingest_token(token) {
                Ok(_) => {
                    self.processes.push(node);
                    self.finished = false;
                    return Ok(true);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        else {
            self.finished = match self.processes.last_mut().unwrap().ingest_token(token) {
                Ok(_) => false,
                Err(err) if err.continuable => true,
                Err(err) => {
                    return Err(err);
                }
            };
        }

        return Ok(true);
    }

    fn is_complete(&self) -> bool {
        return self.finished;
    }

    fn ingest_node(&mut self, _node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        return Err(ParseError{
            error: String::from("Pipeline nodes can't contain other nodes"),
            continuable: false
        });
    }
}

impl fmt::Display for PipelineNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.processes.iter() {
            match write!(f, "{} | ", node) {
                Ok(_) => {},
                Err(err) => {
                    return Err(err);
                }
            }
        }
        return Ok(());
    }
}