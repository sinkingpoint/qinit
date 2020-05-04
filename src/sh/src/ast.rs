use shell;
use nix::unistd::Pid;
use std::fmt;
use strings;

pub trait ASTNode: fmt::Display{
    fn execute(&self, &mut shell::Shell, Option<Pid>, &shell::IOTriple) -> i32;
    fn ingest_token(&mut self, &String) -> Result<bool, ParseError>;
    fn is_complete(&self) -> bool;
    fn ingest_node(&mut self, Box<dyn ASTNode>) -> Result<bool, ParseError>;
    fn takes_tokens(&self) -> bool;
}

pub struct ParseError {
    pub error: String,
    pub continuable: bool, // If an error is continuable, we will continue reading until we hit a valid and complete statement, or a non continuable error
}

impl ParseError {
    fn new(error: &str, continuable: bool) -> ParseError {
        return ParseError {
            error: String::from(error),
            continuable: continuable
        };
    }

    fn from_string(error: String, continuable: bool) -> ParseError {
        return ParseError {
            error: error,
            continuable: continuable
        }
    }
}

pub fn parse_into_ast(tokens: &Vec<String>) -> Result<Box<dyn ASTNode>, ParseError> {
    if tokens.len() == 0 {
        return Err(ParseError::new("Empty Statement", false));
    }

    let mut current_node: Box<dyn ASTNode> = Box::new(PipelineNode::new());
    let mut block_stack:Vec<Box<dyn ASTNode>> = Vec::new();
    block_stack.push(Box::new(ASTHead::new()));
    
    for token in tokens {
        if token.trim().len() == 0 {
            continue;
        }
        match token.as_str() {
            "if" if current_node.as_ref().is_complete() => {
                block_stack.push(Box::new(IfNode::new()) as Box<dyn ASTNode>);
            },
            "for" if current_node.as_ref().is_complete() => {
                block_stack.push(Box::new(ForNode::new()) as Box<dyn ASTNode>);
            },
            "case" if current_node.as_ref().is_complete() => {
                block_stack.push(Box::new(CaseNode::new()) as Box<dyn ASTNode>);
            },
            "while" if current_node.as_ref().is_complete() => {}, // TODO: While Statements
            "then" | "fi" | "do" | "done" | "in" | ";;" => {
                block_stack.last_mut().unwrap().ingest_node(current_node)?;
                current_node = Box::new(PipelineNode::new());
                match block_stack.last_mut().unwrap().ingest_token(token) {
                    Ok(_) => {},
                    Err(err) if err.continuable => {
                        let new_block = block_stack.pop().unwrap();
                        let last_block = block_stack.last_mut().unwrap();
                        last_block.ingest_node(new_block)?;
                    },
                    Err(err) => {
                        return Err(err);
                    }
                }

                if block_stack.last_mut().unwrap().is_complete() {
                    let new_block = block_stack.pop().unwrap();
                    let last_block = block_stack.last_mut().unwrap();
                    last_block.ingest_node(new_block)?;
                }
            },
            "&&" => {}, // TODO: And statements
            "||" => {}, // TODO: Or Statements
            _ => {
                let mut node = match block_stack.last_mut() {
                    Some(node) if node.takes_tokens() => node,
                    _ => &mut current_node
                };

                match node.ingest_token(token) {
                    Ok(_) => {},
                    Err(err) => {
                        if err.continuable {
                            block_stack.last_mut().unwrap().ingest_node(current_node)?;
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

    block_stack.last_mut().unwrap().ingest_node(current_node)?;
    if block_stack.len() == 2 {
        if !block_stack.last_mut().unwrap().is_complete() {
            return Err(ParseError::new("Unclosed block", true));
        }
        let new_block = block_stack.pop().unwrap();
        let last_block = block_stack.last_mut().unwrap();
        last_block.ingest_node(new_block)?;
    }
    else if block_stack.len() > 1 {
        return Err(ParseError::new("Unclosed block", true));
    }

    return Ok(block_stack.pop().unwrap());
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
        return Err(ParseError::new("Failed to ingest token - Head objects don't take tokens!", false));
    }

    fn is_complete(&self) -> bool {
        return false;
    }

    fn ingest_node(&mut self, node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        self.nodes.push(node);
        return Ok(true);
    }

    fn takes_tokens(&self) -> bool {
        return false;
    }
}

impl fmt::Display for ASTHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in self.nodes.iter() {
            write!(f, "{}", node)?;
        }
        return Ok(());
    }
}

#[derive(Debug)]
enum ConditionalBuildState {
    Variable,
    Condition,
    EndCondition,
    Body,
    EndBody,
    Done
}

struct Case {
    glob: String,
    body: Vec<Box<dyn ASTNode>>
}

impl Case {
    fn new(glob:String) -> Case {
        return Case {
            glob: glob,
            body: Vec::new()
        }
    }

    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let mut ret_value = 0;
        for statement in &self.body {
            ret_value = statement.execute(shell, pgid, streams);
        }

        return ret_value;
    }
}

struct CaseNode {
    state: ConditionalBuildState,
    variable: String,
    cases: Vec<Case>
}

impl CaseNode {
    fn new() -> CaseNode {
        return CaseNode {
            state: ConditionalBuildState::Variable,
            variable: String::new(),
            cases: Vec::new()
        };
    }
}

impl ASTNode for CaseNode {
    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let value = strings::do_value_pipeline(&self.variable, shell).unwrap().join(" ");
        let mut default = None;
        for case in &self.cases {
            if case.glob == "*" {
                default = Some(case);
                continue;
            }

            if strings::match_glob(&case.glob, &value) {
                return case.execute(shell, pgid, streams);
            }
        }

        // If we fall through, then execute the default one if it exists
        if let Some(default_case) = default {
            return default_case.execute(shell, pgid, streams);
        }

        return -1;
    }

    fn takes_tokens(&self) -> bool {
        return match &self.state {
            ConditionalBuildState::Variable | ConditionalBuildState::Condition => true,
            _ => false
        };
    }

    fn ingest_token(&mut self, token: &String) -> Result<bool, ParseError> {
        match &self.state {
            ConditionalBuildState::Variable => {
                if self.variable == "" {
                    self.variable = token.clone();
                    return Ok(true);
                }
                else if token == "in" {
                    self.state = ConditionalBuildState::Condition;
                    return Ok(true);
                }
                return Err(ParseError::new("Expected `in`", false));
            }
            ConditionalBuildState::Condition => {
                if token.ends_with(")") {
                    self.cases.push(Case::new(token.trim_end_matches(")").to_string()));
                    self.state = ConditionalBuildState::Body;
                    return Ok(true);
                }
                else if token == "esac" {
                    self.state = ConditionalBuildState::Done;
                    return Ok(true);
                }
                return Err(ParseError::from_string(format!("Parse error near {}", token), false));
            },
            ConditionalBuildState::Body => {
                if token == ";;" {
                    self.state = ConditionalBuildState::Condition;
                    return Ok(true);
                }
                return Err(ParseError::from_string(format!("Unexpected token `{}`", token), false));
            },
            _ => {
                return Err(ParseError::from_string(format!("Unexpected token `{}`", token), false));
            }
        }
    }

    fn is_complete(&self) -> bool {
        return match &self.state {
            ConditionalBuildState::Done => true,
            _ => false
        };
    }

    fn ingest_node(&mut self, node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        match &self.state {
            ConditionalBuildState::Body => {
                self.cases.last_mut().unwrap().body.push(node);
                return Ok(true);
            },
            _ => {}
        }

        return Ok(true);
    }
}

impl fmt::Display for CaseNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "case {} in\n", self.variable)?;
        for case in &self.cases {
            writeln!(f, "{})", case.glob)?;
            for node in &case.body {
                writeln!(f, "{}", node)?;
            }
            writeln!(f, ";;")?;
        }
        writeln!(f, "esac")?;
        return Ok(());
    }
}

struct ForNode {
    variable: String,
    state: ConditionalBuildState,
    iter: Vec<String>,
    body: Vec<Box<dyn ASTNode>>
}

impl ForNode {
    fn new() -> ForNode {
        return ForNode {
            state: ConditionalBuildState::Variable,
            variable: String::new(),
            iter: Vec::new(),
            body: Vec::new(),
        }
    }
}

impl ASTNode for ForNode {
    fn execute(&self, shell: &mut shell::Shell, pgid: Option<Pid>, streams: &shell::IOTriple) -> i32 {
        let mut last_exit = 0;
        for value in &self.iter {
            match strings::do_value_pipeline(&value, shell) {
                Ok(words) => {
                    for word in words {
                        shell.set_variable(shell::Variable::new(self.variable.clone(), word, false));
                        for node in &self.body {
                            last_exit = node.execute(shell, pgid, streams);
                        }
                    }
                },
                Err(err) => {
                    eprintln!("{}", err);
                    last_exit = 255;
                }
            }
        }

        return last_exit;
    }

    fn takes_tokens(&self) -> bool {
        return match &self.state {
            ConditionalBuildState::Variable | ConditionalBuildState::Condition => true,
            _ => false
        };
    }

    fn ingest_token(&mut self, token: &String) -> Result<bool, ParseError> {
        match &self.state {
            ConditionalBuildState::Variable => {
                match token.as_str() {
                    "in" => {
                        if self.variable == "" {
                            return Err(ParseError::new("Expecting variable in for", false));
                        }
                        self.state = ConditionalBuildState::Condition;
                    },
                    _ => {
                        if self.variable != "" {
                            return Err(ParseError::new("Expecting `in`", false));
                        }
                        self.variable = token.clone();
                    }
                }
            },
            ConditionalBuildState::Condition => {
                match token.as_str() {
                    ";" => {
                        self.state = ConditionalBuildState::EndCondition;
                    },
                    _ => {
                        self.iter.push(token.to_string());
                    }
                }
            },
            ConditionalBuildState::EndCondition => {
                match token.as_str() {
                    "do" => {
                        self.state = ConditionalBuildState::Body;
                    },
                    _ => {
                        return Err(ParseError::new("Unexpected token. Expected `do`", false));
                    }
                }
            },
            ConditionalBuildState::Body => {
                match token.as_str() {
                    "done" => {
                        self.state = ConditionalBuildState::Done;
                    },
                    _ => {
                        return Err(ParseError::new("Unexpected token. Expected `do`", false));
                    }
                }
            },
            ConditionalBuildState::EndBody => {
                return Err(ParseError::new("Unexpected state EndBody", false));
            },
            ConditionalBuildState::Done => {
                return Err(ParseError::new("Unexpected state Done", false));
            }
        };
        return Ok(true);
    }

    fn is_complete(&self) -> bool {
        return match &self.state {
            ConditionalBuildState::Done => true,
            _ => false
        };
    }

    fn ingest_node(&mut self, node: Box<dyn ASTNode>) -> Result<bool, ParseError> {
        match &self.state {
            ConditionalBuildState::Body => {
                self.body.push(node);
                return Ok(true);
            },
            _ => {}
        }

        return Ok(true);
    }
}

impl fmt::Display for ForNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("for ")?;
        for node in &self.iter {
            write!(f, "{} ", node)?;
        }
        f.write_str("do\n")?;
        for node in &self.body {
            write!(f, "{}; ", node)?;
        }
        f.write_str("\ndone")?;
        return Ok(());
    }
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
                return Err(ParseError::from_string(format!("Unexpected token: {}", token), false));
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
                return Err(ParseError::from_string(format!("Expected token `fi`"), false));
            },
            _ => {panic!("Unreachable");}
        }
    }

    fn takes_tokens(&self) -> bool {
        return false;
    }
}

impl fmt::Display for IfNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("if ")?;
        for node in &self.condition {
            write!(f, "{}; ", node)?;
        }
        f.write_str("then\n")?;
        for node in &self.body {
            write!(f, "{}; ", node)?;
        }
        f.write_str("\nfi")?;
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
                return Err(ParseError::new("Pipe signals end of process ingestion", true));
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
        return Err(ParseError::new("Process nodes can't contain other nodes", false));
    }

    fn takes_tokens(&self) -> bool {
        return false;
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
                return Err(ParseError::new("End of Pipeline", true)); // If we hit a ; or a \n, we're at the end of the whole pipeline
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
        return Err(ParseError::new("Pipeline nodes can't contain other nodes", false));
    }

    fn takes_tokens(&self) -> bool {
        return false;
    }
}

impl fmt::Display for PipelineNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        for node in self.processes.iter() {
            write!(f, "{}", node)?;
            if i == self.processes.len() {
                write!(f, " | ")?;
            }
            i += 1;
        }
        return Ok(());
    }
}