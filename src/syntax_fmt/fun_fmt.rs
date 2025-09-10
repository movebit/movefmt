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

/// Collect arguments that follow a specifier keyword.
/// Returns the formatted string (may contain new-lines and indent).
fn collect_specifier_args(
    fun_specifiers: &[&str],
    start_idx: usize,
    specifier: &str,
    last_substr_len: &mut usize,
    current_specifier_idx: &mut usize,
    fun_specifiers_code: &mut Vec<(u32, u32, String)>,
    indent_str: &str,
) -> String {
    let mut args = Vec::new();

    // Nothing to do if we are already at the end.
    if start_idx + 1 >= fun_specifiers.len() {
        return String::new();
    }

    let mut old_last_substr_len = *last_substr_len;

    for (j, &item_j) in fun_specifiers.iter().enumerate().skip(start_idx + 1) {
        let mut this_token_is_comment = true;
        let iter_specifier = &specifier[*last_substr_len..];

        // Locate the token in the remaining substring.
        if let Some(idx) = iter_specifier.find(item_j) {
            // Check whether this token is **not** inside a comment.
            for token in &mut *fun_specifiers_code {
                if token.0 == (idx + *last_substr_len) as u32 {
                    this_token_is_comment = false;
                    break;
                }
            }
            old_last_substr_len = *last_substr_len;
            *last_substr_len = *last_substr_len + idx + item_j.len();
        }

        // If inside a comment, keep the token as-is.
        if this_token_is_comment {
            args.push(item_j.to_string());
            continue;
        }

        // Stop collecting when we reach the next specifier keyword.
        if is_fun_specifiers(item_j) {
            *current_specifier_idx = j;
            *last_substr_len = old_last_substr_len;
            break;
        } else {
            // Handle new-lines inside the argument list.
            let judge_new_line = &specifier[old_last_substr_len..*last_substr_len];
            if judge_new_line.contains('\n') {
                args.push("\n".to_string());
                let tmp_indent_str = " ".repeat(
                    indent_str
                        .chars()
                        .filter(|c| *c == ' ')
                        .count()
                        .saturating_sub(2),
                );
                args.push(tmp_indent_str);
            }
            args.push(item_j.to_string());
        }
    }

    args.join(" ")
}

/// Format function-specifier string (e.g. `acquires Foo, Bar reads Baz`).
/// Keywords are placed on new lines with proper indent; comments are preserved.
pub(crate) fn fun_header_specifier_fmt(specifier: &str, indent_str: &str) -> String {
    use std::collections::HashSet;

    tracing::trace!("fun_specifier_str = {}", specifier);

    // Collect all lexer tokens so we can detect which spans are inside comments.
    let mut fun_specifiers_code = vec![];
    let mut lexer = Lexer::new(specifier, FileHash::empty());
    if lexer.advance().is_ok() {
        while lexer.peek() != Tok::EOF {
            fun_specifiers_code.push((
                lexer.start_loc() as u32,
                (lexer.start_loc() + lexer.content().len()) as u32,
                lexer.content().to_string(),
            ));
            if lexer.advance().is_err() {
                break;
            }
        }
    }

    // Split specifier into individual words and record recognised keywords.
    let fun_specifiers: Vec<&str> = specifier.split_whitespace().collect();
    let mut specifier_str_set: HashSet<String> = HashSet::new();

    for &token in &fun_specifiers {
        if is_fun_specifiers(token) {
            specifier_str_set.insert(token.to_string());
        }
    }

    // Fast path: zero or one keyword → return input untouched.
    // See: https://github.com/movebit/movefmt/issues/3
    if specifier_str_set.len() <= 1 {
        return specifier.to_string();
    }

    let mut result = String::new();
    let mut found_specifier = false;
    let mut first_specifier_idx = 0;
    let mut current_specifier_idx = 0;
    let mut last_substr_len = 0;

    for i in 0..fun_specifiers.len() {
        if i < current_specifier_idx {
            continue;
        }

        let specifier_token = fun_specifiers[i];

        // Check whether the current token is inside a comment.
        let mut this_token_is_comment = true;
        let iter_specifier = &specifier[last_substr_len..];
        if let Some(idx) = iter_specifier.find(specifier_token) {
            for token_idx in 0..fun_specifiers_code.len() {
                let token = &fun_specifiers_code[token_idx];
                if token.0 == (idx + last_substr_len) as u32 {
                    this_token_is_comment = false;
                    fun_specifiers_code.remove(token_idx);
                    break;
                }
            }
            last_substr_len = last_substr_len + idx + specifier_token.len();
        }

        // Skip tokens that live inside comments.
        if this_token_is_comment {
            continue;
        }

        if is_fun_specifiers(specifier_token) {
            if !found_specifier {
                first_specifier_idx = last_substr_len - specifier_token.len();
                found_specifier = true;
            }

            // Place the keyword on a new line with indent.
            result.push('\n');
            result.push_str(indent_str);
            result.push_str(specifier_token);

            // Collect arguments that follow the keyword (except for "pure").
            if specifier_token != "pure" {
                let args = collect_specifier_args(
                    &fun_specifiers,
                    i,
                    specifier,
                    &mut last_substr_len,
                    &mut current_specifier_idx,
                    &mut fun_specifiers_code,
                    indent_str,
                );

                if !args.is_empty() {
                    result.push(' ');
                    result.push_str(&args);
                }
            }
        }

        if last_substr_len >= specifier.len() {
            break;
        }
    }

    // Re-assemble the final string.
    let mut ret_str = specifier[0..first_specifier_idx].to_string();
    if found_specifier {
        ret_str.push_str(&result);
        ret_str.push(' ');
    } else {
        ret_str = specifier.to_string();
    }
    ret_str
}

pub(crate) fn process_block_comment_before_fun_header(
    fmt_buffer: String,
    config: Config,
) -> String {
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let mut fun_extractor = FunHandler::new(fmt_buffer.clone());
    fun_extractor.preprocess(&Arc::new(get_defs(fmt_buffer.clone())));
    let mut insert_char_nums = 0;
    for (fun_idx, (fun_start_line, _)) in fun_extractor.loc_line_vec.iter().enumerate() {
        let fun_header_str =
            get_nth_line(buf.as_str(), *fun_start_line as usize).unwrap_or_default();
        let mut lexer = Lexer::new(fun_header_str, FileHash::empty());
        lexer.advance().unwrap();
        if lexer.peek() != Tok::EOF && !fun_header_str[0..lexer.start_loc()].trim_start().is_empty()
        {
            let mut insert_str = "\n".to_string();
            insert_str.push_str(" ".to_string().repeat(config.indent_size()).as_str());
            result.insert_str(
                fun_extractor.loc_vec[fun_idx].start() as usize + insert_char_nums,
                &insert_str,
            );
            insert_char_nums += insert_str.len();
        }
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
    let mut result = process_block_comment_before_fun_header(fmt_buffer, config.clone());
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
    fun_header_specifier_fmt(
        "
        // comment1
        econia: &signer)
        acquires // acquires comment2
        IncentiveParameters 
    ",
        "    ",
    );
}

#[test]
fn test_process_block_comment_before_fun_header_1() {
    process_block_comment_before_fun_header(
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
