use regex::Regex;
use shell;
use std::iter::Peekable;
use std::str::Chars;

struct VariableBuilder {
    build: String,
    in_braces: bool,
    done: bool,

    // Variable substitution options
    ltrim: Option<String>,
}

impl VariableBuilder {
    fn new() -> VariableBuilder {
        return VariableBuilder {
            build: String::new(),
            in_braces: false,
            done: false,
            ltrim: None,
        };
    }

    fn ingest_char(&mut self, c: char) -> Result<(), String> {
        if self.done
            || (c == '$' && self.build.len() > 0)
            || (c == '?' && self.build.len() > 0)
            || (c == '{' && self.build.len() > 0)
            || (c == '#' && self.build.len() == 0)
            || (c == '#' && !self.in_braces)
            || (c == '}' && !self.in_braces)
            || (c == '{' && self.in_braces)
        {
            return Err(format!("Invalid char: {}", c));
        }

        if c == '{' {
            self.in_braces = true;
        } else if c == '}' {
            self.in_braces = false;
            self.done = true;
        } else if c == '$' || c == '?' {
            self.build.push(c);
            self.done = true;
        } else if c == '#' {
            if self.ltrim.is_some() {
                return Err(String::from("Invalid char: #"));
            }
            self.ltrim = Some(String::new());
        } else {
            if self.ltrim.is_some() {
                self.ltrim.as_mut().unwrap().push(c);
            } else {
                self.build.push(c);
            }
        }

        return Ok(());
    }

    fn could_be_done(&self) -> bool {
        return !self.in_braces || self.done;
    }

    fn resolve<'a>(&self, shell: &'a shell::Shell) -> &'a str {
        let var = shell.get_variable(&self.build);
        if self.ltrim.is_none() {
            return var;
        }

        let ltrim = self.ltrim.as_ref().unwrap();
        return var.trim_start_matches(ltrim.as_str());
    }
}

fn glob_to_regex(glob: &String) -> String {
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

pub fn match_glob(glob: &String, value: &String) -> bool {
    let glob_regex = Regex::new(&glob_to_regex(glob)).unwrap();
    return glob_regex.is_match(&value);
}

#[derive(Debug, PartialEq, Copy)]
pub enum QuoteType {
    None,
    Single,
    Double,
    Meta, // Meta quotes are quotes that have started a quote block.
}

impl QuoteType {
    fn from_chr(c: char) -> QuoteType {
        return match c {
            '\'' => QuoteType::Single,
            '\"' => QuoteType::Double,
            _ => QuoteType::None,
        };
    }
}

impl Clone for QuoteType {
    fn clone(&self) -> QuoteType {
        return match self {
            QuoteType::None => QuoteType::None,
            QuoteType::Single => QuoteType::Single,
            QuoteType::Double => QuoteType::Double,
            QuoteType::Meta => QuoteType::Meta,
        };
    }
}

pub struct StringIterWithQuoteContext<'a> {
    base_iter: Peekable<Chars<'a>>,
    current_quote: QuoteType,
    include_quotes: bool,
}

pub struct CharWithQuoteContext {
    pub chr: char,
    pub context: QuoteType,
}

impl CharWithQuoteContext {
    fn new(chr: char, context: QuoteType) -> CharWithQuoteContext {
        return CharWithQuoteContext {
            chr: chr,
            context: context,
        };
    }
}

impl StringIterWithQuoteContext<'_> {
    fn new<'a>(base: &'a String, include_quotes: bool) -> StringIterWithQuoteContext<'a> {
        return StringIterWithQuoteContext {
            base_iter: base.chars().peekable(),
            current_quote: QuoteType::None,
            include_quotes: include_quotes,
        };
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
                        }
                        None => {
                            return Some(CharWithQuoteContext::new('\\', self.current_quote));
                        }
                    }
                }
                quote if QuoteType::from_chr(quote) != QuoteType::None => {
                    let new_quote = QuoteType::from_chr(quote);
                    if self.current_quote == QuoteType::None {
                        self.current_quote = new_quote;
                        if self.include_quotes {
                            return Some(CharWithQuoteContext::new(quote, QuoteType::Meta));
                        }
                    } else if new_quote == self.current_quote {
                        self.current_quote = QuoteType::None;
                        if self.include_quotes {
                            return Some(CharWithQuoteContext::new(quote, QuoteType::Meta));
                        }
                    } else {
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
        return StringIterWithQuoteContext::new(s, false);
    }
}

pub trait IntoStringIterWithQuoteContext {
    fn chars_with_quotes<'a>(&'a self, include_quotes: bool) -> StringIterWithQuoteContext<'a>;
}

impl IntoStringIterWithQuoteContext for String {
    fn chars_with_quotes<'a>(&'a self, include_quotes: bool) -> StringIterWithQuoteContext<'a> {
        return StringIterWithQuoteContext::new(self, include_quotes);
    }
}

fn do_string_interpolation(token: &String, shell: &shell::Shell) -> Result<String, String> {
    let mut build = String::new();
    let mut var_build: Option<VariableBuilder> = None;
    for nchr in token.chars_with_quotes(true) {
        let chr = nchr.chr;
        if chr == '$' && nchr.context != QuoteType::Single && var_build.is_none() {
            var_build = Some(VariableBuilder::new());
        } else {
            match &mut var_build {
                Some(builder) => {
                    if nchr.context == QuoteType::Meta || nchr.chr.is_whitespace() || nchr.chr == ',' || nchr.chr == '.' {
                        // We've hit a quote, or some whitespace, terminate the variable
                        if builder.could_be_done() {
                            build.push_str(builder.resolve(shell));
                            build.push(chr);
                            var_build = None;
                            continue;
                        } else {
                            return Err(format!("Parse error"));
                        }
                    }

                    match builder.ingest_char(chr) {
                        Ok(()) => {}
                        Err(err) => {
                            return Err(format!("Substitution Error: {}", err));
                        }
                    }

                    if builder.done {
                        build.push_str(builder.resolve(shell));
                        var_build = None;
                    }
                }
                None => {
                    build.push(chr);
                }
            }
        }
    }

    match var_build {
        Some(builder) => {
            if builder.could_be_done() {
                build.push_str(builder.resolve(shell));
            } else {
                return Err(String::from("Unclosed variable substitution"));
            }
        }
        None => {}
    }

    return Ok(build);
}

fn do_word_splitting(token: &String) -> Vec<String> {
    let mut build = String::new();
    let mut words = Vec::new();
    for nchr in token.chars_with_quotes(false) {
        let chr = nchr.chr;
        let context = nchr.context;

        if chr.is_whitespace() && context == QuoteType::None {
            if build.len() > 0 {
                words.push(build);
            }
            build = String::new();
        } else {
            build.push(chr);
        }
    }

    words.push(build);

    return words;
}

pub fn do_value_pipeline(token: &String, shell: &shell::Shell) -> Result<Vec<String>, String> {
    return match do_string_interpolation(token, shell) {
        Ok(s) => Ok(do_word_splitting(&s)),
        Err(e) => Err(e),
    };
}
