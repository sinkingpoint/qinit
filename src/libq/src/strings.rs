use std::str::Chars;
use std::fmt;

enum QuoteType {
    Single,
    Double,
    None
}

impl QuoteType {
    fn from_char(chr: char) -> QuoteType {
        match chr {
            '\'' => QuoteType::Single,
            '\"' => QuoteType::Double,
            _ => QuoteType::None,
        }
    }

    fn get_char(&self) -> char {
        match &self {
            QuoteType::Single => '\'',
            QuoteType::Double => '"',
            QuoteType::None => '\0',
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
            _ => false
        }
    }
}

pub enum TokenizationError {
    UnclosedQuote,
    InvalidEscapeChar,
    Done,
}

impl TokenizationError {
    pub fn is_continuable(&self) -> bool{
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
            TokenizationError::Done => write!(f, "Done")
        }
    }
}

pub struct Tokenizer<'a> {
    base: &'a str,
    iter: std::iter::Peekable<Chars<'a>>,
    last_err: Option<TokenizationError>
}

impl<'a> Tokenizer<'a> {
    pub fn new(corpus: &'a str) -> Tokenizer{
        return Tokenizer {
            base: corpus,
            iter: corpus.chars().peekable(),
            last_err: None,
        }
    }

    fn reset(&mut self) {
        self.iter = self.base.chars().peekable();
    }

    pub fn try_tokenize(&mut self) -> Result<Vec<String>, &TokenizationError> {
        let tokens = self.collect();
        self.reset();
        return match &self.last_err {
            Some(err) => Err(err),
            None => Ok(tokens)
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut current_token = Token{
            build: "".to_string(),
            started: false,
            ended: false,
            in_quotes: QuoteType::None
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
            current_token.started = true;
            let new_char = self.iter.next().unwrap();
            if new_char == '"' || new_char == '\''{
                current_token.in_quotes = match current_token.in_quotes {
                    QuoteType::None => QuoteType::from_char(new_char),
                    _ if new_char == current_token.in_quotes.get_char() => QuoteType::None,
                    _ => {
                        current_token.in_quotes
                    }
                };
                current_token.build.push(new_char);
            }
            else if new_char == '\\' {
                // We're got an escape char next
                if self.iter.peek().is_some() {
                    let esc_char = self.iter.next().unwrap();
                    let actual_char = match esc_char {
                        '\'' => '\'',
                        '\"' => '\"',
                        '\\' => '\\',
                        '\n' => ' ',
                        'r' => '\r',
                        'n' => '\n',
                        't' => '\t',
                        '0' => '\0',
                        _ => {
                            self.last_err = Some(TokenizationError::InvalidEscapeChar);
                            return None;
                        }
                    };
                    current_token.build.push(actual_char);
                }
            }
            else if new_char.is_whitespace() {
                if current_token.skip_whitespace() {
                    current_token.ended = true;
                }
                else {
                    current_token.build.push(new_char);
                }
            }
            else {
                current_token.build.push(new_char);
            }
        }

        if current_token.ended {
            return Some(current_token.build.to_string());
        }
        else {
            if self.iter.peek().is_none() {
                // We're at the end of the string
                // If we're still in a quote, we've got an error, so TODO: actually make that an error
                match current_token.in_quotes {
                    QuoteType::Single | QuoteType::Double => {
                        self.last_err = Some(TokenizationError::UnclosedQuote);
                        return None;
                    },
                    QuoteType::None => {
                        if current_token.started {
                            return Some(current_token.build);
                        }
                        return None;
                    }
                }
            }
            else {
                self.last_err = Some(TokenizationError::Done);
                return None;
            }
        }
    }
}