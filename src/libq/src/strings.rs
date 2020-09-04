use super::io::{self, FileType};
use std::ffi::{CStr, CString};
use std::fmt;
use std::str::Chars;

#[derive(PartialEq)]
enum QuoteType {
    Single,
    Double,
    None,
}

impl QuoteType {
    fn from_char(chr: char) -> QuoteType {
        match chr {
            '\'' => QuoteType::Single,
            '\"' => QuoteType::Double,
            _ => QuoteType::None,
        }
    }
}

struct Token {
    build: String,
    started: bool,
    ended: bool,
    in_quotes: QuoteType,
}

impl Token {
    fn skip_whitespace(&self) -> bool {
        return match self.in_quotes {
            QuoteType::None => true,
            _ => false,
        };
    }
}

#[derive(Debug)]
pub enum TokenizationError {
    UnclosedQuote,
    InvalidEscapeChar,
    Done,
}

impl TokenizationError {
    pub fn is_continuable(&self) -> bool {
        match self {
            TokenizationError::UnclosedQuote | TokenizationError::Done => true,
            TokenizationError::InvalidEscapeChar => false,
        }
    }
}

impl fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenizationError::UnclosedQuote => write!(f, "Unclosed Quote"),
            TokenizationError::InvalidEscapeChar => write!(f, "Invalid Escape Char"),
            TokenizationError::Done => write!(f, "Done"),
        }
    }
}

pub struct Tokenizer<'a> {
    base: &'a str,
    iter: std::iter::Peekable<Chars<'a>>,
    last_err: Option<TokenizationError>,
    split_chars: Vec<char>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(corpus: &'a str, split_chars: Vec<char>) -> Tokenizer {
        return Tokenizer {
            base: corpus,
            iter: corpus.chars().peekable(),
            last_err: None,
            split_chars: split_chars,
        };
    }

    fn reset(&mut self) {
        self.iter = self.base.chars().peekable();
    }

    pub fn try_tokenize(&mut self) -> Result<Vec<String>, &TokenizationError> {
        let tokens = self.collect();
        self.reset();
        return match &self.last_err {
            Some(err) => Err(err),
            None => Ok(tokens),
        };
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut current_token = Token {
            build: "".to_string(),
            started: false,
            ended: false,
            in_quotes: QuoteType::None,
        };

        // If the current token hasn't started, lets skip whitespace until we get to the start
        while self.iter.peek().is_some() && self.iter.peek().unwrap().is_whitespace() {
            if self.iter.peek() == Some(&'\n') {
                self.iter.next();
                return Some(String::from("\n"));
            }
            self.iter.next();
            continue;
        }

        // We're at non whitespace, so lets start ingesting chars
        while !current_token.ended && self.iter.peek().is_some() {
            if self.split_chars.contains(&'#')
                && self.iter.peek() == Some(&'#')
                && current_token.in_quotes == QuoteType::None
                && !current_token.started
            {
                // If we hit a #, and we're not in quotes and we're at the start of a token, then this is a comment
                return Some(self.iter.next().unwrap().to_string());
            }

            // Handle "split" chars - special chars that mark the end of a token, but should be in their own token
            if self.split_chars.contains(self.iter.peek()?) && self.iter.peek()? != &'#' {
                if current_token.in_quotes == QuoteType::None {
                    // Split chars are only valid not in quotes
                    if current_token.started {
                        // If we hit a split token and we've got a token, return the token
                        return Some(current_token.build);
                    } else if self.iter.peek() == Some(&';') || self.iter.peek() == Some(&'&') {
                        // special case split strings, that could be two
                        // If we haven't started a token, just consume and return the ;
                        let mut build = String::new();
                        build.push(self.iter.next().unwrap());
                        // ;; is a special meaning in case statements, so just make sure this isn't that
                        if self.iter.peek().is_some() && self.iter.peek() == build.chars().next().as_ref() {
                            build.push(self.iter.next().unwrap());
                        }
                        return Some(build);
                    } else {
                        return Some(self.iter.next().unwrap().to_string());
                    }
                }
            }

            current_token.started = true;
            let new_char = self.iter.next().unwrap();
            if new_char == '"' || new_char == '\'' {
                current_token.in_quotes = match current_token.in_quotes {
                    QuoteType::None => QuoteType::from_char(new_char),
                    _ if QuoteType::from_char(new_char) == current_token.in_quotes => QuoteType::None,
                    _ => current_token.in_quotes,
                };
                current_token.build.push(new_char);
            } else if new_char.is_whitespace() {
                if current_token.skip_whitespace() {
                    current_token.ended = true;
                } else {
                    current_token.build.push(new_char);
                }
            } else {
                current_token.build.push(new_char);
            }
        }

        if current_token.ended {
            return Some(current_token.build.to_string());
        } else {
            if self.iter.peek().is_none() {
                // We're at the end of the string
                // If we're still in a quote, we've got an error, so TODO: actually make that an error
                match current_token.in_quotes {
                    QuoteType::Single | QuoteType::Double => {
                        self.last_err = Some(TokenizationError::UnclosedQuote);
                        return None;
                    }
                    QuoteType::None => {
                        if current_token.started {
                            return Some(current_token.build);
                        }
                        return None;
                    }
                }
            } else {
                self.last_err = Some(TokenizationError::Done);
                return None;
            }
        }
    }
}

pub fn cstr_to_string(s: &[u8]) -> Result<String, ()> {
    let mut bytes = Vec::new();
    for i in 0..s.len() {
        bytes.push(s[i]);
        if s[i] == 0 {
            break;
        }
    }

    if bytes.len() == 1 {
        return Err(());
    }

    if bytes.len() > 0 && bytes.last().unwrap() != &0 {
        bytes.push(0);
    }

    if let Ok(cstr) = CStr::from_bytes_with_nul(&bytes) {
        let cstring = CString::from(cstr);
        if let Ok(s) = cstring.into_string() {
            return Ok(s);
        }
    }
    return Err(());
}

pub fn format_file_mode(mode: u32) -> String {
    let mut out = String::new();
    let chr = match FileType::from_mode(mode) {
        Ok(m) => m.to_char(),
        Err(_) => '-',
    };
    out.push(chr);
    for (bits, chr) in [
        (io::S_IRUSR, 'r'),
        (io::S_IWUSR, 'w'),
        (io::S_ISUID, 's'),
        (io::S_IXUSR, 'x'),
        (io::S_IRGRP, 'r'),
        (io::S_IWGRP, 'w'),
        (io::S_ISGID, 's'),
        (io::S_IXGRP, 'x'),
        (io::S_IROTH, 'r'),
        (io::S_IWOTH, 'w'),
        (io::S_IXOTH, 'x'),
    ]
    .iter()
    {
        if mode & bits == *bits {
            out.push(*chr);
        } else if chr != &'s' {
            out.push('-');
        }
    }

    return out;
}

pub fn glob_to_regex(glob: &str) -> String {
    let mut result = String::new();
    let mut iter = glob.chars().peekable();
    while let Some(chr) = iter.next() {
        match chr {
            '*' => {
                result.push_str(".*");
            }
            '?' => {
                result.push('.');
            }
            '[' => {
                let mut build = String::new();
                if iter.peek().is_some() && iter.peek().unwrap() != &'!' {
                    build.push(iter.next().unwrap());
                }

                if iter.peek().is_some() && iter.peek().unwrap() != &']' {
                    build.push(iter.next().unwrap());
                }

                while iter.peek().is_some() && iter.peek().unwrap() != &']' {
                    build.push(iter.next().unwrap());
                }

                if iter.peek().is_none() {
                    result.push_str("\\[");
                } else {
                    iter.next(); // Skip the ]
                    build = build.replace("\\", "\\\\");
                    if build.starts_with("!") {
                        build = build.replacen("!", "^", 1);
                    } else if build.starts_with("^") {
                        build.insert(0, '\\');
                    }
                    result.push_str(&format!("[{}]", build));
                }
            }
            '(' | ')' | ']' | '{' | '}' | '+' | '-' | '|' | '^' | '$' | '\\' | '.' | '&' | '~' | '#' | ' ' | '\t' | '\n' | '\r' => {
                result.push_str(&format!("\\{}", chr));
            }
            _ => {
                result.push(chr);
            }
        }
    }

    return format!("^{}$", result);
}