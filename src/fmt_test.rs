use crate::{
    fmt::FormatConfig,
    token_tree::{CommentExtrator, CommentExtratorErr, TokenTree},
};

use super::utils::FileLineMapping;

use move_command_line_common::files::FileHash;

use move_compiler::{
    parser::{lexer::Lexer, syntax::parse_file_string},
    shared::CompilationEnv,
    Flags,
};

use std::path::{Path, PathBuf};

fn mk_result_filepath(x: &PathBuf) -> PathBuf {
    let mut x = x.clone();
    let b = x
        .components()
        .last()
        .map(|x| x.as_os_str().to_str())
        .flatten()
        .unwrap()
        .to_string();
    let index = b.as_str().rfind(".").unwrap();
    x.pop();
    let mut ret = x.clone();
    ret.push(format!(
        "{}{}",
        b.as_str()[0..index].to_string(),
        "_formatted.move"
    ));
    ret
}

#[test]
fn scan_dir() {
    let mut num: usize = 0;
    for x in walkdir::WalkDir::new(match std::env::var("MOVE_FMT_TEST_DIR") {
        Ok(x) => x,
        Err(_) => {
            eprintln!("MOVE_FMT_TEST_DIR env var not set this test skipped.");
            return;
        }
    }) {
        let x = match x {
            Ok(x) => x,
            Err(_) => todo!(),
        };
        if x.file_type().is_file() && x.file_name().to_str().unwrap().ends_with(".move") {
            let p = x.into_path();
            test_on_file(p.as_path());
            num += 1;
        }
    }
    eprintln!("formated {} files", num);
}

#[test]
fn xxx() {
    test_on_file(&Path::new(
        "/data/lzw/rust_projects/move/language/move-analyzer/tests/symbols/sources/format_case1.move",
    ));
}

#[test]
fn xxx_chen() {
    test_on_file(&Path::new(
        //"C:/I-Git/aptos-core/aptos-move/framework/aptos-framework/sources/stake.spec.move",
        //"C:/I-Git/sui/sui/sui_programmability/examples/basics/sources/lock.move",
        "C:/I-Git/sui/sui/sui_programmability/examples\\defi\\sources\\pool.move",
    ));
}
fn test_on_file(p: impl AsRef<Path>) {
    let p = p.as_ref();
    eprintln!("try format:{:?}", p);
    let content_origin = std::fs::read_to_string(&p).unwrap();
    {
        let mut env = CompilationEnv::new(Flags::testing());
        match parse_file_string(&mut env, FileHash::empty(), &content_origin) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("file '{:?}' skipped because of parse not ok", p);
                return;
            }
        }
    }
    let content_origin = std::fs::read_to_string(p).unwrap();
    test_content(content_origin.as_str(), p);
}

fn test_content(content_origin: &str, p: impl AsRef<Path>) {
    let p = p.as_ref();
    let tokens_origin =
        extract_tokens(content_origin).expect("test file should be about to lexer,err:{:?}");

    let content_format =
        super::fmt::format(content_origin, FormatConfig { indent_size: 2 }).unwrap();
    let tokens_format = match extract_tokens(content_format.as_str()) {
        Ok(x) => x,
        Err(err) => {
            unreachable!(
                "should be able to parse after format:err{:?},after format:\n\n################\n{}\n###############",
                err,  
                content_format 
            );
        }
    };
    for (t1, t2) in tokens_origin.iter().zip(tokens_format.iter()) {
        assert_eq!(
            t1.content,
            t2.content,
            "format not ok,file:{:?} line:{} col:{},after format line:{} col:{}",
            p,
            // +1 in vscode UI line and col start with 1
            t1.line + 1,
            t1.col + 1,
            t2.line + 1,
            t2.col + 1,
        );
    }
    assert_eq!(
        tokens_origin.len(),
        tokens_format.len(),
        "{:?} tokens count should equal",
        p
    );
    let comments_origin = extract_comments(&content_origin).unwrap();
    let comments_format = extract_comments(&content_format).unwrap();
    for (index, (c1, c2)) in comments_origin
        .iter()
        .zip(comments_format.iter())
        .enumerate()
    {
        assert_eq!(c1, c2, "comment {} not ok.", index);
    }
    assert_eq!(
        comments_origin.len(),
        comments_format.len(),
        "{:?} comments count should equal",
        p,
    );

    let result_file_path = mk_result_filepath(&p.to_path_buf());
    std::fs::write(result_file_path.clone(), content_format);
    // eprintln!("{:?} format ok. \n{}\n", p, content_format);
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ExtractToken {
    content: String,
    line: u32,
    col: u32,
}

fn extract_comments(content: &str) -> Result<Vec<String>, CommentExtratorErr> {
    let c = CommentExtrator::new(content)?;
    let c: Vec<_> = c
        .comments
        .into_iter()
        .map(|x| x.content)
        .map(|x| x.replacen(" ", "", usize::MAX))
        .map(|x| x.replacen("\t", "", usize::MAX))
        .map(|x| x.replacen("\n", "", usize::MAX))
        .collect();
    return Ok(c);
}

fn extract_tokens(content: &str) -> Result<Vec<ExtractToken>, Vec<String>> {
    let p = Path::new(".").to_path_buf();
    let mut line_mapping = FileLineMapping::default();
    line_mapping.update(p.clone(), &content);
    let filehash = FileHash::empty();
    let mut env = CompilationEnv::new(Flags::testing());
    let (defs, _comments) = match parse_file_string(&mut env, filehash, content) {
        Ok(x) => x,
        Err(d) => {
            let mut ret = Vec::with_capacity(d.len());
            for x in d.into_codespan_format() {
                let (_s, msg, (loc, m), _, _notes) = x;
                let loc = line_mapping.translate(&p, loc.start(), loc.end()).unwrap();
                ret.push(format!(
                    "{}:{} {}",
                    loc.line_start + 1,
                    loc.col_start + 1,
                    format!("{}\n{}", msg, m)
                ));
            }
            return Err(ret);
        }
    };
    let lexer = Lexer::new(&content, filehash);
    let mut ret = Vec::new();
    let parse = super::token_tree::Parser::new(lexer, &defs);
    let token_tree = parse.parse_tokens();
    let mut line_mapping = FileLineMapping::default();
    line_mapping.update(p.to_path_buf(), &content);
    fn collect_token_tree(ret: &mut Vec<ExtractToken>, m: &FileLineMapping, t: &TokenTree) {
        match t {
            TokenTree::SimpleToken {
                content,
                pos,
                tok: _tok,
                note: _,
            } => {
                let loc = m
                    .translate(&Path::new(".").to_path_buf(), *pos, *pos)
                    .unwrap();

                ret.push(ExtractToken {
                    content: content.clone(),
                    line: loc.line_start,
                    col: loc.col_start,
                });
            }
            TokenTree::Nested {
                elements,
                kind,
                note: _,
            } => {
                let start_loc = m
                    .translate(
                        &Path::new(".").to_path_buf(),
                        kind.start_pos,
                        kind.start_pos,
                    )
                    .unwrap();
                ret.push(ExtractToken {
                    content: format!("{}", kind.kind.start_tok()),
                    line: start_loc.line_start,
                    col: start_loc.col_start,
                });

                for token in elements.iter() {
                    collect_token_tree(ret, m, token);
                }
                let end_loc = m
                    .translate(&Path::new(".").to_path_buf(), kind.end_pos, kind.end_pos)
                    .unwrap();
                ret.push(ExtractToken {
                    content: format!("{}", kind.kind.end_tok()),
                    line: end_loc.line_start,
                    col: end_loc.col_start,
                });
            }
        }
    }
    for token in token_tree.iter() {
        collect_token_tree(&mut ret, &line_mapping, token);
    }

    Ok(ret)
}

#[test]
fn test_str() {
    test_content(
        r#"
        module 0x1::xxx { 
            public fun escaped_backslash_before_quote(): vector<u8> {
                b"\\"
            }
        }
        
            "#,
        &Path::new("."),
    );
}

#[test]
fn test_str_chen() {
    test_content(
        r#"

        module 0x1::xxx {
            fun xxx() { 
                1
            }
        }
    "#,
        &Path::new("."),
    );
}
