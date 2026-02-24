use libgm::gml::highlevel::token::lexer::tokenize_contextless;

fn ok(source_code: &str) {
    let Err(errors) = tokenize_contextless(source_code) else {
        return;
    };
    println!("{source_code}");
    for error in errors {
        println!("{error}");
    }
    panic!("Errors during lexing, see logs");
}

fn err(source_code: &str) {
    let Ok(tokens) = tokenize_contextless(source_code) else {
        return;
    };
    println!("{source_code}");
    for token in tokens {
        println!("{token:?}");
    }
    panic!("This was expected to fail but lexed successfully, see logs for produced tokens");
}

#[test]
fn empty() {
    ok("");
}

#[test]
fn whitespace() {
    ok("    \t   \r   ");
}

#[test]
fn ident() {
    ok("hello")
}

#[test]
fn ident_ws() {
    ok("  \thello\r\r\r \t ")
}

#[test]
fn hex_int_0x() {
    ok("0xdeadb245")
}

#[test]
fn hex_int_dollar() {
    ok("$76fae45")
}

#[test]
fn hex_overflow() {
    err("0x53789bbababa471238d")
}

#[test]
fn int_suffix() {
    err("512765d")
}

#[test]
fn int_sep() {
    ok("5623_532__87395____571")
}

#[test]
fn int_sep_pre() {
    err("_5623_532__87395____571")
}

#[test]
fn float() {
    ok("69420.1337")
}

#[test]
fn float_sep() {
    ok("6_9_420.1___337__")
}

#[test]
fn float_sep2() {
    ok("6_9_4_2_0__.___1337__")
}

#[test]
fn float_sep_pre() {
    err("_69420.1337")
}

#[test]
fn float_large() {
    ok("75877519857.81258921758912______________-8591")
}

#[test]
fn string_lit() {
    ok(r#" hello = "uwu :3 \n\n\t this is gms2 format"; "#)
}

#[test]
fn string_lit_inv_escape() {
    err(r#" hello = "uwu :3 \K \` \6 this is sparta"; "#)
}

#[test]
fn string_lit_unclosed_eof() {
    err(r#" hello = "uwu :3 this is unclosed; "#)
}

#[test]
fn string_lit_eof() {
    err(" hello = \"")
}

#[test]
fn string_lit_unclosed_eol() {
    err(" hello = \"uwu :3 this is unclosed; \nthis is a newline")
}

#[test]
fn line_comment1() {
    ok("this += 62316 // increments this 62316 times")
}

#[test]
fn line_comment2() {
    ok("// increments this 62316 times")
}

#[test]
fn line_comment3() {
    ok("  //increments this 62316 times    \r\t")
}

#[test]
fn line_comment_eof() {
    ok("//")
}

#[test]
fn line_comment_eol() {
    ok("this=is_sparta()\n//\n");
}

#[test]
fn semicolons() {
    ok(";\n\n\n;;;;;;;;\n\n;;;;;;;\n")
}

#[test]
fn block_comment_eof() {
    err("x <<= 62/* this is an unclosed comment ")
}

#[test]
fn block_comment() {
    ok("x <<= 62/* this is an foid closed comment */\nvery = normal")
}

#[test]
fn block_comment_eof_ok() {
    ok("x <<= 62/* this is a barely closed comment */")
}

#[test]
fn keywords() {
    ok("var _wgat = it + is;\n")
}
