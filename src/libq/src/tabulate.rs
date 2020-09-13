use std::cmp::max;
use std::fmt;

pub struct Table {
    headers: Vec<String>,
    values: Vec<Vec<String>>,
    widths: Vec<usize>,
}

#[derive(Copy, Clone, Debug)]
pub enum TableError {
    InvalidValues,
}

const BAR: char = '｜';
const UNDERSCORE: char = '-';

impl Table {
    pub fn new_from_vec(headers: Vec<String>) -> Table {
        return Table {
            widths: headers.iter().map(|s| s.len()).collect(),
            headers: headers,
            values: Vec::new(),
        };
    }

    pub fn new(headers: &[&str]) -> Table {
        return Table {
            headers: headers.iter().map(|&s| s.to_owned()).collect(),
            values: Vec::new(),
            widths: headers.iter().map(|s| s.len()).collect(),
        };
    }

    pub fn add_values(&mut self, values: Vec<String>) -> Result<(), TableError> {
        if values.len() != self.headers.len() {
            return Err(TableError::InvalidValues);
        }

        self.widths = (0..values.len()).map(|i| max(self.widths[i], values[i].len())).collect();

        self.values.push(values);
        return Ok(());
    }

    pub fn to_string(&self) -> String {
        let mut build = String::new();
        build.push(BAR);
        for x in 0..self.headers.len() {
            let value = &self.headers[x];
            let padding = self.widths[x] - value.len();
            build.push(' ');
            build.push_str(&value);
            if padding > 0 {
                build.push_str(&(0..padding).map(|_| " ").collect::<String>());
            }
            build.push(' ');
            build.push(BAR);
        }

        build.push('\n');
        build.push(BAR);

        for w in self.widths.iter() {
            build.push_str(&(0..*w + 2).map(|_| UNDERSCORE).collect::<String>());
            build.push(BAR);
        }
        build.push('\n');

        for y in 0..self.values.len() {
            build.push(BAR);
            for x in 0..self.headers.len() {
                let value = &self.values[y][x];
                let padding = self.widths[x] - value.len();
                build.push(' ');
                build.push_str(&value);
                if padding > 0 {
                    build.push_str(&(0..padding).map(|_| " ").collect::<String>());
                }
                build.push(' ');
                build.push(BAR);
            }
            build.push('\n');
        }

        return build;
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.to_string());
    }
}
