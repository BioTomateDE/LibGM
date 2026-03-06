use crate::gml::highlevel::{
    node::AstNode,
    token::{Keyword, TokenData},
};

struct Block {
    statements: Vec<Statement>,
}

impl AstNode for Block {
    fn parse(ctx: &mut super::ParseContext) -> Self {
        if !ctx.assert_char_or_keyword(TokenData::CurlyBracketOpen, Keyword::Begin) {
            return;
        }
        ctx.skip_terminators();

        let mut statements = Vec::new();
        while !ctx.current_token_is_char_or_keyword(TokenData::CurlyBracketClose, Keyword::End) {
            let stmt = Statement::parse(ctx);
            statements.push(stmt);
        }

        Self { statements }
    }
}
