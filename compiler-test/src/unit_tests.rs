use libgm::gml::highlevel::token::{
    Keyword as K,
    TokenData::{self, *},
    lexer::tokenize_contextless,
};
use std::{
    any::Any,
    panic::{RefUnwindSafe, catch_unwind},
    process,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

static CURRENT_TEST: Mutex<Option<(&'static str, Instant)>> = Mutex::new(None);

fn detect_timeouts() -> ! {
    const SLEEP_DUR: Duration = Duration::from_millis(100);
    const TIMEOUT_DUR: Duration = Duration::from_millis(2000);

    loop {
        thread::sleep(SLEEP_DUR);
        let guard = CURRENT_TEST.lock().expect("Could not lock mutex");
        let Some((name, start_time)) = &*guard else {
            continue;
        };
        if start_time.elapsed() < TIMEOUT_DUR {
            continue;
        }

        println!("Test {name:?} timed out after {TIMEOUT_DUR:?}; probably in an infinite loop.");
        println!("Use a debugger like GDB (or plain logging) to find out the cause.");
        println!("The program will now be aborted.");
        process::exit(61);
    }
}

trait Test: RefUnwindSafe {
    #[must_use]
    fn name(&self) -> &'static str;

    #[must_use]
    fn run(&self) -> bool;
}

struct OkTest {
    name: &'static str,
    code: &'static str,
    expected_tokens: Vec<TokenData>,
}

impl OkTest {
    #[must_use]
    fn new(name: &'static str, code: &'static str, expected_tokens: Vec<TokenData>) -> Self {
        Self { name, code, expected_tokens }
    }
}

impl Test for OkTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&self) -> bool {
        let expected_tokens: &[TokenData] = &self.expected_tokens;
        let tokens = match tokenize_contextless(self.code) {
            Ok(tokens) => tokens,
            Err(errors) => {
                println!("Errors while lexing:");
                println!("{}", self.code);
                for error in errors {
                    println!("{error}");
                }
                return false;
            },
        };

        let matches = tokens.iter().map(|t| &t.data).eq(expected_tokens);

        if !matches {
            println!("Index | Expected Token | Actual Token");

            let placeholder = "---------";
            let end = tokens.len().max(expected_tokens.len());

            for i in 0..end {
                let expected = if let Some(td) = expected_tokens.get(i) {
                    format!("{td:?}")
                } else {
                    placeholder.to_owned()
                };
                let actual = if let Some(t) = tokens.get(i) {
                    format!("{:?}", t.data)
                } else {
                    placeholder.to_owned()
                };
                println!("{i} | {expected} | {actual}");
            }
        }

        matches
    }
}

struct ErrTest {
    name: &'static str,
    code: &'static str,
}

impl ErrTest {
    #[must_use]
    fn new(name: &'static str, code: &'static str) -> Self {
        Self { name, code }
    }
}

impl Test for ErrTest {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&self) -> bool {
        let code: &str = self.code;
        let Ok(tokens) = tokenize_contextless(code) else {
            return true;
        };
        println!("Expected this to fail but it lexed successfully:");
        println!("{code}");
        for token in tokens {
            println!("{:?}", token.data);
        }
        false
    }
}

fn extract_panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
    if let Some(string) = payload.downcast_ref::<&str>() {
        string.to_string()
    } else if let Ok(string) = payload.downcast::<String>() {
        *string
    } else {
        "Unknown panic value".to_string()
    }
}

fn run_tests<T: Test>(tests: &[T]) {
    let mut successful = 0;
    let mut panicked = 0;
    let total = tests.len();

    for test in tests {
        let name: &'static str = test.name();
        //println!("Running Test {name:?}...");

        let mut guard = CURRENT_TEST.lock().unwrap();
        *guard = Some((name, Instant::now()));
        drop(guard);

        match catch_unwind(|| test.run()) {
            Ok(true) => successful += 1,
            Ok(false) => {
                println!("The above test {name:?} was unsuccessful.\n");
            },
            Err(e) => {
                let payload = extract_panic_message(e);
                eprintln!("Test {name:?} panicked: {payload}\n");
                panicked += 1;
            },
        }

        let mut guard = CURRENT_TEST.lock().unwrap();
        *guard = None;
    }

    println!("{successful}/{total} tests passed; {panicked} panicked.");
}

pub fn main() {
    let ok_tests: Vec<OkTest> = get_ok_tests();
    let err_tests: Vec<ErrTest> = get_err_tests();

    thread::spawn(detect_timeouts);
    println!("Running OK tests...");
    run_tests(&ok_tests);
    println!();
    println!("Running ERR tests...");
    run_tests(&err_tests);
    println!();
    println!("All done.");
}

#[rustfmt::skip]
fn get_ok_tests() -> Vec<OkTest> {
    vec![
        OkTest::new(
            "empty",
            "",
            vec![],
        ),
        OkTest::new(
            "whitespace",
            "   \t  \r \t   ",
            vec![],
        ),
        OkTest::new(
            "ok_whitespace_nl",
            "   \t  \r \t  \n    \t   \r",
            vec![],
        ),
        OkTest::new(
            "ok_line_comment",
            "  \r\r\t // hello world \r  \t",
            vec![],
        ),
        OkTest::new(
            "ok_line_comment_start",
            "// hello world \r  \t",
            vec![],
        ),
        OkTest::new(
            "ok_line_comment_newline",
            "// hello world \t  \r\n   \t ",
            vec![],
        ),
        OkTest::new(
            "ok_line_comment_empty",
            "\n//\n",
            vec![LineComment("".to_owned())],
        ),
        OkTest::new(
            "ok_block_comment_empty",
            "/**/",
            vec![BlockComment("".to_owned())],
        ),
        OkTest::new(
            "ok_semicolon",
            "     ;  \r   \n  \n ;  \t ",
            vec![],
        ),
        OkTest::new(
            "ok_ident",
            "hello",
            vec![Identifier("hello".to_owned())],
        ),
        OkTest::new(
            "ok_ident_ws",
            "  \r\t hello \t  ",
            vec![Identifier("hello".to_owned())],
        ),
        OkTest::new(
            "ok_int",
            "657821",
            vec![IntLiteral(657821)],
        ),
        OkTest::new(
            "ok_int_under",
            "657_821",
            vec![IntLiteral(657821)],
        ),
        OkTest::new(
            "ok_int_under_multi",
            "65__7___82_1",
            vec![IntLiteral(657821)],
        ),
        OkTest::new(
            "ok_int_under_end",
            "657821___",
            vec![IntLiteral(657821)],
        ),
        OkTest::new(
            "ok_ident_under_digit",
            "__123_45",
            vec![Identifier("__123_45".to_owned())],
        ),
        OkTest::new(
            "ok_float",
            "1234.5678",
            vec![FloatLiteral(1234.5678)],
        ),
        OkTest::new(
            "ok_float_sep",
            "1__23__4.5__6__78__",
            vec![FloatLiteral(1234.5678)],
        ),
        OkTest::new(
            "ok_float_sep_predot",
            "1234_.5678",
            vec![FloatLiteral(1234.5678)],
        ),
        OkTest::new(
            "ok_float_no_lead",
            ".5678",
            vec![FloatLiteral(0.5678)],
        ),
        OkTest::new(
            "ok_bigint_int",
            "18446744073709551615",
            vec![IntLiteral(18446744073709551615)],
        ),
        OkTest::new(
            "ok_bigint_float",
            "18446744073709551616",
            vec![FloatLiteral(18446744073709551616.0)],
        ),
        OkTest::new(
            "ok_hex",
            "0xdd1a71e5",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_sep",
            "0xdd_1a_71_e5",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_sep_multi",
            "0x__d__d_1a____71__e__5__",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_long",
            "0xf9c07540f545eabe",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_long_sep",
            "0x___f_9____c__0754______________0f545ea___b_e___",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_mixed_case",
            "0xdD1a71E5",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_hex_dollar",
            "$dd1a71e5",
            vec![HexIntLiteral(0xdd1a71e5)],
        ),
        OkTest::new(
            "ok_keyword",
            "var",
            vec![Keyword(K::Var)],
        ),
        OkTest::new(
            "ok_string",
            r#" "this is gms2 format :3"; "#,
            vec![StringLiteral("this is gms2 format :3".to_owned())],
        ),
        OkTest::new(
            "ok_string_escape",
            r#" "this\t is\t gms2 format :3\n"; "#,
            vec![StringLiteral("this\t is\t gms2 format :3\n".to_owned())],
        ),
        OkTest::new(
            "ok_string_verbatim",
            r#" @"this is a raw string format \n \t"; "#,
            vec![StringLiteral("this is a raw string format \\n \\t".to_owned())],
        ),
        OkTest::new(
            "ok_string_single_quotes",
            " 'this is (technically unsupported) gms2 format :3'; ",
            vec![StringLiteral("this is (technically unsupported) gms2 format :3".to_owned())],
        ),
        OkTest::new(
            "ok_string_verbatim_multiline",
            " @\"this verbatim string lit\nspans multiple \nlines\"; ",
            vec![StringLiteral("this verbatim string literal\nspans multiple \nlines".to_owned())],
        ),
    ]
}

#[rustfmt::skip]
fn get_err_tests() -> Vec<ErrTest> {
    vec![
        ErrTest::new(
            "err_float_no_tail",
            "1234.",
        ),
        ErrTest::new(
            "err_int_suffix",
            "1337the",
        ),
        ErrTest::new(
            "err_hex_overflow",
            "0x53789bbababa471238d",
        ),
        ErrTest::new(
            "err_string_inv_escape",
            r#" "this is an invalid \string"; "#,
        ),
        ErrTest::new(
            "err_string_unclosed_eol",
            r#" "this is an unclosed string \n bad newline"; "#,
        ),
        ErrTest::new(
            "err_string_unclosed_eof",
            r#" "this is an unclosed string"#,
        ),
        ErrTest::new(
            "err_string_verbatim_unclosed_eof",
            r#" @"this is an unclosed string"#,
        ),
        ErrTest::new(
            "err_string_verbatim_quote_escape",
            r#" @"this is \verbatim str\ing but u cant \"escape\" quotes here" "#,
        ),
        ErrTest::new(
            "err_block_comment_unclosed",
            "hello this is /* an unclosed comment\n(still not closed)",
        ),
    ]
}
