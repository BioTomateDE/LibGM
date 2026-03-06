mod block;

use crate::gml::highlevel::{
    compile::CompileError,
    token::{Keyword, Token, TokenData},
};

struct ParseContext<'a> {
    tokens: Vec<Token<'a>>,
    position: u32,
    source_code: &'a str,
    errors: Vec<CompileError<'a>>,
}

impl<'a> ParseContext<'a> {
    /// Pushes a new [`CompileError`] to `self.errors`.
    pub fn throw(&mut self, error: impl Into<crate::Error>, associated_token: &Token) {
        let error = CompileError {
            error: error.into(),
            source_code: self.source_code,
            start_position: associated_token.start,
            end_position: associated_token.end,
        };
        self.errors.push(error);
    }

    #[must_use]
    pub fn reached_end(&self) -> bool {
        self.position >= self.tokens.len() as u32
    }

    #[must_use]
    fn current_token(&self) -> Option<&Token<'a>> {
        let idx = self.position as usize;
        if let Some(token) = self.tokens.get(idx) {
            Some(&token)
        } else {
            None
        }
    }

    #[must_use]
    pub fn peek_token(&self) -> Option<&TokenData<'a>> {
        self.current_token().map(|t| &t.data)
    }

    pub fn consume_token(&mut self) -> Option<&TokenData<'a>> {
        let idx = self.position as usize;
        if let Some(token) = self.tokens.get(idx) {
            self.position += 1;
            Some(&token.data)
        } else {
            None
        }
    }

    fn assert_any_token(&mut self, tokens: &[&TokenData<'a>]) -> bool {
        let token_data = self.peek_token();
        let matches = tokens.iter().any(|&t| token_data == Some(t));
        if matches {
            self.consume_token();
        } else {
            let token = self.current_token().unwrap_or(self.tokens.last()).unwrap();
            self.throw(err, token);
        }
        matches
    }

    pub fn assert_token(&mut self, token_data: impl AsRef<TokenData<'a>>) -> bool {
        self.assert_any_token(&[token_data.as_ref()])
    }

    pub fn assert_char_or_keyword(&mut self, char_token: TokenData<'a>, keyword: Keyword) -> bool {
        self.assert_any_token(&[&char_token, &TokenData::Keyword(keyword)])
    }

    #[must_use]
    pub fn current_token_is(&self, token_data: impl AsRef<TokenData<'a>>) -> bool {
        self.peek_token() == Some(token_data.as_ref())
    }

    #[must_use]
    pub fn current_token_is_any(&self, tokens: &[TokenData<'a>]) -> bool {
        let Some(token) = self.peek_token() else {
            return false;
        };
        tokens.iter().any(|t| t == token)
    }

    #[must_use]
    pub fn current_token_is_char_or_keyword(
        &self,
        char_token: TokenData<'a>,
        keyword: Keyword,
    ) -> bool {
        self.current_token_is_any(&[char_token, TokenData::Keyword(keyword)])
    }

    pub fn skip_terminators(&mut self) {
        let Some(token) = self.peek_token() else {
            return;
        };
        if *token == TokenData::Newline || *token == TokenData::Semicolon {
            self.consume_token();
        }
        // Loop not needed because the lexer skips consecutive terminators.
    }
}

struct BuildContext;
//TODO:this

trait AstNode {
    /// Creates an AST Node from a token stream.
    fn parse(ctx: &mut ParseContext) -> Self;

    // /// Turns this AST Node into a token stream.
    // fn build(&self, ctx: &mut BuildContext);
}
