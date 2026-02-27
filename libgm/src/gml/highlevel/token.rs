use crate::gml::highlevel::Location;

pub mod lexer;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The actual data strored by this token.
    pub data: TokenData,

    /// This token's start position in the source code.
    start: Location,

    /// This token's end position in the source code.
    end: Location,
}

/// The token kind with its potential corresponding data.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    /// A newline '\n'.
    /// This is a statement terminator if no round/square brackets are currently open.
    Newline,

    /// A Semicolon ';'.
    /// This is always a statement terminator.
    Semicolon,

    /// A dot '.'.
    Dot,

    /// A comma ','.
    /// This is used in:
    /// * array/list literals
    /// * function call arguments
    /// * multi local declarations
    Comma,

    /// A plus sign '+'.
    /// This can be an unary no-op operator or an addition operation.
    Plus,

    /// A minus sign '-'.
    /// This can be an unary negation operator or a subtraction operation.
    Minus,

    /// A star `*`.
    /// This is always used as a multiplication operation.
    Multiply,

    /// A slash `/`.
    /// This is always used as a division operation.
    /// Comments are parsed differently
    /// (see [`TokenData::LineComment`] and [`TokenData::BlockComment`]).
    Divide,

    /// The modulo operator `%`.
    /// Equivalent to the `mod` keyword.
    /// Similar to the `div` keyword.
    /// This performs euclidean division.
    /// Compiles to the `mod` instruction.
    Modulus,

    /// A left shift operator `<<`.
    ShiftLeft,

    /// A right shift operator `>>`.
    ShiftRight,

    /// The `&` operator.
    /// Used for bitwise operations.
    BitAnd,

    /// The `|` operator.
    /// Used for bitwise operations.
    BitOr,

    /// The `^` operator.
    /// Used for bitwise operations.
    BitXor,

    /// `+=`
    AssignAdd,

    /// `-=`
    AssignSub,

    /// `*=`
    AssignMultiply,

    /// `/=`
    AssignDivide,

    /// `%=`
    AssignModulus,

    /// `<<=`
    AssignShiftLeft,

    /// `>>=`
    AssignShiftRight,

    /// `&=`.
    /// Used for bitwise operations.
    AssignAnd,

    /// `|=`.
    /// Used for bitwise operations.
    AssignOr,

    /// `^=`.
    /// Used for bitwise operations.
    AssignXor,

    /// The increment operator `++`.
    /// This can either be a pre-increment `foo++`
    /// or a post-increment `++foo`.
    Increment,

    /// The decrement operator `--`.
    /// This can either be a pre-decrement `foo--`
    /// or a post-edcrement `--foo`.
    Decrement,

    /// The `&&` operator.
    /// Equivalent to the `and` keyword.
    /// Used for boolean operations.
    DoubleAnd,

    /// The `||` operator .
    /// Equivalent to the `or` keyword.
    /// Used for boolean operations.
    DoubleOr,

    /// The `^^` operator.
    /// Equivalent to the `xor` keyword.
    /// Used for boolean operations.
    DoubleXor,

    /// An exclamation mark `!`.
    /// This is used for negating bools (or ints).
    /// The inequality operator is parsed separately as [`TokenData::NotEqual`].
    Bang,

    /// The bitwise negation operator `~`.
    Tilde,

    /// The tenary `?` operator.
    Question,

    /// The tenary `:` operator.
    Colon,

    /// The nullish coalesing operator `??`.
    Nullish,

    /// The assignment nullish coalesing operator `??=`.
    AssignNullish,

    /// A less than sign '<'.
    /// This is used for ordinal comparisons.
    /// This token is parsed separately from [`TokenData::LessEqual`].
    Less,

    /// A less or equal sign '<='.
    /// This is used for ordinal comparisons.
    LessEqual,

    /// A greater than sign '>'.
    /// This is used for ordinal comparisons.
    /// This token is parsed separately from [`TokenData::GreaterEqual`].
    Greater,

    /// A greater or equal sign '>='.
    /// This is used for ordinal comparisons.
    GreaterEqual,

    /// A single equality sign `=`.
    /// This is only used for assignments.
    /// `>=`, `<=`, `!=`, `==` use different tokens.
    EqualSign,

    /// Two equality signs `==`.
    /// This is used for equality comparisons.
    DoubleEqual,

    /// An exclamation mark and an equality sign `!=`.
    /// This is used for inequality comparisons.
    NotEqual,

    /// An open round bracket `(`.
    /// This is used for function calls.
    /// It can also be used in expressions to specify a custom order-of-operations.
    RoundBracketOpen,

    /// A closed round bracket `)`.
    /// See [`TokenData::RoundBracketOpen`] for more.
    RoundBracketClose,

    /// An open square bracket `[`.
    /// This is used for list/array literals and indexing.
    SquareBracketOpen,

    /// A closed square bracket `]`.
    /// See [`TokenData::SquareBracketOpen`] for more.
    SquareBracketClose,

    /// An open curly bracket `{`.
    /// This is used for blocks.
    CurlyBracketOpen,

    /// A closed curly bracket `}`.
    /// This is used for blocks.
    CurlyBracketClose,

    /// A comment spanning a full line.
    /// Starts with `//` and ends at the next line break.
    LineComment(String),

    /// A comment spanning less than one line or multiple lines.
    /// Starts with `/*` and ends at the next `*/`.
    BlockComment(String),

    /// A GameMaker identifer.
    /// This can be a function name, asset name, variable name, etc.
    /// This **cannot** be a reserved keyword.
    Identifier(String),

    /// A standard string literal, denoted by double or single quotation marks.
    StringLiteral(String),

    /// A raw/verbatim string literal, denoted by a prefixed `@`.
    /// This is also always used in GMS1 since escaping did not exist back then.
    RawStringLiteral(String),

    BinIntLiteral(u64),
    HexIntLiteral(u64),
    CssColorLiteral(u32),
    IntLiteral(u64),
    FloatLiteral(f64),

    Keyword(Keyword),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    If,
    Then,
    Else,
    Switch,
    Case,
    Default,

    // Equivalent to `{`.
    Begin,

    // Equivalent to `}`.
    End,

    Break,
    Continue,
    Exit,
    Return,

    While,
    For,
    Repeat,
    Do,
    Until,
    With,

    Var,

    /// Equivalent to the `%` operator.
    /// Similar to `div`.
    /// This performs euclidean division.
    /// Compiles to the `mod` instruction.
    Mod,

    /// Similar to `mod` / `%`.
    /// This does not perform euclidean division.
    /// Compiles to the `rem` instruction.
    Div,

    /// Equivalent to the boolean `&&` operator.
    And,

    /// Equivalent to the boolean `||` operator.
    Or,

    /// Equivalent to the boolean `^^` operator.
    Xor,

    Enum,

    Try,
    Catch,
    Finally,
    Throw,

    New,
    Delete,
    Function,
    Static,
}

impl Keyword {
    #[must_use]
    pub fn try_from_str(string: &str, gmlv2: bool) -> Option<Self> {
        Some(match string {
            "if" => Self::If,
            "then" => Self::Then,
            "else" => Self::Else,
            "switch" => Self::Switch,
            "case" => Self::Case,
            "default" => Self::Default,
            "begin" => Self::Begin,
            "end" => Self::End,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "exit" => Self::Exit,
            "return" => Self::Return,
            "while" => Self::While,
            "for" => Self::For,
            "repeat" => Self::Repeat,
            "do" => Self::Do,
            "until" => Self::Until,
            "with" => Self::With,
            "var" => Self::Var,
            "mod" => Self::Mod,
            "div" => Self::Div,
            "and" => Self::And,
            "or" => Self::Or,
            "xor" => Self::Xor,
            "enum" => Self::Enum,
            "try" if gmlv2 => Self::Try,
            "catch" if gmlv2 => Self::Catch,
            "finally" if gmlv2 => Self::Finally,
            "throw" if gmlv2 => Self::Throw,
            "new" if gmlv2 => Self::New,
            "delete" if gmlv2 => Self::Delete,
            "function" if gmlv2 => Self::Function,
            "static" if gmlv2 => Self::Static,
            _ => return None,
        })
    }

    #[must_use]
    pub fn to_str(self) -> &'static str {
        match self {
            Self::If => "if",
            Self::Then => "then",
            Self::Else => "else",
            Self::Switch => "switch",
            Self::Case => "case",
            Self::Default => "default",
            Self::Begin => "begin",
            Self::End => "end",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Exit => "exit",
            Self::Return => "return",
            Self::While => "while",
            Self::For => "for",
            Self::Repeat => "repeat",
            Self::Do => "do",
            Self::Until => "until",
            Self::With => "with",
            Self::Var => "var",
            Self::Mod => "mod",
            Self::Div => "div",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Enum => "enum",
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Finally => "finally",
            Self::Throw => "throw",
            Self::New => "new",
            Self::Delete => "delete",
            Self::Function => "function",
            Self::Static => "static",
        }
    }
}
