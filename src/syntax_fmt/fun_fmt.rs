// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::core::token_tree::{NestKind, NestKind_};
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

fn get_nth_line(s: &str, n: usize) -> Option<&str> {
    s.lines().nth(n)
}

fn get_space_cnt_before_line_str(s: &str) -> usize {
    let mut result = 0;
    let trimed_header_prefix = s.trim_start();
    if !trimed_header_prefix.is_empty() {
        if let Some(indent) = s.find(trimed_header_prefix) {
            result = indent;
        }
    }
    result
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

pub(crate) fn process_block_comment_before_fun(fmt_buffer: String, config: Config) -> String {
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

    let mut result = fmt_buffer.to_string();
    for (off, txt) in inserts.iter().rev() {
        result.insert_str(*off, &txt);
    }
    result
}

pub(crate) fn process_fun_header_too_long(fmt_buffer: String, config: Config) -> String {
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let mut fun_extractor = FunHandler::new(fmt_buffer.clone());
    fun_extractor.preprocess(&Arc::new(get_defs(fmt_buffer.clone())));
    let mut insert_char_nums = 0;
    let mut fun_idx = 0;
    for fun_loc in fun_extractor.loc_vec.iter() {
        let ret_ty_loc = fun_extractor.ret_ty_loc_vec[fun_idx];
        if ret_ty_loc.start() < fun_loc.start() {
            // this fun return void
            fun_idx += 1;
            continue;
        }

        let mut fun_name_str = &buf[fun_loc.start() as usize..ret_ty_loc.start() as usize];
        if !fun_name_str
            .chars()
            .filter(|&ch| ch == '\n')
            .collect::<String>()
            .is_empty()
        {
            // if it is multi line
            fun_idx += 1;
            continue;
        }
        let ret_ty_len = (ret_ty_loc.end() - ret_ty_loc.start()) as usize;
        if fun_name_str.len() + ret_ty_len < config.max_width() {
            fun_idx += 1;
            continue;
        }

        let mut insert_loc = ret_ty_loc.end() as usize - fun_loc.start() as usize;
        let mut lexer = Lexer::new(fun_name_str, FileHash::empty());
        lexer.advance().unwrap();
        while lexer.peek() != Tok::EOF {
            if lexer.peek() == Tok::Colon {
                insert_loc = lexer.start_loc() + 1;
            }
            lexer.advance().unwrap();
        }
        fun_name_str = &buf[fun_loc.start() as usize..(fun_loc.start() as usize) + insert_loc];
        tracing::debug!("fun_name_str = {}", fun_name_str);
        // there maybe comment bewteen fun_name and ret_ty
        if fun_name_str.len() + ret_ty_len < config.max_width() {
            fun_idx += 1;
            continue;
        }

        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(&fmt_buffer);
        let start_line = line_mapping
            .translate(fun_loc.start(), fun_loc.start())
            .unwrap()
            .start
            .line;
        let fun_header_str = get_nth_line(buf.as_str(), start_line as usize).unwrap_or_default();
        let trimed_header_prefix = fun_header_str.trim_start();
        if !trimed_header_prefix.is_empty() {
            let s = result[fun_loc.start() as usize + insert_char_nums + insert_loc..].to_string();
            if s.trim_start().starts_with("(\n") {
                fun_idx += 1;
                continue;
            }

            let mut insert_str = "\n".to_string();
            if let Some(indent) = fun_header_str.find(trimed_header_prefix) {
                insert_str.push_str(
                    " ".to_string()
                        .repeat(indent + config.indent_size() - 1)
                        .as_str(),
                );
            }
            result.insert_str(
                fun_loc.start() as usize + insert_char_nums + insert_loc,
                &insert_str,
            );
            insert_char_nums += insert_str.len();
        }
        fun_idx += 1;
    }
    result
}

pub(crate) fn process_fun_ret_ty(fmt_buffer: String, config: Config) -> String {
    // process this case:
    // fun fun_name()
    // : u64 {}
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let mut fun_extractor = FunHandler::new(fmt_buffer.clone());
    fun_extractor.preprocess(&Arc::new(get_defs(fmt_buffer.clone())));
    let mut insert_char_nums = 0;
    let mut fun_idx = 0;
    for fun_loc in fun_extractor.loc_vec.iter() {
        let ret_ty_loc = fun_extractor.ret_ty_loc_vec[fun_idx];
        if ret_ty_loc.start() < fun_loc.start() {
            // this fun return void
            fun_idx += 1;
            continue;
        }

        let fun_name_str = &buf[fun_loc.start() as usize..ret_ty_loc.start() as usize];
        if fun_name_str.lines().count() > 1 {
            let fun_header_str = buf
                .lines()
                .nth(fun_extractor.loc_line_vec[fun_idx].0 as usize)
                .unwrap_or_default();
            let ret_ty_str = fun_name_str.lines().last().unwrap_or_default();
            let mut lexer = Lexer::new(ret_ty_str, FileHash::empty());
            lexer.advance().unwrap();
            if lexer.peek() != Tok::Colon {
                continue;
            }

            let indent1 = get_space_cnt_before_line_str(fun_header_str);
            let indent2 = get_space_cnt_before_line_str(ret_ty_str);
            if indent1 == indent2 {
                result.insert_str(
                    ret_ty_loc.start() as usize - ret_ty_str.len() + insert_char_nums,
                    " ".to_string().repeat(config.indent_size()).as_str(),
                );
                insert_char_nums += config.indent_size();
            }
        }
    }
    result
}

pub fn fmt_fun(fmt_buffer: String, config: Config) -> String {
    let mut result = process_block_comment_before_fun(fmt_buffer, config.clone());
    result = process_fun_header_too_long(result, config.clone());
    result = process_fun_ret_ty(result, config.clone());
    result
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
        "
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
fn test_process_fun_header_too_long1() {
    let ret_str = process_fun_header_too_long(
"
module TestFunFormat {
    fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(v: u64): SomeOtherStruct {}

    // xxxx
    fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(v: u64): SomeOtherStruct {}
}
".to_string(), Config::default());

    tracing::debug!("fun_specifier_fmted_str = --------------{}", ret_str);
}

#[test]
fn test_process_fun_header_too_long2() {
    let ret_str = process_fun_header_too_long(
        "
module 0x42::LambdaTest1 {
    // Public inline function
    public inline fun inline_mul(a: u64, // Input parameter a
        b: u64) // Input parameter b
    : u64 { // Returns a u64 value
        // Multiply a and b
        a * b
    }
}
"
        .to_string(),
        Config::default(),
    );

    tracing::debug!("fun_specifier_fmted_str = --------------{}", ret_str);
}

#[test]
fn test_process_fun_ret_ty() {
    process_fun_ret_ty(
        "
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
