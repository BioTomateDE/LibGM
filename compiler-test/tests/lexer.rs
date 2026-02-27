use libgm::gml::highlevel::token::{
    Keyword as K,
    TokenData::{self, *},
    lexer::tokenize_contextless,
};
use std::process::exit;

macro_rules! tests_ok {
    [$($name:ident $code:literal [$($token:expr)*] @)*] => {
        $(
            #[test]
            fn $name() {
                let tokens: &[TokenData] = &[$($token,)*];
                assert_ok($code, tokens);
            }
        )*
    };
}

macro_rules! tests_err {
    [$($name:ident $code:literal @)*] => {
        $(
            #[test]
            fn $name() {
                assert_err($code);
            }
        )*
    };
}

fn assert_ok(code: &str, expected_tokens: &[TokenData]) {
    let tokens = match tokenize_contextless(code) {
        Ok(tokens) => tokens,
        Err(errors) => {
            println!("Errors while lexing:");
            println!("{code}");
            for error in errors {
                println!("{error}");
            }
            exit(1);
        },
    };

    let mut success = true;
    let exp_len = expected_tokens.len();
    let act_len = tokens.len();
    if act_len != exp_len {
        println!("Token length mismatch: expected {exp_len}, got {act_len}");
        success = false;
    }

    for (token, exp_data) in tokens.iter().zip(expected_tokens) {
        let act_data = &token.data;
        if act_data != exp_data {
            println!("Expected token {exp_data:?}, got {act_data:?}");
            success = false;
        }
    }

    if success { exit(0) } else { exit(1) }
}

fn assert_err(code: &str) {
    let Ok(tokens) = tokenize_contextless(code) else {
        exit(0);
    };
    println!("Expected this to fail but it lexed successfully:");
    println!("{code}");
    for token in tokens {
        println!("{:?}", token.data);
    }
    exit(1);
}

tests_ok![
    ok_empty
    ""
    []
@
    ok_whitespace
    "   \t  \r \t   "
    []
@
    ok_whitespace_nl
    "   \t  \r \t  \n    \t   \r"
    []
@
    ok_line_comment
    "  \r\r\t // hello world \r  \t"
    []
@
    ok_line_comment_start
    "// hello world \r  \t"
    []
@
    ok_line_comment_newline
    "// hello world \t  \r\n   \t "
    []
@
    ok_line_comment_empty
    "\n//\n"
    [LineComment("".to_owned())]
@
    ok_block_comment_empty
    "/**/"
    [BlockComment("".to_owned())]
@
    ok_semicolon
    "     ;  \r   \n  \n ;  \t "
    []
@
    ok_ident
    "hello"
    [Identifier("hello".to_owned())]
@
    ok_ident_ws
    "  \r\t hello \t  "
    [Identifier("hello".to_owned())]
@
    ok_int
    "657821"
    [IntLiteral(657821)]
@
    ok_int_under
    "657_821"
    [IntLiteral(657821)]
@
    ok_int_under_multi
    "65__7___82_1"
    [IntLiteral(657821)]
@
    ok_int_under_end
    "657821___"
    [IntLiteral(657821)]
@
    ok_ident_under_digit
    "__123_45"
    [Identifier("__123_45".to_owned())]
@
    ok_float
    "1234.5678"
    [FloatLiteral(1234.5678)]
@
    ok_float_sep
    "1__23__4.5__6__78__"
    [FloatLiteral(1234.5678)]
@
    ok_float_sep_predot
    "1234_.5678"
    [FloatLiteral(1234.5678)]
@
    ok_float_no_lead
    ".5678"
    [FloatLiteral(0.5678)]
@
    ok_bigint_int
    "18446744073709551615"
    [IntLiteral(18446744073709551615)]
@
    ok_bigint_float
    "18446744073709551616"
    [FloatLiteral(18446744073709551616.0)]
@
    ok_hex
    "0xdd1a71e5"
    [HexIntLiteral(0xdd1a71e5)]
@
    ok_hex_sep
    "0xdd_1a_71_e5"
    [HexIntLiteral(0xdd1a71e5)]
@
    ok_hex_sep_multi
    "0x__d__d_1a____71__e__5__"
    [HexIntLiteral(0xdd1a71e5)]
@
    ok_hex_mixed_case
    "0xdD1a71E5"
    [HexIntLiteral(0xdd1a71e5)]
@
    ok_hex_dollar
    "$dd1a71e5"
    [HexIntLiteral(0xdd1a71e5)]
@
    ok_keyword
    "var"
    [Keyword(K::Var)]
@
    ok_string
    r#" "this is gms2 format :3"; "#
    [StringLiteral("this is gms2 format :3".to_owned())]
@
    ok_string_escape
    r#" "this\t is\t gms2 format :3\n"; "#
    [StringLiteral("this\t is\t gms2 format :3\n".to_owned())]
@
    ok_string_verbatim
    r#" @"this is a raw string format \n \t"; "#
    [StringLiteral("this is a raw string format \\n \\t".to_owned())]
@
    ok_string_single_quotes
    " 'this is (technically unsupported) gms2 format :3'; "
    [StringLiteral("this is (technically unsupported) gms2 format :3".to_owned())]
@
    ok_string_verbatim_multiline
    " @\"this verbatim string lit\nspans multiple \nlines\"; "
    [StringLiteral("this verbatim string literal\nspans multiple \nlines".to_owned())]
@
];

tests_err![
    err_float_no_tail
    "1234."
@
    err_int_suffix
    "1337the"
@
    err_hex_overflow
    "0x53789bbababa471238d"
@
    err_string_inv_escape
    r#" "this is an invalid \string"; "#
@
    err_string_unclosed_eol
    r#" "this is an unclosed string \n bad newline"; "#
@
    err_string_unclosed_eof
    r#" "this is an unclosed string"#
@
    err_string_verbatim_unclosed_eof
    r#" @"this is an unclosed string"#
@
    err_string_verbatim_quote_escape
    r#" @"this is \verbatim str\ing but u cant \"escape\" quotes here" "#
@
    err_block_comment_unclosed
    "hello this is /* an unclosed comment\n(still not closed)"
@
];
