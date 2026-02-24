use std::{
    fs::{read_dir, read_to_string},
    path::Path,
};

use libgm::gml::highlevel::token::lexer::LexerContext;

type Res = Result<(), Box<dyn std::error::Error>>;

fn test_all() -> Res {
    println!("Testing all...");
    let dir = Path::new("/tmp/gmdecomp/");
    let ctx1 = LexerContext { has_string_escaping: false };
    let ctx2 = LexerContext { has_string_escaping: true };

    for (ctx, dirname) in [(ctx1, "gms1"), (ctx2, "gms2")] {
        let dir = dir.join(dirname);
        for entry in read_dir(dir)? {
            let dir = entry?.path();
            let name = dir.file_name().unwrap().to_string_lossy();
            println!("Testing {name} ({dirname})...");
            for entry in read_dir(dir)? {
                let path = entry?.path();
                let source_code = read_to_string(&path)?;

                tokenize(&source_code, &ctx, &path)?;
            }
        }
    }

    println!("All done!");
    Ok(())
}

fn tokenize(code: &str, ctx: &LexerContext, path: &Path) -> Result<(), &'static str> {
    let Err(errors) = ctx.tokenize(code) else {
        return Ok(());
    };
    for error in errors {
        println!("Path: {path:?}");
        println!("{error}");
        println!("{code}");
    }
    Err("Errors during lexing, see logs")
}

fn main() -> Res {
    test_all()
}
