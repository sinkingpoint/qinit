use shell;
use std::str::Chars;
use std::iter::Peekable;

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

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Copy)]
pub enum QuoteType {
    None,
    Single,
    Double
}

impl QuoteType {
    fn from_chr(c: char) -> QuoteType {
        return match c {
            '\'' => QuoteType::Single,
            '\"' => QuoteType::Double,
            _ => QuoteType::None
        }
    }
}

impl Clone for QuoteType {
    fn clone(&self) -> QuoteType {
        return match self {
            QuoteType::None => QuoteType::None,
            QuoteType::Single => QuoteType::Single,
            QuoteType::Double => QuoteType::Double
        }
    }
}

pub struct StringIterWithQuoteContext<'a> {
    base_iter: Peekable<Chars<'a>>,
    current_quote: QuoteType
}

pub struct CharWithQuoteContext {
    pub chr: char,
    pub context: QuoteType
}

impl CharWithQuoteContext {
    fn new(chr: char, context: QuoteType) -> CharWithQuoteContext {
        return CharWithQuoteContext{
            chr: chr,
            context: context
        };
    }
}

impl StringIterWithQuoteContext<'_> {
    fn new<'a>(base: &'a String) -> StringIterWithQuoteContext<'a> {
        return StringIterWithQuoteContext {
            base_iter: base.chars().peekable(),
            current_quote: QuoteType::None
        }
    }
}

impl Iterator for StringIterWithQuoteContext<'_> {
    type Item = CharWithQuoteContext;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(chr) = self.base_iter.next() {
            match chr {
                '\\' => {
                    let next = self.base_iter.peek();
                    match next {
                        Some(c) => {
                            if QuoteType::from_chr(*c) != QuoteType::None {
                                return Some(CharWithQuoteContext::new(self.base_iter.next().unwrap(), self.current_quote));
                            }
                            return Some(CharWithQuoteContext::new('\\', self.current_quote));
                        },
                        None => {
                            return Some(CharWithQuoteContext::new('\\', self.current_quote));
                        }
                    }
                },
                quote if QuoteType::from_chr(quote) != QuoteType::None => {
                    let new_quote = QuoteType::from_chr(quote);
                    if self.current_quote == QuoteType::None {
                        self.current_quote = new_quote;
                    }
                    else if new_quote == self.current_quote {
                        self.current_quote = QuoteType::None;
                    }
                    else {
                        return Some(CharWithQuoteContext::new(quote, self.current_quote));
                    }
                }
                _ => {
                    return Some(CharWithQuoteContext::new(chr, self.current_quote));
                }
            }
        }

        return None;
    }
}

impl<'a> From<&'a String> for StringIterWithQuoteContext<'a> {
    fn from(s: &'a String) -> StringIterWithQuoteContext<'a> {
        return StringIterWithQuoteContext::new(s);
    }
}

pub trait IntoStringIterWithQuoteContext {
    fn chars_with_quotes<'a>(&'a self) -> StringIterWithQuoteContext<'a>;
}

impl IntoStringIterWithQuoteContext for String {
    fn chars_with_quotes<'a>(&'a self) -> StringIterWithQuoteContext<'a> {
        return StringIterWithQuoteContext::new(self);
    }
}

pub fn do_string_interpolation(token: &String, shell: &shell::Shell) -> Result<String, &'static str> {
    let mut build = String::new();
    let mut var_build: Option<VariableBuilder> = None;
    for nchr in token.chars_with_quotes() {
        let chr = nchr.chr;
        let context = nchr.context;
        if chr == '$' && context != QuoteType::Single && var_build.is_none(){
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

