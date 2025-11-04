// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tools::utils::*;
use memchr::memchr;
use move_command_line_common::files::FileHash;
use move_compiler::parser::ast::*;
use move_compiler::parser::syntax::parse_file_string;
use move_ir_types::location::*;

use super::syntax_trait::SingleSyntaxExtractor;

#[derive(Debug, Default)]
pub struct BigBlockExtractor {
    pub blk_loc_vec: Vec<Loc>,
    pub line_mapping: FileLineMappingOneFile,
}

impl SingleSyntaxExtractor for BigBlockExtractor {
    fn new(fmt_buffer: String) -> Self {
        let mut big_block_extractor = Self {
            blk_loc_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
        };

        big_block_extractor.line_mapping.update(&fmt_buffer);
        let parse_result =
            parse_file_string(&mut get_compile_env(), FileHash::empty(), &fmt_buffer);
        let Ok((defs, _)) = parse_result else {
            return big_block_extractor;
        };

        for d in defs.iter() {
            big_block_extractor.collect_definition(d);
        }
        big_block_extractor
    }

    fn collect_seq_item(&mut self, _s: &SequenceItem) {}

    fn collect_seq(&mut self, _s: &Sequence) {}

    fn collect_expr(&mut self, _e: &Exp) {}

    fn collect_const(&mut self, _c: &Constant) {}

    fn collect_struct(&mut self, s: &StructDefinition) {
        self.blk_loc_vec.push(s.loc);
    }

    fn collect_function(&mut self, d: &Function) {
        match &d.body.value {
            FunctionBody_::Defined(..) => {
                self.blk_loc_vec.push(d.loc);
            }
            FunctionBody_::Native => {}
        }
    }

    fn collect_spec(&mut self, spec_block: &SpecBlock) {
        self.blk_loc_vec.push(spec_block.loc);
    }

    fn collect_module(&mut self, d: &ModuleDefinition) {
        for m in d.members.iter() {
            match &m {
                ModuleMember::Struct(x) => self.collect_struct(x),
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

#[inline]
fn nth_line_range(s: &[u8], n: usize, lines: &[usize]) -> Option<(usize, usize)> {
    let &start = lines.get(n)?;
    let end = lines.get(n + 1).map(|&p| p - 1).unwrap_or(s.len());
    Some((start, end))
}

fn add_blank_row_in_two_blocks(s: &mut String) {
    let extractor = BigBlockExtractor::new(s.clone());
    if extractor.blk_loc_vec.len() < 2 {
        return;
    }

    let mut nls = vec![0];
    let mut start = 0;
    while let Some(p) = memchr(b'\n', &s.as_bytes()[start..]) {
        start += p + 1;
        nls.push(start);
    }

    let mut inserts = Vec::new();
    for idx in 0..extractor.blk_loc_vec.len() - 1 {
        let blk1_end_byte = extractor.blk_loc_vec[idx].end() as usize;
        let blk2_start_byte = extractor.blk_loc_vec[idx + 1].start() as usize;

        let line1 = match nls.binary_search(&blk1_end_byte) {
            Ok(l) => l,
            Err(l) => l.saturating_sub(1),
        };
        let line2 = match nls.binary_search(&blk2_start_byte) {
            Ok(l) => l,
            Err(l) => l.saturating_sub(1),
        };

        let need = match line2.checked_sub(line1 + 1) {
            None | Some(0) => true,
            Some(_) => {
                let mid_line = line1 + 1;
                if let Some((st, en)) = nth_line_range(s.as_bytes(), mid_line, &nls) {
                    let line_bytes = &s.as_bytes()[st..en];
                    en - st > 1
                        && line_bytes
                            .iter()
                            .any(|&b| b != b' ' && b != b'\t' && b != b'\n')
                } else {
                    false
                }
            }
        };

        if need {
            let ins_pos = nls.get(line1 + 1).copied().unwrap_or(s.len());
            inserts.push(ins_pos);
        }
    }

    inserts.reverse();
    for pos in inserts {
        s.insert(pos, '\n');
    }
}

pub fn fmt_big_block(fmt_buffer: &mut String) {
    add_blank_row_in_two_blocks(fmt_buffer)
}

#[test]
fn test_add_blank_row_in_two_blocks_1() {
    let mut input = "
    module std::ascii {
        struct Char {
            byte: u8,
        }
        spec Char {
            // comment
            invariant is_valid_char(byte); //comment
        }
    }    
    "
    .to_string();
    add_blank_row_in_two_blocks(&mut input);

    tracing::debug!("result = {}", input);
}

#[test]
fn test_add_blank_row_in_two_blocks_2() {
    let mut input = "
module Test {
    struct SomeOtherStruct1 has drop {
        some_other_field1: u64,
    }

    struct SomeOtherStruct2 has drop {
        some_other_field2: SomeOtherStruct1,
    }

    struct SomeOtherStruct3 has drop {
        some_other_field3: SomeOtherStruct2,
    } //comment
    struct SomeOtherStruct4 has drop {
        some_other_field4: SomeOtherStruct3,
    }

    struct SomeOtherStruct5 has drop {
        some_other_field5: SomeOtherStruct4,
    }

    struct SomeOtherStruct6 has drop {
        some_other_field6: SomeOtherStruct5,
    }
    struct SomeStruct has key, drop, store {
        some_field: SomeOtherStruct6,
    }

    fun acq(addr: address): u64
        acquires SomeStruct {
        let val = borrow_global<SomeStruct>(addr);

        val.some_field.some_other_field6.some_other_field5.some_other_field4.some_other_field3.
        some_other_field2.some_other_field1
    }
}
"
    .to_string();
    add_blank_row_in_two_blocks(&mut input);

    tracing::debug!("result = {}", input);
}

#[test]
fn test_add_blank_row_in_two_blocks_3() {
    let mut input = "
module test_module1 {

    struct TestStruct1 {
        // This is field1 comment
        field1: u64,
        field2: bool,
    }
}

module test_module2 {

    struct TestStruct2 { // This is a comment before struct definition
        field1: u64, // This is a comment for field1
        field2: bool, // This is a comment for field2
    } // This is a comment after struct definition
}

module test_module4 {

    struct TestStruct4<T>{
        // This is a comment before complex field
        field: vector<T>, // This is a comment after complex field
    }
}
"
    .to_string();
    add_blank_row_in_two_blocks(&mut input);

    tracing::debug!("result = {}", input);
}

#[test]
fn test_add_blank_row_in_two_blocks_4() {
    let mut input = "
spec std::string {
    spec internal_check_utf8(v: &vector<u8>): bool {
        pragma opaque;
        aborts_if [abstract] false;
        ensures [abstract] result == spec_internal_check_utf8(v);
    }
    spec internal_is_char_boundary(v: &vector<u8>, i: u64): bool {
        pragma opaque;
        aborts_if[abstract] false;
        ensures[abstract] result == spec_internal_is_char_boundary(v, i);
    }
}
    
"
    .to_string();
    add_blank_row_in_two_blocks(&mut input);

    tracing::debug!("result = {}", input);
}

#[test]
fn test_add_blank_row_in_two_blocks_5() {
    let mut input = "
address 0x1 {
    module M {
        #[test]
        #[expected_failure(vector_error, minor_status = 1, location = Self)]
        fun borrow_out_of_range() {}
        #[test]
        #[expected_failure(abort_code = 26113, location = extensions::table)]
        fun test_destroy_fails() {}
    }
}
    "
    .to_string();
    add_blank_row_in_two_blocks(&mut input);

    tracing::debug!("result = {}", input);
}
