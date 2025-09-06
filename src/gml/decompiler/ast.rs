#[derive(Debug, Clone)]
pub enum Statement {
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Vec<Statement>,
    },
    Loop {
        /// Used in for loops
        init: Option<Expression>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Vec<Statement>,
    },
    Block(Vec<Statement>),
    // ... other statements ...
}

#[derive(Debug, Clone)]
pub enum Expression {
    // todo: stub
}

