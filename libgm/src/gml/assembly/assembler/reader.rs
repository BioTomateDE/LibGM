use std::{
    ops::Neg,
    str::{Chars, FromStr},
};

use crate::{prelude::*, util::fmt::typename};

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    pub line: &'a str,
}

impl<'a> Reader<'a> {
    #[must_use]
    pub const fn new(line: &'a str) -> Self {
        Self { line }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.line.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.line.is_empty()
    }

    pub const fn clear(&mut self) -> &'a str {
        let line = self.line;
        self.line = "";
        line
    }

    #[must_use]
    pub fn starts_with(&self, substring: &str) -> bool {
        self.line.starts_with(substring)
    }

    pub fn consume_to(&mut self, length: usize) -> &str {
        let line = self.line;
        self.line = &self.line[length..];
        &line[..length]
    }

    #[must_use]
    pub fn peek_char(&self) -> Option<char> {
        self.line.chars().next()
    }

    pub fn consume_char(&mut self) -> Option<char> {
        let mut chars: Chars = self.line.chars();
        let first_char: Option<char> = chars.next();
        self.line = chars.as_str();
        first_char
    }

    #[must_use]
    pub fn consume_str(&mut self, string: &'static str) -> Option<()> {
        if !self.starts_with(string) {
            return None;
        }
        self.line = &self.line[string.len()..];
        Some(())
    }

    pub fn consume_space(&mut self) -> Result<()> {
        let char: char = self.consume_char().ok_or("Expected space, got EOL")?;
        if char != ' ' {
            bail!("Expected space, got '{char}'");
        }
        Ok(())
    }

    pub fn consume_dot(&mut self) -> Result<()> {
        let char: char = self.consume_char().ok_or("Expected dot, got EOL")?;
        if char != '.' {
            bail!("Expected dot, got '{char}'");
        }
        Ok(())
    }

    fn consume_brackets(&mut self, open: char, close: char) -> Result<Option<&str>> {
        if !self.line.starts_with(open) {
            return Ok(None);
        }

        let close_pos = self
            .line
            .find(close)
            .ok_or_else(|| format!("'{open}' was never closed"))?;

        let line = self.line;
        self.line = &self.line[close_pos + 1..];
        let inside = &line[1..close_pos];

        Ok(Some(inside))
    }

    pub fn consume_round_brackets(&mut self) -> Result<Option<&str>> {
        self.consume_brackets('(', ')')
    }

    pub fn consume_square_brackets(&mut self) -> Result<Option<&str>> {
        self.consume_brackets('[', ']')
    }

    pub fn consume_angle_brackets(&mut self) -> Result<Option<&str>> {
        self.consume_brackets('<', '>')
    }

    pub fn parse_identifier(&mut self) -> Result<&str> {
        // Identifiers can't start with a digit
        if self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
            bail!("Expected identifier; found {:?}", self.line);
        }

        for (i, char) in self.line.char_indices() {
            match char {
                'a'..='z' | '0'..='9' | 'A'..='Z' | '_' => continue,
                _ => {},
            }

            if i == 0 {
                bail!("Expected identifier; found {:?}", self.line);
            }

            let identifier = self.consume_to(i);
            return Ok(identifier);
        }

        // Identifier goes to end of line
        Ok(self.clear())
    }

    pub fn parse_int<T: FromStr + Neg<Output = T>>(&mut self) -> Result<T> {
        let is_negative: bool = self.starts_with("-");
        if is_negative {
            self.consume_char(); // Consume minus sign
        }
        let integer: T = self.parse_uint()?;
        if is_negative {
            Ok(-integer)
        } else {
            Ok(integer)
        }
    }

    pub fn parse_uint<T: FromStr>(&mut self) -> Result<T> {
        let end: usize = self.find_non_digit();
        if end == 0 {
            bail!("Expected integer, got {:?}", self.line);
        }

        let integer: &str = self.consume_to(end);
        let integer: T = integer.parse().ok().ok_or_else(|| {
            format!(
                "Integer {} is out of bounds for integer type {}",
                integer,
                typename::<T>(),
            )
        })?;

        Ok(integer)
    }

    #[must_use]
    fn find_non_digit(&self) -> usize {
        for (index, character) in self.line.as_bytes().iter().enumerate() {
            match character {
                b'0'..=b'9' => {},
                _ => return index,
            }
        }
        self.len()
    }
}
