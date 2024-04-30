// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use crate::core::token_tree::{NestKind, TokenTree, get_code_buf_len, analyze_token_tree_length};
use crate::tools::syntax::parse_file_string;
use crate::tools::utils::FileLineMappingOneFile;
use commentfmt::Config;
use move_command_line_common::files::FileHash;
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use move_ir_types::location::*;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct CallExtractor {
    pub call_loc_vec: Vec<Loc>,
    pub call_paren_loc_vec: Vec<Loc>,
    pub pack_in_call_loc_vec: Vec<Loc>,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
}

impl CallExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut this_call_extractor = Self {
            call_loc_vec: vec![],
            call_paren_loc_vec: vec![],
            pack_in_call_loc_vec: vec![],
            source: fmt_buffer.clone(),
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_call_extractor.line_mapping.update(&fmt_buffer);
        let attrs: BTreeSet<String> = BTreeSet::new();
        let mut env = CompilationEnv::new(Flags::testing(), attrs);
        let filehash = FileHash::empty();
        let (defs, _) = parse_file_string(&mut env, filehash, &fmt_buffer).unwrap();

        for d in defs.iter() {
            this_call_extractor.collect_definition(d);
        }

        this_call_extractor
    }

    fn collect_seq_item(&mut self,  s: &SequenceItem) {
        match &s.value {
            SequenceItem_::Seq(e) => self.collect_expr(e),
            SequenceItem_::Bind(_, _, e) => {
                self.collect_expr(e);
            }
            _ => {}
        }
    }

    fn collect_seq(&mut self,  s: &Sequence) {
        for s in s.1.iter() {
            self.collect_seq_item(s);
        }
        if let Some(t) = s.3.as_ref() {
            self.collect_expr(t);
        }
    }

    fn collect_expr(&mut self,  e: &Exp) {
        match &e.value {
            Exp_::Call(_, _, _tys, es) => {
                self.call_loc_vec.push(e.loc);
                self.call_paren_loc_vec.push(es.loc);
                es.value.iter().for_each(|e| self.collect_expr(e));
            }
            Exp_::Pack(_, _tys, es) => {
                self.pack_in_call_loc_vec.push(e.loc);
                es.iter().for_each(|e| self.collect_expr(&e.1));
            }
            Exp_::Vector(_, _tys, es) => {
                es.value.iter().for_each(|e| self.collect_expr(e));
            }
            Exp_::IfElse(c, then_, eles_) => {
                self.collect_expr(c.as_ref());
                self.collect_expr(then_.as_ref());
                if let Some(else_) = eles_ {
                    self.collect_expr(else_.as_ref());
                }
            }
            Exp_::While(e, then_) => {
                self.collect_expr(e.as_ref());
                self.collect_expr(then_.as_ref());
            }
            Exp_::Loop(b) => {
                self.collect_expr(b.as_ref());
            }
            Exp_::Block(b) => self.collect_seq(b),

            Exp_::Lambda(_, e) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Quant(_, _, es, e1, e2) => {
                es.iter().for_each(|e| {
                    for e in e.iter() {
                        self.collect_expr(e)
                    }
                });
                if let Some(t) = e1 {
                    self.collect_expr(t.as_ref());
                }
                self.collect_expr(e2.as_ref());
            }
            Exp_::ExpList(es) => {
                es.iter().for_each(|e| self.collect_expr(e));
            }
            Exp_::Assign(l, r) => {
                self.collect_expr(l.as_ref());
                self.collect_expr(r.as_ref());
            }
            Exp_::Return(Some(t)) => {
                self.collect_expr(t.as_ref());
            }
            Exp_::Abort(e) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Dereference(e) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::UnaryExp(_, e) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::BinopExp(l, _, r) => {
                self.collect_expr(l.as_ref());
                self.collect_expr(r.as_ref());
            }
            Exp_::Borrow(_, e) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Dot(e, _) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Index(e, i) => {
                self.collect_expr(e.as_ref());
                self.collect_expr(i.as_ref());
            }
            Exp_::Cast(e, _) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Annotate(e, _) => {
                self.collect_expr(e.as_ref());
            }
            Exp_::Spec(_s) => {
                // self.collect_spec(s)
            },
            _ => {}
        }
    }

    fn collect_function(&mut self, d: &Function) {
        match &d.body.value {
            FunctionBody_::Defined(seq) => {
                self.collect_seq(seq);
            }
            FunctionBody_::Native => {}
        }
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

impl CallExtractor {
    // fn_call(comp1, comp2, nested_call_maybe_too_long(...), comp4);
    // >>
    // fn_call(comp1, comp2,
    //     nested_call_maybe_too_long(...), comp4);
    fn should_split_call_component(&self, next_t_start_pos: u32, config: Config, cur_ret_last_len: usize) -> bool {
        for call_in_call_loc in &self.call_loc_vec {
            if next_t_start_pos == call_in_call_loc.start() {
                let start_line = self.line_mapping
                    .translate(call_in_call_loc.start(), call_in_call_loc.start())
                    .unwrap()
                    .start
                    .line;
                let end_line = self.line_mapping
                    .translate(call_in_call_loc.end(), call_in_call_loc.end())
                    .unwrap()
                    .start
                    .line;
                let call_component_str = &self.source[call_in_call_loc.start() as usize..call_in_call_loc.end() as usize];
                let component_lenth = get_code_buf_len(call_component_str.to_string());
                if (cur_ret_last_len + component_lenth > config.max_width() && component_lenth > 8) || end_line - start_line > 2 {
                    tracing::debug!("should_split_call_component -- cur_ret_last_len: {}, component_lenth: {}", cur_ret_last_len, component_lenth);
                    return true;
                }
            }
        }
        false
    }

    // fn_call(comp1, comp2, pack {...}, comp4);
    // >>
    // fn_call(comp1, comp2,
    //     pack {...}, comp4);
    fn should_split_pack_component(&self, next_t_start_pos: u32, config: Config, cur_ret_last_len: usize) -> bool {
        for pack_in_call_loc in &self.pack_in_call_loc_vec {
            if next_t_start_pos == pack_in_call_loc.start() {
                let start_line = self.line_mapping
                    .translate(pack_in_call_loc.start(), pack_in_call_loc.start())
                    .unwrap()
                    .start
                    .line;
                let end_line = self.line_mapping
                    .translate(pack_in_call_loc.end(), pack_in_call_loc.end())
                    .unwrap()
                    .start
                    .line;
                let call_component_str = &self.source[pack_in_call_loc.start() as usize..pack_in_call_loc.end() as usize];
                let component_lenth = get_code_buf_len(call_component_str.to_string());
                if (cur_ret_last_len + component_lenth > config.max_width() && component_lenth > 8) || end_line - start_line > 2 {
                    tracing::debug!("should_split_pack_component -- cur_ret_last_len: {}, component_lenth: {}", cur_ret_last_len, component_lenth);
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn should_call_component_split(
        &self,
        config: Config,
        kind: &NestKind,
        elements: &[TokenTree],
        index: usize,
        cur_ret_last_len: usize) -> bool {
        let current = elements.get(index).unwrap();
        let next_t = elements.get(index + 1);
        if current.simple_str() == Some(",") && next_t.is_some() {
            let component_lenth = analyze_token_tree_length(&[next_t.unwrap().clone()], 10);
            if cur_ret_last_len + component_lenth > config.max_width() && component_lenth > 4 {
                return true;
            }
    
            let next_t_start_pos = get_tok_start_pos(next_t.unwrap());
    
            for call_loc in self.call_paren_loc_vec.iter() {
                if kind.start_pos <= call_loc.start() && call_loc.end() <= kind.end_pos {
                    if self.should_split_pack_component(next_t_start_pos, config.clone(), cur_ret_last_len) {
                        tracing::debug!("should split pack: next_t = {:?}", next_t.unwrap().simple_str());
                        return true;
                    }
                    if self.should_split_call_component(next_t_start_pos, config.clone(), cur_ret_last_len) {
                        tracing::debug!("should split call: next_t = {:?}", next_t.unwrap().simple_str());
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn paren_in_call(&self, kind: &NestKind,) -> bool {
        for call_loc in self.call_paren_loc_vec.iter() {
            if kind.start_pos <= call_loc.start() && call_loc.end() <= kind.end_pos {
                return  true;
            }
        }
        false
    }
}

fn get_tok_start_pos(t: &TokenTree) -> u32 {
    match t {
        TokenTree::SimpleToken {
            content: _,
            pos,
            tok: _,
            note: _,
        } => *pos,
        TokenTree::Nested {
            elements: _,
            kind,
            note: _,
        } => kind.start_pos,
    }
}
