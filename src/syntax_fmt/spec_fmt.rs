// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use super::syntax_trait::SingleSyntaxExtractor;
use crate::tools::utils::*;
use commentfmt::Config;
use commentfmt::comment::contains_comment;
use move_command_line_common::files::FileHash;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::syntax::parse_file_string;
use move_compiler::shared::Identifier;
use move_ir_types::location::*;
use std::vec;

#[derive(Debug, Default)]
pub struct SpecExtractor {
    pub spec_pragma_properties_num_vec: Vec<usize>,
    pub spec_pragma_loc_vec: Vec<Loc>,

    pub spec_fn_loc_vec: Vec<Loc>,
    pub spec_fn_name_loc_vec: Vec<Loc>,
    pub spec_fn_para_loc_vec: Vec<Loc>,
    pub spec_fn_ret_ty_loc_vec: Vec<Loc>,
    pub spec_fn_body_loc_vec: Vec<Loc>,
    pub spec_fn_loc_line_vec: Vec<(u32, u32)>,

    pub blk_loc_vec: Vec<Loc>,
    pub line_mapping: FileLineMappingOneFile,
}

impl SingleSyntaxExtractor for SpecExtractor {
    fn new(fmt_buffer: &str) -> Self {
        let mut spec_extractor = Self {
            spec_pragma_properties_num_vec: vec![],
            spec_pragma_loc_vec: vec![],

            spec_fn_loc_vec: vec![],
            spec_fn_name_loc_vec: vec![],
            spec_fn_para_loc_vec: vec![],
            spec_fn_ret_ty_loc_vec: vec![],
            spec_fn_body_loc_vec: vec![],
            spec_fn_loc_line_vec: vec![],

            blk_loc_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
        };

        spec_extractor.line_mapping.update(&fmt_buffer);
        let parse_result = parse_file_string(&mut get_compile_env(), FileHash::empty(), fmt_buffer);
        let Ok((defs, _)) = parse_result else {
            return spec_extractor;
        };
        for d in defs.iter() {
            spec_extractor.collect_definition(d);
        }
        spec_extractor
    }

    fn collect_seq_item(&mut self, _s: &SequenceItem) {}

    fn collect_seq(&mut self, _s: &Sequence) {}

    fn collect_expr(&mut self, _e: &Exp) {}

    fn collect_const(&mut self, _c: &Constant) {}

    fn collect_struct(&mut self, _s: &StructDefinition) {}

    fn collect_function(&mut self, _d: &Function) {}

    fn collect_spec(&mut self, spec_block: &SpecBlock) {
        self.blk_loc_vec.push(spec_block.loc);

        if let SpecBlockTarget_::Member(member_name, Some(signature)) =
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
            self.spec_fn_loc_vec.push(spec_block.value.target.loc);
            self.spec_fn_name_loc_vec.push(member_name.loc);
            self.spec_fn_para_loc_vec
                .push(if !signature.parameters.is_empty() {
                    signature.parameters[0].0.loc()
                } else {
                    signature.return_type.loc
                });
            self.spec_fn_ret_ty_loc_vec.push(signature.return_type.loc);
            // self.spec_fn_body_loc_vec.push(body.loc);
            self.spec_fn_loc_line_vec.push((start_line, end_line));
        }

        for m in spec_block.value.members.iter() {
            if let SpecBlockMember_::Function {
                uninterpreted: _,
                name,
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
                    self.spec_fn_loc_vec.push(m.loc);
                    self.spec_fn_name_loc_vec.push(name.0.loc);
                    self.spec_fn_para_loc_vec
                        .push(if !signature.parameters.is_empty() {
                            signature.parameters[0].0.loc()
                        } else {
                            signature.return_type.loc
                        });
                    self.spec_fn_ret_ty_loc_vec.push(signature.return_type.loc);
                    self.spec_fn_body_loc_vec.push(body.loc);
                    self.spec_fn_loc_line_vec.push((start_line, end_line));
                }
            }

            if let SpecBlockMember_::Pragma { properties } = &m.value {
                self.spec_pragma_properties_num_vec.push(properties.len());
                self.spec_pragma_loc_vec.push(m.loc);
            }
        }
    }

    fn collect_module(&mut self, d: &ModuleDefinition) {
        for m in d.members.iter() {
            if let ModuleMember::Spec(s) = &m {
                self.collect_spec(s)
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

fn get_nth_line(s: &str, n: usize) -> Option<&str> {
    s.lines().nth(n)
}

/// A simple TextEdit representation (all offsets are relative to the original buffer)
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Collect edits to break long spec function header lines.
fn collect_long_header_edits(
    spec_extractor: &SpecExtractor,
    raw_buffer: &str,
    config: &Config,
) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    let mut fun_idx = 0;
    for fun_loc in spec_extractor.spec_fn_loc_vec.iter() {
        let ret_ty_loc = spec_extractor.spec_fn_ret_ty_loc_vec[fun_idx];
        if ret_ty_loc.start() < fun_loc.start() {
            // this fun return void
            fun_idx += 1;
            continue;
        }

        let mut fun_name_str = &raw_buffer[fun_loc.start() as usize..ret_ty_loc.start() as usize];
        if !fun_name_str
            .chars()
            .filter(|&ch| ch == '\n')
            .collect::<String>()
            .is_empty()
        {
            // already multi-line
            fun_idx += 1;
            continue;
        }

        let ret_ty_len = (ret_ty_loc.end() - ret_ty_loc.start()) as usize;
        if fun_name_str.len() + ret_ty_len < config.max_width() {
            fun_idx += 1;
            continue;
        }

        let para_start_pos_in_header_line = spec_extractor.spec_fn_para_loc_vec[fun_idx].start()
            as usize
            - fun_loc.start() as usize;
        let mut insert_loc = ret_ty_loc.end() as usize - fun_loc.start() as usize;
        let mut lexer = Lexer::new(fun_name_str, FileHash::empty());
        lexer.advance().unwrap();
        while lexer.peek() != Tok::EOF {
            if lexer.peek() == Tok::Colon {
                insert_loc = lexer.start_loc();
            }
            lexer.advance().unwrap();
        }

        // insert pos is (/*insert here*/para1...)/*or insert here*/ : return_type
        insert_loc = if insert_loc <= para_start_pos_in_header_line {
            insert_loc
        } else {
            para_start_pos_in_header_line
        };
        fun_name_str =
            &raw_buffer[fun_loc.start() as usize..(fun_loc.start() as usize) + insert_loc];
        tracing::debug!("spec_fun_name_str = {}", fun_name_str);
        // there maybe comment between fun_name and ret_ty
        if fun_name_str.len() + ret_ty_len < config.max_width() {
            fun_idx += 1;
            continue;
        }

        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(raw_buffer);
        let start_line = line_mapping
            .translate(fun_loc.start(), fun_loc.start())
            .unwrap()
            .start
            .line;
        let fun_header_str = get_nth_line(raw_buffer, start_line as usize).unwrap_or_default();
        let trimed_header_prefix = fun_header_str.trim_start();
        if !trimed_header_prefix.is_empty() {
            let mut insert_str = "\n".to_string();
            if let Some(indent) = fun_header_str.find(trimed_header_prefix) {
                insert_str.push_str(
                    " ".to_string()
                        .repeat(indent + config.indent_size())
                        .as_str(),
                );
            }

            let insert_pos = fun_loc.start() as usize + insert_loc;
            edits.push(TextEdit {
                start: insert_pos,
                end: insert_pos, // insertion
                text: insert_str,
            });
        }
        fun_idx += 1;
    }

    edits
}

/// Collect edits to reformat pragma blocks into multiple lines when they have many properties.
/// Returns Vec<TextEdit> representing full replacement of pragma region.
fn collect_pragma_edits(
    spec_extractor: &SpecExtractor,
    raw_buffer: &str,
    config: &Config,
) -> Vec<TextEdit> {
    let mut edits = Vec::new();

    for (idx, pragma_loc) in spec_extractor.spec_pragma_loc_vec.iter().enumerate() {
        if spec_extractor.spec_pragma_properties_num_vec.len() > idx
            && spec_extractor.spec_pragma_properties_num_vec[idx] > 4
            && !contains_comment(
                &raw_buffer[pragma_loc.start() as usize..pragma_loc.end() as usize],
            )
        {
            let start_line = spec_extractor
                .line_mapping
                .translate(pragma_loc.start(), pragma_loc.start())
                .unwrap()
                .start
                .line;
            let start_line_str = raw_buffer
                .lines()
                .nth(start_line as usize)
                .unwrap_or_default();
            let leading_space_cnt =
                start_line_str.len() - start_line_str.trim_start_matches(char::is_whitespace).len();
            let mut insert_str = "\n".to_string();
            insert_str.push_str(
                " ".to_string()
                    .repeat(config.indent_size() + leading_space_cnt)
                    .as_str(),
            );

            let mut lexer = Lexer::new(
                &raw_buffer[pragma_loc.start() as usize..pragma_loc.end() as usize],
                FileHash::empty(),
            );
            let mut last_idx = pragma_loc.start() as usize;
            let mut tmp_str_vec = vec![];
            let mut insert_loc_vec = vec![];
            lexer.advance().unwrap();
            while lexer.peek() != Tok::EOF {
                if lexer.peek() == Tok::Comma {
                    insert_loc_vec.push(pragma_loc.start() + lexer.start_loc() as u32);
                    let tmp_str = raw_buffer
                        [last_idx..pragma_loc.start() as usize + lexer.start_loc() + 1]
                        .replace('\n', "")
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .join(" ");
                    if tmp_str_vec.is_empty() {
                        tmp_str_vec.push(tmp_str.clone());
                    } else {
                        tmp_str_vec.push(tmp_str.clone().trim_start().to_string());
                    }
                    last_idx = pragma_loc.start() as usize + lexer.start_loc() + 1;
                }
                lexer.advance().unwrap();
            }

            // build pragma_str by inserting insert_str before each subsequent item
            let mut pragma_str = "".to_string();
            if tmp_str_vec.is_empty() {
                // fallback: use original trimmed region
                pragma_str +=
                    raw_buffer[pragma_loc.start() as usize..pragma_loc.end() as usize].trim_start();
            } else {
                pragma_str += tmp_str_vec[0].as_str();
                for item in tmp_str_vec.iter().skip(1) {
                    pragma_str += &insert_str;
                    pragma_str += item;
                }
                pragma_str += &insert_str;
                let tmp_str = &raw_buffer[last_idx..pragma_loc.end() as usize];
                pragma_str += tmp_str.trim_start();
            }

            tracing::trace!("pragma_str = \n{}", pragma_str);
            tracing::trace!(
                "pragma_str.len = {}, pragma_loc.len = {}",
                pragma_str.len(),
                pragma_loc.end() - pragma_loc.start()
            );

            let start = pragma_loc.start() as usize;
            let end = pragma_loc.end() as usize;
            edits.push(TextEdit {
                start,
                end,
                text: pragma_str,
            });
        }
    }

    edits
}

/// Apply a batch of edits to fmt_buffer. Edits are expected to be relative to the original buffer.
/// We sort edits by start position and apply them in reverse order so earlier indexes stay valid.
/// If overlapping edits are detected, we log and skip (conservative) — this behavior can be changed.
fn apply_edits_in_place(fmt_buffer: &mut String, mut edits: Vec<TextEdit>) {
    if edits.is_empty() {
        return;
    }

    // sort by start ascending
    edits.sort_by_key(|e| e.start);

    // simple overlap detection: if overlaps, log and skip overlapping edit (conservative).
    // You can choose to merge overlapping edits instead depending on desired behavior.
    let mut non_overlapping: Vec<TextEdit> = Vec::with_capacity(edits.len());
    let mut last_end: usize = 0;
    for e in edits.into_iter() {
        if e.start < last_end {
            // overlap detected: log and skip this edit to avoid corrupting indices.
            tracing::warn!(
                "Skipping overlapping edit (start < last_end): start={} end={} last_end={}",
                e.start,
                e.end,
                last_end
            );
            continue;
        }
        last_end = e.end;
        non_overlapping.push(e);
    }

    // apply in reverse order so that earlier positions are not invalidated
    for e in non_overlapping.iter().rev() {
        // ensure bounds are valid to avoid panics
        let buf_len = fmt_buffer.len();
        let s = std::cmp::min(e.start, buf_len);
        let en = std::cmp::min(e.end, buf_len);
        fmt_buffer.replace_range(s..en, &e.text);
    }
}

pub fn fmt_spec(fmt_buffer: &mut String, config: Config) {
    let buf_clone = fmt_buffer.clone();
    let spec_extractor = SpecExtractor::new(&buf_clone);

    let mut edits: Vec<TextEdit> = Vec::new();
    edits.extend(collect_long_header_edits(
        &spec_extractor,
        &buf_clone,
        &config,
    ));
    edits.extend(collect_pragma_edits(&spec_extractor, &buf_clone, &config));

    apply_edits_in_place(fmt_buffer, edits);
}

#[test]
fn test_process_spec_fn_header_too_long_1() {
    let mut input = "
    /// test_point: fun name too long
    spec aptos_std::big_vector {
        // -----------------
        // Data invariants
        // -----------------
        
        spec singletonlllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll<T: store>(element: T, bucket_size: u64): BigVector<T>{
            ensures length(result) == 1;
            ensures result.bucket_size == bucket_size;
        }
    }   
    "
    .to_string();

    fmt_spec(&mut input, Config::default());

    tracing::trace!("result = {}", input);
}

#[test]
fn test_process_pragma_1() {
    let mut input = "
    /// Specifications of the `table_with_length` module.
    spec aptos_std::table_with_length {
    
        // Make most of the public API intrinsic. Those functions have custom specifications in the prover.
    
        spec TableWithLength {
            pragma intrinsic = map,
                map_new = new,
                map_destroy_empty = destroy_empty,                map_len = length,
                map_is_empty = empty,
                map_has_key = contains,                map_add_no_override = add,
                map_add_override_if_exists = upsert,                map_del_must_exist = remove,
                map_borrow = borrow,                map_borrow_mut = borrow_mut,
                map_borrow_mut_with_default = borrow_mut_with_default,                map_spec_get = spec_get,
                map_spec_set = spec_set,                map_spec_del = spec_remove,                map_spec_len = spec_len,                map_spec_has_key = spec_contains;
        }

        spec TableWithLength {
            pragma intrinsic = map,
                map_new = new,
                map_destroy_empty = destroy_empty,                map_len = length,
                map_is_empty = empty,
                map_has_key = contains,                map_add_no_override = add,
                map_add_override_if_exists = upsert,                map_del_must_exist = remove,
                map_borrow = borrow,                map_borrow_mut = borrow_mut,
                map_borrow_mut_with_default = borrow_mut_with_default,                map_spec_get = spec_get,
                map_spec_set = spec_set,                map_spec_del = spec_remove,                map_spec_len = spec_len,                map_spec_has_key = spec_contains;
        }

        spec TableWithLength {
            pragma intrinsic = map,
                map_new = new,
                map_destroy_empty = destroy_empty,                map_len = length,
                map_is_empty = empty,
                map_has_key = contains,                map_add_no_override = add,
                map_add_override_if_exists = upsert,                map_del_must_exist = remove,
                map_borrow = borrow,                map_borrow_mut = borrow_mut,
                map_borrow_mut_with_default = borrow_mut_with_default,                map_spec_get = spec_get,
                map_spec_set = spec_set,                map_spec_del = spec_remove,                map_spec_len = spec_len,                map_spec_has_key = spec_contains;
        }

        // cddfsdfasadfsdfs
    }
    "
    .to_string();

    fmt_spec(&mut input, Config::default());

    tracing::trace!("result = {}", input);
}

#[test]
fn test_process_pragma_2() {
    let mut input = "
    /// Specifications of the `table_with_length` module.
    spec aptos_std::table_with_length {
    
        // Make most of the public API intrinsic. Those functions have custom specifications in the prover.
    
        spec TableWithLength {
            pragma intrinsic = map, // cmt1
                map_new = new,   // cmt2
                map_destroy_empty = destroy_empty,// cmt3
                map_len = length,          // cmt4
                map_is_empty = empty,
                map_has_key = contains,/*cmt5*/
                map_add_no_override = add/*cmt6*/,
                /*cmt7*/map_add_override_if_exists = upsert,
                map_del_must_exist/*cmt7*/ = remove,
                map_borrow = borrow,
                map_borrow_mut = borrow_mut,
                map_borrow_mut_with_default = borrow_mut_with_default,
                map_spec_get = spec_get,
                map_spec_set = spec_set,
                map_spec_del = spec_remove,
                map_spec_len = spec_len,
                map_spec_has_key = spec_contains;
        }

        // commentxxx
        spec TableWithLength {
            pragma intrinsic = map,
                map_new = new,
                map_destroy_empty = destroy_empty,
                map_len = length,
                map_is_empty = empty,
                map_has_key = contains,
                map_add_no_override = add,
                map_add_override_if_exists = upsert,
                map_del_must_exist = remove,
                map_borrow = borrow,
                map_borrow_mut = borrow_mut,
                map_borrow_mut_with_default = borrow_mut_with_default,
                map_spec_get = spec_get,
                map_spec_set = spec_set,
                map_spec_del = spec_remove,
                map_spec_len = spec_len,
                map_spec_has_key = spec_contains;
        }
    }
    "
    .to_string();

    fmt_spec(&mut input, Config::default());

    tracing::trace!("result = {}", input);
}
