// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::core::token_tree::{NestKind, NestKind_, TokenTree};
use crate::tools::utils::*;
use commentfmt::Config;
use move_command_line_common::files::FileHash;
use move_compiler::parser::ast::*;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::syntax::parse_file_string;
use move_compiler::shared::Identifier;
use move_ir_types::location::*;

use super::syntax_trait::{Preprocessor, SingleSyntaxExtractor};

#[derive(Clone, Debug, Default)]
pub struct FunHandler {
    pub loc_vec: Vec<Loc>,
    pub para_span_vec: Vec<Loc>,
    pub ret_ty_loc_vec: Vec<Loc>,
    pub body_loc_vec: Vec<Loc>,
    pub loc_line_vec: Vec<(u32, u32)>,
    pub line_mapping: FileLineMappingOneFile,
    pub source: String,
}

impl SingleSyntaxExtractor for FunHandler {
    fn new(fmt_buffer: &str) -> Self {
        let mut this_fun_extractor = Self {
            loc_vec: vec![],
            para_span_vec: vec![],
            ret_ty_loc_vec: vec![],
            body_loc_vec: vec![],
            loc_line_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
            source: fmt_buffer.to_string(),
        };

        this_fun_extractor.line_mapping.update(&fmt_buffer);
        this_fun_extractor
    }

    fn collect_seq_item(&mut self, _s: &SequenceItem) {}

    fn collect_seq(&mut self, _s: &Sequence) {}

    fn collect_spec(&mut self, spec_block: &SpecBlock) {
        if let SpecBlockTarget_::Member(_member_name, Some(signature)) =
            &spec_block.value.target.value
        {
            let start_line = self
                .line_mapping
                .translate(
                    spec_block.value.target.loc.start(),
                    spec_block.value.target.loc.start(),
                )
                .unwrap()
                .start
                .line;
            let end_line = self
                .line_mapping
                .translate(
                    spec_block.value.target.loc.end(),
                    spec_block.value.target.loc.end(),
                )
                .unwrap()
                .start
                .line;

            self.loc_vec.push(spec_block.value.target.loc);

            if signature.parameters.is_empty() {
                self.para_span_vec.push(Loc::new(FileHash::empty(), 0, 0));
            } else {
                let first_para_loc = signature.parameters.first().unwrap().0.loc();
                let last_para_loc = signature.parameters.last().unwrap().0.loc();
                self.para_span_vec.push(Loc::new(
                    first_para_loc.file_hash(),
                    first_para_loc.start(),
                    last_para_loc.end(),
                ));
            }

            if let Type_::Unit = signature.return_type.value {
                self.ret_ty_loc_vec.push(Loc::new(FileHash::empty(), 0, 0));
            } else {
                self.ret_ty_loc_vec.push(signature.return_type.loc);
            }

            self.body_loc_vec.push(Loc::new(FileHash::empty(), 0, 0));
            self.loc_line_vec.push((start_line, end_line));
        }

        for m in spec_block.value.members.iter() {
            if let SpecBlockMember_::Function {
                uninterpreted: _,
                name: _,
                signature,
                body,
            } = &m.value
            {
                if let FunctionBody_::Defined(..) = &body.value {
                    let start_line = self
                        .line_mapping
                        .translate(m.loc.start(), m.loc.start())
                        .unwrap()
                        .start
                        .line;
                    let end_line = self
                        .line_mapping
                        .translate(m.loc.end(), m.loc.end())
                        .unwrap()
                        .start
                        .line;

                    self.loc_vec.push(m.loc);

                    if signature.parameters.is_empty() {
                        self.para_span_vec.push(Loc::new(FileHash::empty(), 0, 0));
                    } else {
                        let first_para_loc = signature.parameters.first().unwrap().0.loc();
                        let last_para_loc = signature.parameters.last().unwrap().0.loc();
                        self.para_span_vec.push(Loc::new(
                            first_para_loc.file_hash(),
                            first_para_loc.start(),
                            last_para_loc.end(),
                        ));
                    }

                    if let Type_::Unit = signature.return_type.value {
                        self.ret_ty_loc_vec.push(Loc::new(FileHash::empty(), 0, 0));
                    } else {
                        self.ret_ty_loc_vec.push(signature.return_type.loc);
                    }

                    self.body_loc_vec.push(body.loc);
                    self.loc_line_vec.push((start_line, end_line));
                }
            }
        }
    }

    fn collect_expr(&mut self, _e: &Exp) {}

    fn collect_const(&mut self, _c: &Constant) {}

    fn collect_struct(&mut self, _s: &StructDefinition) {}

    fn collect_function(&mut self, d: &Function) {
        let start_line = self
            .line_mapping
            .translate(d.loc.start(), d.loc.start())
            .unwrap()
            .start
            .line;
        let end_line = self
            .line_mapping
            .translate(d.loc.end(), d.loc.end())
            .unwrap()
            .start
            .line;
        self.loc_vec.push(d.loc);

        if d.signature.parameters.is_empty() {
            self.para_span_vec.push(Loc::new(FileHash::empty(), 0, 0));
        } else {
            let first_para_loc = d.signature.parameters.first().unwrap().0.loc();
            let last_para_loc = d.signature.parameters.last().unwrap().0.loc();
            self.para_span_vec.push(Loc::new(
                first_para_loc.file_hash(),
                first_para_loc.start(),
                last_para_loc.end(),
            ));
        }

        if let Type_::Unit = d.signature.return_type.value {
            self.ret_ty_loc_vec.push(Loc::new(FileHash::empty(), 0, 0));
        } else {
            self.ret_ty_loc_vec.push(d.signature.return_type.loc);
        }
        self.body_loc_vec.push(d.body.loc);
        self.loc_line_vec.push((start_line, end_line));
    }

    fn collect_module(&mut self, d: &ModuleDefinition) {
        for m in d.members.iter() {
            match m {
                ModuleMember::Function(x) => self.collect_function(x),
                ModuleMember::Spec(s) => self.collect_spec(s),
                _ => {}
            }
        }
    }

    fn collect_script(&mut self, d: &Script) {
        self.collect_function(&d.function);
        for s in d.specs.iter() {
            self.collect_spec(s);
        }
    }

    fn collect_definition(&mut self, d: &Definition) {
        match d {
            Definition::Module(x) => self.collect_module(x),
            Definition::Address(x) => {
                for x in x.modules.iter() {
                    self.collect_module(x);
                }
            }
            Definition::Script(x) => self.collect_script(x),
        }
    }
}

impl Preprocessor for FunHandler {
    fn preprocess(&mut self, module_defs: &Arc<Vec<Definition>>) {
        for d in module_defs.iter() {
            self.collect_definition(d);
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn is_fun_specifiers(specifier: &str) -> bool {
    matches!(
        specifier,
        "acquires" | "reads" | "writes" | "pure" | "!acquires" | "!reads" | "!writes"
    )
}

impl FunHandler {
    pub(crate) fn is_generic_ty_in_fun_header(&self, kind: &NestKind) -> bool {
        let loc_vec = &self.loc_vec;
        let body_loc_vec = &self.body_loc_vec;

        let len = loc_vec.len();
        let mut left = 0;
        let mut right = len;

        while left < right {
            if kind.end_pos < loc_vec[left].start() || kind.start_pos > loc_vec[right - 1].end() {
                return false;
            }

            let mid = left + (right - left) / 2;
            let mid_loc = loc_vec[mid];
            let mid_body_loc = body_loc_vec[mid];

            if mid_loc.start() < kind.start_pos && kind.end_pos < mid_body_loc.start() {
                return true;
            } else if mid_loc.start() < kind.start_pos {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        false
    }

    pub(crate) fn is_parameter_paren_in_fun_header(&self, kind: &NestKind) -> (bool, usize) {
        if kind.kind != NestKind_::ParentTheses {
            return (false, 0);
        }
        for (i, para_span) in self.para_span_vec.iter().enumerate() {
            if kind.start_pos <= para_span.start() && para_span.end() <= kind.end_pos {
                let header_str = if self.body_loc_vec[i].end() - self.body_loc_vec[i].start() == 0 {
                    // no function body
                    &self.source[self.loc_vec[i].start() as usize..self.loc_vec[i].end() as usize]
                } else {
                    &self.source
                        [self.loc_vec[i].start() as usize..self.body_loc_vec[i].start() as usize]
                };

                let fun_header_len = header_str
                    .bytes()
                    .filter(|b| !b.is_ascii_whitespace())
                    .count();
                return (true, fun_header_len);
            }
        }
        (false, 0)
    }

    pub(crate) fn is_fun_return_colon(&self, token_tree: &TokenTree) -> usize {
        for ret_loc in &self.ret_ty_loc_vec {
            if (ret_loc.start() as u32).abs_diff(token_tree.start_pos()) <= 2 {
                return (ret_loc.end() - ret_loc.start()) as usize;
            }
        }
        0
    }
}

fn get_defs(fmt_buffer: String) -> Vec<Definition> {
    let filehash = FileHash::empty();
    parse_file_string(&mut get_compile_env(), filehash, &fmt_buffer)
        .unwrap()
        .0
}

/// Format function-specifier string (e.g. `acquires Foo, Bar reads Baz`).
/// Keywords are placed on new lines with proper indent; comments are preserved.
/// This is the optimized version.
pub(crate) fn fun_header_specifier_fmt(specifier: &str, indent_str: &str) -> String {
    tracing::trace!("fun_specifier_str = {}", specifier);

    // Early return for empty or whitespace-only input
    if specifier.trim().is_empty() {
        return specifier.to_string();
    }

    // Collect lexer tokens and count specifiers in one pass
    let (token_positions, specifier_count) = collect_tokens_and_count_specifiers(specifier);

    // Fast path: zero or one keyword → return input untouched.
    // See: https://github.com/movebit/movefmt/issues/3
    if specifier_count <= 1 {
        return specifier.to_string();
    }

    // Process tokens and format specifiers
    format_specifiers(specifier, &token_positions, indent_str)
}

#[derive(Debug, Clone, Copy)]
struct SpecTok<'a> {
    start: u32,
    end: u32,
    text: &'a str,
    is_spec: bool,
}

/// Collect token positions and count specifiers in a single pass
fn collect_tokens_and_count_specifiers<'a>(specifier: &'a str) -> (Vec<SpecTok<'a>>, usize) {
    let mut tokens = Vec::new();
    let mut specifier_count = 0;

    let mut lexer = Lexer::new(specifier, FileHash::empty());
    if lexer.advance().is_err() {
        return (tokens, specifier_count);
    }

    while lexer.peek() != Tok::EOF {
        let start = lexer.start_loc() as u32;
        let text = lexer.content();
        let end;
        let combined_text;

        // Handle '!' followed by a specifier keyword
        if text == "!" && lexer.advance().is_ok() && is_fun_specifiers(lexer.content()) {
            // Combine '!' with the following specifier (e.g., !reads)
            end = lexer.start_loc() as u32 + lexer.content().len() as u32;
            combined_text = &specifier[start as usize..end as usize];
        } else {
            end = start + text.len() as u32;
            combined_text = text;
        }

        let is_spec = is_fun_specifiers(combined_text);
        if is_spec {
            specifier_count += 1;
        }

        tokens.push(SpecTok {
            start,
            end,
            text: combined_text,
            is_spec,
        });

        if lexer.advance().is_err() {
            break;
        }
    }

    (tokens, specifier_count)
}

/// Optimized specifier formatting with better memory management
fn format_specifiers(specifier: &str, token_positions: &[SpecTok], indent_str: &str) -> String {
    // Pre-allocate result string with estimated capacity
    let estimated_size = specifier.len() + (token_positions.len() * (indent_str.len() + 10));
    let mut result = String::with_capacity(estimated_size);

    let mut found_specifier = false;
    let mut first_specifier_idx = 0;

    for (i, tok) in token_positions.iter().enumerate() {
        if tok.is_spec {
            if !found_specifier {
                first_specifier_idx = tok.start as usize;
                found_specifier = true;
            }

            // Format specifier
            result.push('\n');
            result.push_str(indent_str);
            result.push_str(tok.text);

            // Collect arguments for non-pure specifiers
            if tok.text != "pure" {
                let args = collect_args_from_tokens(specifier, token_positions, i);

                if !args.is_empty() {
                    result.push(' ');
                    result.push_str(&args);
                }
            }
        }
    }

    // Assemble final result
    if found_specifier {
        let mut final_result = String::with_capacity(first_specifier_idx + result.len() + 1);
        final_result.push_str(&specifier[..first_specifier_idx]);
        final_result.push_str(&result);
        final_result.push(' ');
        final_result
    } else {
        specifier.to_string()
    }
}

/// Collect arguments from tokens following a specifier, preserving original spacing
fn collect_args_from_tokens(
    specifier: &str,
    token_positions: &[SpecTok],
    spec_idx: usize,
) -> String {
    // Find the end of current specifier
    let spec_tok = &token_positions[spec_idx];
    let spec_end = spec_tok.end as usize;

    // Find the start of next specifier (or end of string)
    let next_spec_start = token_positions
        .iter()
        .skip(spec_idx + 1)
        .find(|tok| tok.is_spec)
        .map(|tok| tok.start as usize)
        .unwrap_or(specifier.len());

    // Extract the raw text between current and next specifier
    if spec_end < next_spec_start {
        specifier[spec_end..next_spec_start].trim().to_string()
    } else {
        String::new()
    }
}

// Return the byte start offset of each row, with an additional EOF position at the end
#[allow(dead_code)]
fn build_line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

// Return the vec with 'how many spaces before each row'
#[allow(dead_code)]
fn build_line_indent(text: &str, line_starts: &[usize]) -> Vec<usize> {
    let mut indent = Vec::with_capacity(line_starts.len().saturating_sub(1));
    for &start in &line_starts[..line_starts.len() - 1] {
        let line = &text[start..];
        let spaces = line.bytes().take_while(|&b| b == b' ').count();
        indent.push(spaces);
    }
    indent
}

// Given byte offset, return which line it falls on (0-based)
#[allow(dead_code)]
fn byte_offset_to_line(offset: usize, line_starts: &[usize]) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(l) => l,
        Err(l) => l.saturating_sub(1),
    }
}

// Return the [start, end) byte interval of line line_idx
// The last element is the virtual EOF position, so it will not exceed the boundary
#[allow(dead_code)]
fn line_range(line_idx: usize, line_starts: &[usize], text_len: usize) -> std::ops::Range<usize> {
    let start = line_starts[line_idx];
    let end = line_starts.get(line_idx + 1).copied().unwrap_or(text_len);
    start..end
}

#[allow(dead_code)]
fn process_block_comment_before_fun(fmt_buffer: &mut String, config: Config) {
    let mut fun_extractor = FunHandler::new(&fmt_buffer);
    fun_extractor.preprocess(&Arc::new(get_defs(fmt_buffer.clone())));
    let mut inserts: Vec<(usize, String)> = Vec::new(); // (byte_offset, text_to_insert)

    // precompute start offset per line
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(fmt_buffer.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    for (fun_idx, (fun_start_line, _)) in fun_extractor.loc_line_vec.iter().enumerate() {
        let line_idx = *fun_start_line as usize;
        let line_start = line_starts
            .get(line_idx)
            .copied()
            .unwrap_or(fmt_buffer.len());
        let line_end = line_starts
            .get(line_idx + 1)
            .copied()
            .unwrap_or(fmt_buffer.len());
        let fun_header_str = &fmt_buffer[line_start..line_end];

        let fun_col = fun_header_str
            .bytes()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(fun_header_str.len());

        let fun_start_pos = fun_extractor.loc_vec[fun_idx]
            .start()
            .try_into()
            .unwrap_or_default();
        if fun_start_pos != line_start + fun_col {
            let insert_txt = format!("\n{}", " ".repeat(config.indent_size()));
            inserts.push((fun_start_pos, insert_txt));
        }
    }

    for (off, txt) in inserts.iter().rev() {
        fmt_buffer.insert_str(*off, &txt);
    }
}

#[test]
fn test_rewrite_fun_header_1() {
    let cases = [
        "acquires *(make_up_address(x))",
        "!reads *(0x42), *(0x43)",
        ": u32 !reads *(0x42), *(0x43)",
        ": /*(bool, bool)*/ (bool, bool) ",
    ];
    for input in cases {
        fun_header_specifier_fmt(input, "    ");
    }
}

#[test]
fn test_rewrite_fun_header_2() {
    let cases = [
        ": u64 /* acquires comment1 */ acquires SomeStruct ",
        ": u64 acquires SomeStruct/* acquires comment2 */ ",
        ": u64 /* acquires comment3 */ acquires /* acquires comment4 */ SomeStruct /* acquires comment5 */",
        "acquires R reads R writes T, S reads G<u64> ",
        "fun f11() !reads *(0x42) ",
    ];
    for input in cases {
        fun_header_specifier_fmt(input, "    ");
    }
}

#[test]
fn test_rewrite_fun_header_3() {
    let input = "
        // comment1
        econia: &signer)
        acquires // acquires comment2
        IncentiveParameters 
    ";
    fun_header_specifier_fmt(input, "    ");
}

#[test]
fn test_performance_comparison() {
    use std::time::Instant;

    let test_cases = vec![
        "acquires *(make_up_address(x))",
        "!reads *(0x42), *(0x43)",
        ": u32 !reads *(0x42), *(0x43)",
        ": /*(bool, bool)*/ (bool, bool) ",
        ": u64 /* acquires comment1 */ acquires SomeStruct ",
        ": u64 acquires SomeStruct/* acquires comment2 */ ",
        ": u64 /* acquires comment3 */ acquires /* acquires comment4 */ SomeStruct /* acquires comment5 */",
        "acquires R reads R writes T, S reads G<u64> ",
        "fun f11() !reads *(0x42) ",
        "
        // comment1
        econia: &signer)
        acquires // acquires comment2
        IncentiveParameters 
        ",
    ];

    let iterations = 1000;

    // Test optimized version
    let start = Instant::now();
    for _ in 0..iterations {
        for case in &test_cases {
            fun_header_specifier_fmt(case, "    ");
        }
    }
    let optimized_duration = start.elapsed();

    println!("Optimized version: {:?}", optimized_duration);
}

#[test]
fn test_rewrite_fun_header_4() {
    let input = "
fun complex_function()
    acquires SomeVeryLongStructName,
      AnotherLongStructName
    reads SomeResource
    writes AnotherResource,
      YetAnotherResource
    ";
    let optimized_result = fun_header_specifier_fmt(input, "    ");
    println!("optimized_result = \n{}", optimized_result);
}

#[test]
fn test_exclamation_bug() {
    let input = "reads 0x42::*::*, !reads 0x43::*::*";
    let indent = "        ";
    let result = fun_header_specifier_fmt(input, indent);
    println!("Result: {:?}", result);

    let expected = "\n        reads 0x42::*::*,\n        !reads 0x43::*::* ";
    assert_eq!(result, expected);
}
