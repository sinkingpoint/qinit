use std::str::Chars;

enum QuoteType {
    SINGLE,
    DOUBLE,
    NONE
}

impl QuoteType {
    fn from_char(chr: char) -> QuoteType {
        match chr {
            '\'' => QuoteType::SINGLE,
            '\"' => QuoteType::DOUBLE,
            _ => QuoteType::NONE,
        }
    }

    fn get_char(&self) -> char {
        match &self {
            QuoteType::SINGLE => '\'',
            QuoteType::DOUBLE => '"',
            QuoteType::NONE => '\0',
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
            QuoteType::NONE => true,
            _ => false
        }
    }
}

pub struct Tokenizer<'a> {
    iter: std::iter::Peekable<Chars<'a>>
}

impl<'a> Tokenizer<'a> {
    pub fn new(corpus: &'a str) -> Tokenizer{
        return Tokenizer {
            iter: corpus.chars().peekable(),
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
            in_quotes: QuoteType::NONE
        };

        // If the current token hasn't started, lets skip whitespace until we get to the start
        while self.iter.peek().is_some() && self.iter.peek().unwrap().is_whitespace() {
            self.iter.next();
            continue;
        }

        // We're at non whitespace, so lets start ingesting chars
        while !current_token.ended && self.iter.peek().is_some() {
            current_token.started = true;
            let new_char = self.iter.next().unwrap();
            if new_char == '"' || new_char == '\''{
                let new_quote = QuoteType::from_char(new_char);
                current_token.in_quotes = match current_token.in_quotes {
                    QuoteType::NONE => new_quote,
                    _ if new_char == current_token.in_quotes.get_char() => QuoteType::NONE,
                    _ => {
                        current_token.build.push(new_char);
                        current_token.in_quotes
                    }
                }
            }
            else if new_char == '\\' {
                // We're got an escape char next
                if self.iter.peek().is_some() {
                    let esc_char = self.iter.next().unwrap();
                    let actual_char = match esc_char {
                        '\'' => '\'',
                        '\"' => '\"',
                        '\\' => '\\',
                        'r' => '\r',
                        'n' => '\n',
                        't' => '\t',
                        '0' => '\0',
                        _ => {
                            // TODO: Handle invalid escape chars here
                            panic!("Invalid escape char: \\{}", esc_char);
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
                if current_token.started {
                    return Some(current_token.build);
                }
                return None;
            }
            else {
                return None; // Here we should handle the case where we're at the end of the string, but not at the end of a token
            }
        }
    }
}