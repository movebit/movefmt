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
    fn new(fmt_buffer: String) -> Self {
        let mut this_fun_extractor = Self {
            loc_vec: vec![],
            para_span_vec: vec![],
            ret_ty_loc_vec: vec![],
            body_loc_vec: vec![],
            loc_line_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
            source: fmt_buffer.clone(),
        };

        this_fun_extractor.line_mapping.update(&fmt_buffer);
        this_fun_extractor
    }

    fn collect_seq_item(&mut self, _s: &SequenceItem) {}

    fn collect_seq(&mut self, _s: &Sequence) {}

    fn collect_spec(&mut self, _spec_block: &SpecBlock) {}

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
            if let ModuleMember::Function(x) = &m {
                self.collect_function(x)
            }
        }
    }

    fn collect_script(&mut self, d: &Script) {
        self.collect_function(&d.function);
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
                    .replace('\n', "")
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join("")
                    .len();
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

    // Pre-calculate indent for arguments to avoid repeated computation
    let arg_indent = calculate_arg_indent(indent_str);

    // Process tokens and format specifiers
    format_specifiers_optimized(specifier, &token_positions, indent_str, &arg_indent)
}

/// Collect token positions and count specifiers in a single pass
fn collect_tokens_and_count_specifiers(specifier: &str) -> (Vec<(u32, u32, String)>, usize) {
    let mut token_positions = Vec::new();
    let mut specifier_count = 0;

    let mut lexer = Lexer::new(specifier, FileHash::empty());
    if lexer.advance().is_ok() {
        while lexer.peek() != Tok::EOF {
            let content = lexer.content().to_string();
            token_positions.push((
                lexer.start_loc() as u32,
                (lexer.start_loc() + content.len()) as u32,
                content.clone(),
            ));

            // Count specifiers while we're at it
            if is_fun_specifiers(&content) {
                specifier_count += 1;
            }

            if lexer.advance().is_err() {
                break;
            }
        }
    }

    (token_positions, specifier_count)
}

/// Pre-calculate argument indentation to avoid repeated computation
fn calculate_arg_indent(indent_str: &str) -> String {
    let space_count = indent_str.chars().filter(|&c| c == ' ').count();
    " ".repeat(space_count.saturating_add(2))
}

/// Optimized specifier formatting with better memory management
fn format_specifiers_optimized(
    specifier: &str,
    token_positions: &[(u32, u32, String)],
    indent_str: &str,
    arg_indent: &str,
) -> String {
    let tokens: Vec<&str> = specifier.split_whitespace().collect();
    let mut token_positions = token_positions.to_vec(); // Make mutable copy

    // Pre-allocate result string with estimated capacity
    let estimated_size = specifier.len() + (tokens.len() * (indent_str.len() + 10));
    let mut result = String::with_capacity(estimated_size);

    let mut found_specifier = false;
    let mut first_specifier_idx = 0;
    let mut current_pos = 0;
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        // Find token position in original string
        if let Some(token_idx) = specifier[current_pos..].find(token) {
            let absolute_pos = current_pos + token_idx;

            // Check if token is in comment (optimized lookup)
            let is_comment = !is_token_at_position(&mut token_positions, absolute_pos as u32);
            current_pos = absolute_pos + token.len();

            if !is_comment && is_fun_specifiers(token) {
                if !found_specifier {
                    first_specifier_idx = absolute_pos;
                    found_specifier = true;
                }

                // Format specifier
                result.push('\n');
                result.push_str(indent_str);
                result.push_str(token);

                // Collect arguments for non-pure specifiers
                if token != "pure" {
                    let args = collect_args_optimized(
                        &tokens,
                        i,
                        specifier,
                        &mut current_pos,
                        &mut i, // This will be updated to skip processed tokens
                        arg_indent,
                    );

                    if !args.is_empty() {
                        result.push(' ');
                        result.push_str(&args);
                    }
                }
            }
        }

        i += 1;
        if current_pos >= specifier.len() {
            break;
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

/// Optimized token position lookup with removal
fn is_token_at_position(token_positions: &mut Vec<(u32, u32, String)>, pos: u32) -> bool {
    if let Some(index) = token_positions
        .iter()
        .position(|(start, _, _)| *start == pos)
    {
        token_positions.remove(index);
        true
    } else {
        false
    }
}

/// Optimized argument collection with better string handling
fn collect_args_optimized(
    tokens: &[&str],
    start_idx: usize,
    specifier: &str,
    current_pos: &mut usize,
    next_i: &mut usize,
    arg_indent: &str,
) -> String {
    let mut args = Vec::new();

    for (j, token) in tokens.iter().enumerate().skip(start_idx + 1) {
        // Stop at next specifier
        if is_fun_specifiers(token) {
            *next_i = j - 1; // Set to process this specifier next
            break;
        }

        // Find token in remaining string
        if let Some(token_idx) = specifier[*current_pos..].find(token) {
            let absolute_pos = *current_pos + token_idx;
            let between_text = &specifier[*current_pos..absolute_pos];

            // Handle newlines more efficiently
            if between_text.contains('\n') {
                args.push("\n"); // added one space with '\n' and arg_indent
                args.push(arg_indent); // added one space with arg_indent and token
            }

            args.push(token);
            *current_pos = absolute_pos + token.len();
        }
    }

    args.join(" ")
}

// Return the byte start offset of each row, with an additional EOF position at the end
fn build_line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

// Return the vec with 'how many spaces before each row'
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
pub fn byte_offset_to_line(offset: usize, line_starts: &[usize]) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(l) => l,
        Err(l) => l.saturating_sub(1),
    }
}

// Return the [start, end) byte interval of line line_idx
// The last element is the virtual EOF position, so it will not exceed the boundary
pub fn line_range(
    line_idx: usize,
    line_starts: &[usize],
    text_len: usize,
) -> std::ops::Range<usize> {
    let start = line_starts[line_idx];
    let end = line_starts.get(line_idx + 1).copied().unwrap_or(text_len);
    start..end
}

#[allow(dead_code)]
fn process_block_comment_before_fun(fmt_buffer: &mut String, config: Config) {
    let mut fun_extractor = FunHandler::new(fmt_buffer.clone());
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

// process_fun_ret_ty is used to process this case:
// fun fun_name()
// : u64 {}
fn process_fun_ret_ty(fmt_buffer: &mut String, config: Config) {
    let mut fh = FunHandler::new(fmt_buffer.to_string());
    fh.preprocess(&Arc::new(get_defs(fmt_buffer.clone())));
    let line_starts = build_line_starts(&fmt_buffer);
    let line_indent = build_line_indent(&fmt_buffer, &line_starts);

    let mut inserts = Vec::new(); // (byte_offset, text_to_insert)

    for (idx, fun_loc) in fh.loc_vec.iter().enumerate() {
        let ret_loc = &fh.ret_ty_loc_vec[idx];
        if ret_loc.start() < fun_loc.start() {
            continue; // this fun return void
        }

        let name_end = fun_loc.start() as usize;
        let ret_start = ret_loc.start() as usize;
        // Slice positioning: the last line of the function name and the line where the colon is located
        let name_line_idx = byte_offset_to_line(name_end, &line_starts);
        let ret_line_idx = byte_offset_to_line(ret_start, &line_starts);
        if name_line_idx == ret_line_idx {
            continue;
        }

        let ret_line_range = line_range(ret_line_idx, &line_starts, fmt_buffer.len());
        let ret_ty_str = &fmt_buffer[ret_line_range.clone()];
        let mut lexer = Lexer::new(ret_ty_str, FileHash::empty());
        lexer.advance().unwrap();
        if lexer.peek() != Tok::Colon {
            continue;
        }

        let fun_head_tail_line_range = line_range(ret_line_idx - 1, &line_starts, fmt_buffer.len());
        let fun_head_str = &fmt_buffer[fun_head_tail_line_range.clone()];
        let name_end_line_idx = name_line_idx + fun_head_str.lines().count() - 1;

        let wanted_indent = line_indent[name_end_line_idx] + config.indent_size();
        let actual_indent = line_indent[ret_line_idx];
        if actual_indent != wanted_indent {
            inserts.push((ret_line_range.start, " ".repeat(config.indent_size())));
        }
    }

    for (off, txt) in inserts.into_iter().rev() {
        fmt_buffer.insert_str(off, &txt);
    }
}

// TODO: remove fmt_fun
pub fn fmt_fun(fmt_buffer: &mut String, config: Config) -> String {
    process_fun_ret_ty(fmt_buffer, config.clone());
    fmt_buffer.to_string()
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
fn test_process_block_comment_before_fun_header_1() {
    process_block_comment_before_fun(
        &mut "
        module TestFunFormat {
        
            struct SomeOtherStruct has drop {
                some_field: u64,
            } 
            /* BlockComment1 */ public fun multi_arg(p1: u64, p2: u64): u64 {
                p1 + p2
            }
            // test two fun Close together without any blank lines, and here is a InlineComment
            /* BlockComment2 */ public fun multi_arg22(p1: u64, p2: u64): u64 {
                p1 + p2
            } 
            /* BlockComment3 */ /* BlockComment4 */ fun multi_arg22(p1: u64, p2: u64): u64 {
                p1 + p2
            }
        }
        "
        .to_string(),
        Config::default(),
    );
}

#[test]
fn test_process_fun_ret_ty() {
    process_fun_ret_ty(
        &mut "
module 0x42::LambdaTest1 {  
    /** Public inline function */  
    public inline fun inline_mul(/** Input parameter a */ a: u64,   
                                 /** Input parameter b */ b: u64)   
    /** Returns a u64 value */ : u64 {  
        /** Multiply a and b */  
        a * b  
    }  
}
"
        .to_string(),
        Config::default(),
    );
}
