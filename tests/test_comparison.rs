use colored::*;
use move_command_line_common::files::FileHash;
use move_compiler::parser::{lexer::Lexer, syntax::parse_file_string};
use movefmt::{core::token_tree::TokenTree, tools::utils::*};
use std::path::Path;
use tracing_subscriber::EnvFilter;

// Import both formatters
use movefmt::core::fmt::format_entry as format_entry_original;
use movefmt::core::fmt_state::format_entry_functional;

/// Print colored diff between two texts
fn print_colored_diff(original: &str, functional: &str) {
    let original_lines: Vec<&str> = original.lines().collect();
    let functional_lines: Vec<&str> = functional.lines().collect();

    // Simple line-by-line diff
    let max_lines = original_lines.len().max(functional_lines.len());

    println!(
        "{}",
        "┌────────────────────────────────────────────────────────────────────────┐".blue()
    );
    println!(
        "{}",
        "│                             DIFF OUTPUT                              │".blue()
    );
    println!(
        "{}",
        "├────────────────────────────────────────────────────────────────────────┤".blue()
    );

    for i in 0..max_lines {
        match (original_lines.get(i), functional_lines.get(i)) {
            (Some(orig_line), Some(func_line)) => {
                if orig_line == func_line {
                    println!(
                        "{} {:4} {}",
                        "│".blue(),
                        (i + 1).to_string().white(),
                        orig_line
                    );
                } else {
                    println!(
                        "{} {:4} {}",
                        "│-".red(),
                        (i + 1).to_string().white(),
                        orig_line.red()
                    );
                    println!(
                        "{} {:4} {}",
                        "│+".green(),
                        (i + 1).to_string().white(),
                        func_line.green()
                    );
                }
            }
            (Some(orig_line), None) => {
                println!(
                    "{} {:4} {}",
                    "│-".red(),
                    (i + 1).to_string().white(),
                    orig_line.red()
                );
            }
            (None, Some(func_line)) => {
                println!(
                    "{} {:4} {}",
                    "│+".green(),
                    (i + 1).to_string().white(),
                    func_line.green()
                );
            }
            (None, None) => break,
        }
    }

    println!(
        "{}",
        "└────────────────────────────────────────────────────────────────────────┘".blue()
    );
}

fn test_formatter_comparison_on_file(p: impl AsRef<Path>) -> bool {
    let p = p.as_ref();
    eprintln!("Comparing formatters on file: {:?}", p);
    let content_origin = std::fs::read_to_string(&p).unwrap();

    // Parse check
    match parse_file_string(&mut get_compile_env(), FileHash::empty(), &content_origin) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("file '{:?}' skipped because of parse not ok", p);
            return false;
        }
    }

    // Test both formatters
    test_formatter_comparison(&content_origin, p);
    true
}

fn test_formatter_comparison(content_origin: &str, p: impl AsRef<Path>) {
    let p = p.as_ref();

    // Format with original formatter
    let result_original = format_entry_original(content_origin, commentfmt::Config::default());
    let content_original = match result_original {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Original formatter failed on {:?}: {:?}", p, e);
            return;
        }
    };

    // Format with functional formatter
    let result_functional = format_entry_functional(content_origin, commentfmt::Config::default());
    let content_functional = match result_functional {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Functional formatter failed on {:?}: {:?}", p, e);
            return;
        }
    };

    // Compare results
    if content_original.trim() != content_functional.trim() {
        eprintln!("Formatter outputs differ for file: {:?}", p);
        print_colored_diff(&content_original, &content_functional);

        // Save outputs for inspection
        let original_path = format!("{}.original.out", p.to_string_lossy());
        let functional_path = format!("{}.functional.out", p.to_string_lossy());
        std::fs::write(&original_path, &content_original).ok();
        std::fs::write(&functional_path, &content_functional).ok();
        eprintln!("Outputs saved to {} and {}", original_path, functional_path);
    } else {
        eprintln!("✅ Formatters produce identical output for {:?}", p);
    }

    // Extract tokens for detailed comparison
    let tokens_original = extract_tokens(&content_original);
    let tokens_functional = extract_tokens(&content_functional);

    match (tokens_original, tokens_functional) {
        (Ok(tokens_orig), Ok(tokens_func)) => {
            if tokens_orig.len() != tokens_func.len() {
                eprintln!(
                    "❌ Token count differs: original={} functional={}",
                    tokens_orig.len(),
                    tokens_func.len()
                );
                return;
            }

            for (i, (t1, t2)) in tokens_orig.iter().zip(tokens_func.iter()).enumerate() {
                if t1.content != t2.content {
                    eprintln!(
                        "❌ Token {} differs: original='{}' functional='{}'",
                        i, t1.content, t2.content
                    );
                    return;
                }
            }
            eprintln!("✅ Token comparison passed for {:?}", p);
        }
        (Err(e1), Err(e2)) => {
            eprintln!(
                "Both formatters produced unparseable output: original={:?}, functional={:?}",
                e1, e2
            );
        }
        (Err(e), _) => {
            eprintln!("Original formatter produced unparseable output: {:?}", e);
        }
        (_, Err(e)) => {
            eprintln!("Functional formatter produced unparseable output: {:?}", e);
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct ExtractToken {
    content: String,
    line: u32,
    col: u32,
}

fn extract_tokens(content: &str) -> Result<Vec<ExtractToken>, Vec<String>> {
    let p = Path::new(".").to_path_buf();
    let mut line_mapping = FileLineMapping::default();
    line_mapping.update(p.clone(), &content);
    let filehash = FileHash::empty();
    let (defs, _comments) = match parse_file_string(&mut get_compile_env(), filehash, content) {
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
                    format_args!("{}\n{}", msg, m)
                ));
            }
            return Err(ret);
        }
    };
    let lexer = Lexer::new(content, filehash);
    let mut ret = Vec::new();
    let parse = movefmt::core::token_tree::Parser::new(lexer, &defs, content);
    let token_tree = parse.parse_tokens();
    let mut line_mapping = FileLineMapping::default();
    line_mapping.update(p.to_path_buf(), content);
    fn collect_token_tree(ret: &mut Vec<ExtractToken>, m: &FileLineMapping, t: &TokenTree) {
        match t {
            TokenTree::SimpleToken { content, pos, .. } => {
                let loc = m
                    .translate(&Path::new(".").to_path_buf(), *pos, *pos)
                    .unwrap();

                if content != "," {
                    ret.push(ExtractToken {
                        content: content.clone(),
                        line: loc.line_start,
                        col: loc.col_start,
                    });
                }
            }
            TokenTree::Nested { elements, kind, .. } => {
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
fn test_single_file_comparison() {
    eprintln!("================== test_single_file_comparison ===================");
    test_formatter_comparison_on_file("./tests/complex/input1.move");
}
