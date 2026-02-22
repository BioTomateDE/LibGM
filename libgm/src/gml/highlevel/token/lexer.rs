use crate::gml::highlevel::{Location, compile::CompileError, token::Keyword};

use super::{Token, TokenData};

type Result<'a, T> = std::result::Result<T, CompileError<'a>>;

#[derive(Debug)]
struct Lexer<'a> {
    /// The source code to lex/tokenize.
    source_code: &'a str,

    /// The current location of this self in the source code.
    location: Location,

    /// The token stream being produced by this self.
    ///
    /// These will later have to be converted to [`Token`]s.
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(source_code: &'a str) -> Self {
        let location = Location::default();
        let tokens = Vec::new();
        Self { source_code, location, tokens }
    }

    /// Throw a new [`CompileError`].
    pub fn throw<T>(
        &self,
        error: impl Into<crate::Error>,
        start_position: Location,
    ) -> Result<'a, T> {
        let error = CompileError {
            error: error.into(),
            source_code: self.source_code,
            start_position,
            end_position: self.location,
        };
        Err(error)
    }

    /// The current location of this self in the source code.
    #[must_use]
    pub const fn location(&self) -> Location {
        self.location
    }

    fn increment_line(&mut self) {
        self.location.line += 1;
        self.location.char = 0;
    }

    #[must_use]
    pub fn peek_nth_char(&self, n: u32) -> Option<char> {
        self.remaining().chars().nth(n as usize)
    }

    #[must_use]
    pub fn next_char_is(&self, character: char) -> bool {
        self.peek_nth_char(1) == Some(character)
    }

    #[must_use]
    pub fn peek_char(&self) -> Option<char> {
        self.peek_nth_char(0)
    }

    pub fn consume_char(&mut self) -> Option<char> {
        let ch: char = self.peek_char()?;
        self.location.byte += ch.len_utf8() as u32;
        self.location.char += 1;
        if self.peek_char() == Some('\n') {
            self.increment_line();
        }
        Some(ch)
    }

    /// Skips the specified number of ASCII characters.
    /// This function will not work properly if one of the skipped chars is a newline.
    pub fn skip_achars(&mut self, count: u32) {
        self.location.char += count;
        self.location.byte += count;
    }

    #[must_use]
    pub fn remaining(&self) -> &'a str {
        let pos = self.location.byte as usize;
        &self.source_code[pos..]
    }

    /// Consumes until the predicate is `false` or until the end is reached.
    ///
    /// TODO(perf): Optimize this (with benchmarks)
    pub fn consume_while(&mut self, pred: impl Fn(char) -> bool) -> &'a str {
        let start = self.location.byte as usize;
        while let Some(ch) = self.peek_char() {
            if !pred(ch) {
                let end = self.location.byte as usize;
                return &self.source_code[start..end];
            }
            self.consume_char();
        }
        &self.source_code[start..]
    }

    pub fn emit(&mut self, token_data: TokenData, start_position: Location) {
        let token = Token {
            data: token_data,
            start: start_position,
            end: self.location,
        };
        self.tokens.push(token);
    }

    /// [`Self::emit`] but for tokens that are always n ASCII characters wide.
    /// This also advances the current self position by n.
    /// This function will not behave properly for newlines.
    pub fn emit_n_chars(&mut self, token_data: TokenData, n: u32) {
        let start = self.location();
        self.skip_achars(n);
        self.emit(token_data, start);
    }

    /// [`Self::emit`] but for tokens that are always one ASCII character wide.
    /// This also advances the current self position by one.
    /// This function will not behave properly for newlines.
    pub fn emit_char(&mut self, token_data: TokenData) {
        self.emit_n_chars(token_data, 1);
    }

    /// [`Self::emit`] but for tokens that are always two ASCII characters wide.
    /// This also advances the current self position by two.
    /// This function will not behave properly for newlines.
    pub fn emit_two_chars(&mut self, token_data: TokenData) {
        self.emit_n_chars(token_data, 2);
    }

    fn ends_with_statement_terminator(&self) -> bool {
        let Some(token) = self.tokens.last() else {
            return true;
        };
        token.data == TokenData::Newline || token.data == TokenData::Semicolon
    }

    fn parse_newline(&mut self) {
        let start = self.location();
        self.location.byte += 1;
        self.location.line += 1;
        self.location.char = 0;
        if !self.ends_with_statement_terminator() {
            self.emit(TokenData::Newline, start)
        }
    }

    fn parse_semicolon(&mut self) {
        if !self.ends_with_statement_terminator() {
            self.emit_char(TokenData::Semicolon);
        }
    }

    fn parse_identifier(&mut self) {
        let start = self.location();
        let ident =
            self.consume_while(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch.is_ascii_digit());
        debug_assert!(!ident.is_empty());

        if let Some(keyword) = Keyword::try_from_str(ident) {
            self.emit(TokenData::Keyword(keyword), start);
        } else {
            self.emit(TokenData::Identifier(ident.to_owned()), start);
        }
    }

    fn parse_int(&mut self) -> Result<'a, ()> {
        let start = self.location();
        let digits = self.consume_while(|ch| ch.is_ascii_digit() || ch == '_');

        if self.peek_char().is_some_and(|x| x.is_ascii_alphabetic()) {
            let ch = self.consume_char().unwrap();
            let msg = format!("Invalid suffix for integer literal {ch:?}");
            return self.throw(msg, start);
        }

        let Some(integer) = parse_decimal_uint(digits) else {
            let msg = format!("Integer literal {digits} is out of u64 bounds");
            return self.throw(msg, start);
        };

        self.emit(TokenData::IntLiteral(integer), start);
        Ok(())
    }

    fn parse_hex_int(&mut self, prefix_len: u32) -> Result<'a, ()> {
        let start = self.location();
        self.skip_achars(prefix_len);
        let digits: &str = self.consume_while(|ch| ch.is_ascii_hexdigit() || ch == '_');

        if self.peek_char().is_some_and(|x| x.is_ascii_alphabetic()) {
            let ch = self.consume_char().unwrap();
            let msg = format!("Invalid suffix for hexadecimal integer literal {ch:?}");
            return self.throw(msg, start);
        }

        let Some(integer) = parse_hex_uint(digits) else {
            let msg = format!("Hexadecimal integer literal {digits} is out of u64 bounds");
            return self.throw(msg, start);
        };

        self.emit(TokenData::HexIntLiteral(integer), start);
        Ok(())
    }

    fn parse_bin_int(&mut self) -> Result<'a, ()> {
        let start = self.location();
        self.skip_achars(2);
        let digits: &str = self.consume_while(|ch| matches!(ch, '0' | '1' | '_'));
        let Some(integer) = parse_bin_uint(digits) else {
            let msg = format!("Binary integer literal {digits} is out of u64 bounds");
            return self.throw(msg, start);
        };
        self.emit(TokenData::BinIntLiteral(integer), start);
        Ok(())
    }

    fn parse_line_comment(&mut self) {
        // Consume '//'
        self.skip_achars(2);

        let start = self.location();
        let line: &str = self.consume_while(|c| c != '\n');
        let line: &str = line.strip_prefix(' ').unwrap_or(line);
        self.emit(TokenData::LineComment(line.to_owned()), start);
    }

    fn parse_block_comment(&mut self) -> Result<'a, ()> {
        // Consume '/*'
        self.skip_achars(2);

        let start = self.location;
        let code = self.remaining();
        let mut star_found = false;
        let mut end_found = false;

        for ch in code.chars() {
            if ch == '\n' {
                self.increment_line();
                star_found = false;
                continue;
            }

            self.location.byte += ch.len_utf8() as u32;
            self.location.char += 1;

            if ch == '/' && star_found {
                end_found = true;
                break;
            }
            star_found = ch == '*';
        }

        if !end_found {
            return self.throw("Block comment was never closed", start);
        }

        let end = self.location.byte - start.byte - 2;
        let comment = &code[..end as usize];
        self.emit(TokenData::BlockComment(comment.to_owned()), start);
        Ok(())
    }

    fn parse_string_literal(&mut self) -> Result<'a, ()> {
        // TODO: support @ strings, format strings
        let start = self.location();
        let delimiter = self.consume_char().unwrap();
        debug_assert!(delimiter == '"' || delimiter == '\'');

        let mut escaping: bool = false;
        let mut string = String::new();

        while let Some(char) = self.consume_char() {
            if escaping {
                match char {
                    '\n' => {
                        let msg = "String literal was never closed (on the same line)";
                        return self.throw(msg, start);
                    },
                    '"' => string.push('"'),
                    '\\' => string.push('\\'),
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    _ => {
                        let msg = format!("Invalid escape character '{char}'");
                        return self.throw(msg, start);
                    },
                }
                escaping = false;
            } else if char == delimiter {
                // String was closed.
                self.emit(TokenData::StringLiteral(string), start);
                return Ok(());
            } else if char == '\\' {
                escaping = true;
            } else {
                string.push(char);
            }
        }

        let msg = "String literal's quotation marks were never closed";
        self.throw(msg, start)
    }

    pub fn tokenize(&mut self) -> Result<'a, ()> {
        while let Some(char) = self.peek_char() {
            match char {
                '\n' => self.parse_newline(),
                c if c.is_whitespace() => {
                    self.consume_char();
                },
                'a'..='z' | 'A'..='Z' | '_' => self.parse_identifier(),
                '0' if self.next_char_is('x') => self.parse_hex_int(2)?,
                '0' if self.next_char_is('b') => self.parse_bin_int()?,
                '0'..='9' => self.parse_int()?,
                '$' => self.parse_hex_int(1)?,
                ';' => self.parse_semicolon(),
                '"' | '\'' => self.parse_string_literal()?,
                '.' => self.emit_char(TokenData::Dot),
                ',' => self.emit_char(TokenData::Comma),
                '+' => match self.peek_nth_char(1) {
                    Some('+') => self.emit_two_chars(TokenData::Increment),
                    Some('=') => self.emit_two_chars(TokenData::AssignAdd),
                    _ => self.emit_char(TokenData::Plus),
                },
                '-' => match self.peek_nth_char(1) {
                    Some('-') => self.emit_two_chars(TokenData::Decrement),
                    Some('=') => self.emit_two_chars(TokenData::AssignSub),
                    _ => self.emit_char(TokenData::Minus),
                },
                '*' if self.next_char_is('=') => self.emit_two_chars(TokenData::AssignMultiply),
                '*' => self.emit_char(TokenData::Multiply),
                '/' => match self.peek_nth_char(1) {
                    Some('/') => self.parse_line_comment(),
                    Some('*') => self.parse_block_comment()?,
                    Some('=') => self.emit_two_chars(TokenData::AssignDivide),
                    _ => self.emit_char(TokenData::Divide),
                },
                '%' if self.next_char_is('=') => self.emit_two_chars(TokenData::AssignModulus),
                '%' => self.emit_char(TokenData::Modulus),
                '(' => self.emit_char(TokenData::RoundBracketOpen),
                '[' => self.emit_char(TokenData::SquareBracketOpen),
                '{' => self.emit_char(TokenData::CurlyBracketOpen),
                ')' => self.emit_char(TokenData::RoundBracketClose),
                ']' => self.emit_char(TokenData::SquareBracketClose),
                '}' => self.emit_char(TokenData::CurlyBracketClose),
                '<' => match self.peek_nth_char(1) {
                    Some('<') => {
                        if self.peek_nth_char(2) == Some('=') {
                            self.emit_n_chars(TokenData::AssignShiftLeft, 3)
                        } else {
                            self.emit_two_chars(TokenData::ShiftLeft)
                        }
                    },
                    Some('=') => self.emit_two_chars(TokenData::LessEqual),
                    _ => self.emit_char(TokenData::Less),
                },
                '>' => match self.peek_nth_char(1) {
                    Some('>') => {
                        if self.peek_nth_char(2) == Some('=') {
                            self.emit_n_chars(TokenData::AssignShiftRight, 3)
                        } else {
                            self.emit_two_chars(TokenData::ShiftRight)
                        }
                    },
                    Some('=') => self.emit_two_chars(TokenData::GreaterEqual),
                    _ => self.emit_char(TokenData::Greater),
                },
                '=' if self.next_char_is('=') => self.emit_two_chars(TokenData::DoubleEqual),
                '=' => self.emit_char(TokenData::EqualSign),
                '!' if self.next_char_is('=') => self.emit_two_chars(TokenData::NotEqual),
                '!' => self.emit_char(TokenData::Bang),
                '?' if self.next_char_is('?') => {
                    if self.peek_nth_char(2) == Some('=') {
                        self.emit_n_chars(TokenData::AssignNullish, 3);
                    } else {
                        self.emit_two_chars(TokenData::Nullish);
                    }
                },
                '?' => self.emit_char(TokenData::Question),
                ':' => self.emit_char(TokenData::Colon),
                '~' => self.emit_char(TokenData::Tilde),
                '|' => match self.peek_char() {
                    Some('|') => self.emit_two_chars(TokenData::DoubleOr),
                    Some('=') => self.emit_two_chars(TokenData::AssignOr),
                    _ => self.emit_char(TokenData::BitOr),
                },
                '&' => match self.peek_char() {
                    Some('&') => self.emit_two_chars(TokenData::DoubleAnd),
                    Some('=') => self.emit_two_chars(TokenData::AssignAnd),
                    _ => self.emit_char(TokenData::BitAnd),
                },
                '^' => match self.peek_char() {
                    Some('^') => self.emit_two_chars(TokenData::DoubleXor),
                    Some('=') => self.emit_two_chars(TokenData::AssignXor),
                    _ => self.emit_char(TokenData::BitXor),
                },

                _ => {
                    let start = self.location();
                    self.consume_char();
                    return self.throw(format!("Unexpected character {char}"), start);
                },
            }
        }

        Ok(())
    }
}

pub fn tokenize(source_code: &'_ str) -> Result<'_, Vec<Token>> {
    validate_size(source_code)?;
    let mut lexer = Lexer::new(source_code);
    lexer.tokenize()?;
    Ok(lexer.tokens)
}

fn validate_size(source_code: &'_ str) -> Result<'_, ()> {
    let len = source_code.len();
    let max = u32::MAX as usize;
    if len > max {
        let msg = format!(
            "Source code is too long as its byte length ({len}) exceeds the maximum of {max}"
        );
        Err(CompileError {
            error: crate::Error::new(msg),
            source_code,
            start_position: Location::default(),
            end_position: Location::default(),
        })
    } else {
        Ok(())
    }
}

/// Parses an unsigned decimal integer into a [`u64`].
///
/// # Safety
/// This function assumes the following:
/// * The string is not empty
/// * The string starts with an ascii digit (0123456789)
/// * The string contains only ascii digits and underscores
///
/// # Errors
/// This function returns [`None`] if the integer overflowed.
/// (The integer will overflow if it is larger than [`u64::MAX`]).
fn parse_decimal_uint(digits: &str) -> Option<u64> {
    let bytes: &[u8] = digits.as_bytes();
    debug_assert!(bytes[0].is_ascii_digit());
    let mut acc: u64 = 0;

    for &byte in bytes {
        if byte == b'_' {
            continue;
        }
        let digit: u8 = byte.wrapping_sub(b'0');
        debug_assert!(digit < 10); // Prerequisite
        acc = acc.checked_mul(10)?.checked_add(digit as u64)?;
    }

    Some(acc)
}

/// Parses an unsigned hexadecimal integer into a [`u64`].
///
/// # Safety
/// This function assumes the following:
/// * The string is not empty
/// * The string starts with an ascii hexdigit (0123456789abcdefABCDEF)
/// * The string contains only ascii hexdigits and underscores
fn parse_hex_uint(digits: &str) -> Option<u64> {
    let bytes: &[u8] = digits.as_bytes();
    debug_assert!(bytes[0].is_ascii_hexdigit());
    let mut acc: u64 = 0;

    for &byte in bytes {
        let digit: u8 = match byte {
            b'0'..=b'9' => byte - b'0',
            b'A'..=b'F' => byte - b'A' + 10,
            b'a'..=b'f' => byte - b'a' + 10,
            b'_' => continue,
            _ => panic!("Invalid hexdigit"),
        };
        acc = acc.checked_mul(16)?.checked_add(digit as u64)?;
    }

    Some(acc)
}

/// Parses an unsigned binary integer into a [`u64`].
///
/// # Safety
/// This function assumes the following:
/// * The string is not empty
/// * The string starts with a 0 or 1
/// * The string contains only 0, 1 and _
fn parse_bin_uint(digits: &str) -> Option<u64> {
    if digits.len() as u32 > u64::BITS {
        return None;
    }

    let bytes: &[u8] = digits.as_bytes();
    debug_assert!(bytes[0] == b'0' || bytes[0] == b'1');
    let mut acc: u64 = 0;

    for &byte in bytes {
        if byte == b'_' {
            continue;
        }
        let digit = u64::from(byte == b'1');
        acc <<= 1;
        acc |= digit;
    }

    Some(acc)
}
