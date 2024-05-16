// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tools::utils::FileLineMappingOneFile;
use commentfmt::Config;
// use move_command_line_common::files::FileHash;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
// use move_compiler::parser::lexer::{Lexer, Tok};
use move_ir_types::location::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct LetIfElseBlock {
    pub let_if_else_block_loc_vec: Vec<Loc>,
    pub then_in_let_loc_vec: Vec<Loc>,
    pub else_in_let_loc_vec: Vec<Loc>,

    pub let_if_else_block: Vec<lsp_types::Range>,
    pub if_cond_in_let: Vec<lsp_types::Range>,
    pub then_in_let: Vec<lsp_types::Range>,
    pub else_in_let: Vec<lsp_types::Range>,
}

#[derive(Debug)]
pub struct ComIfElseBlock {
    pub if_else_blk_loc_vec: Vec<Loc>,
    pub then_loc_vec: Vec<Loc>,
    pub else_loc_vec: Vec<Loc>,
    pub else_with_if_vec: Vec<bool>,
}

#[derive(Debug)]
pub struct BranchExtractor {
    pub let_if_else: LetIfElseBlock,
    pub com_if_else: ComIfElseBlock,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
    pub added_new_line_branch: RefCell<HashMap<ByteIndex, usize>>,
}

impl BranchExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let let_if_else = LetIfElseBlock {
            let_if_else_block_loc_vec: vec![],
            then_in_let_loc_vec: vec![],
            else_in_let_loc_vec: vec![],

            let_if_else_block: vec![],
            if_cond_in_let: vec![],
            then_in_let: vec![],
            else_in_let: vec![],
        };
        let com_if_else = ComIfElseBlock {
            if_else_blk_loc_vec: vec![],
            then_loc_vec: vec![],
            else_loc_vec: vec![],
            else_with_if_vec: vec![],
        };
        let mut this_branch_extractor = Self {
            let_if_else,
            com_if_else,
            source: fmt_buffer.clone(),
            line_mapping: FileLineMappingOneFile::default(),
            added_new_line_branch: HashMap::default().into(),
        };

        this_branch_extractor
            .line_mapping
            .update(&fmt_buffer.clone());
        this_branch_extractor
    }

    fn collect_seq_item(&mut self, s: &SequenceItem) {
        match &s.value {
            SequenceItem_::Seq(e) => self.collect_expr(e),
            SequenceItem_::Bind(_, _, e) => {
                self.collect_expr(e);
            }
            _ => {}
        }
    }

    fn collect_seq(&mut self, s: &Sequence) {
        for s in s.1.iter() {
            self.collect_seq_item(s);
        }
        if let Some(t) = s.3.as_ref() {
            self.collect_expr(t);
        }
    }

    fn collect_spec(&mut self, spec_block: &SpecBlock) {
        match &spec_block.value.target.value {
            SpecBlockTarget_::Code => {}
            SpecBlockTarget_::Module => {}
            SpecBlockTarget_::Member(_, _) | SpecBlockTarget_::Schema(_, _) => {}
        }
        for m in spec_block.value.members.iter() {
            match &m.value {
                SpecBlockMember_::Condition {
                    kind: _,
                    properties: _,
                    exp,
                    additional_exps: _,
                } => {
                    self.collect_expr(exp);
                }
                SpecBlockMember_::Function {
                    uninterpreted: _,
                    name: _,
                    signature: _,
                    body,
                } => match &body.value {
                    FunctionBody_::Defined(s) => self.collect_seq(s),
                    FunctionBody_::Native => {}
                },
                SpecBlockMember_::Variable {
                    is_global: _,
                    name: _,
                    type_parameters: _,
                    type_: _,
                    init: _,
                } => {}

                SpecBlockMember_::Let {
                    name: _,
                    post_state: _,
                    def,
                } => self.collect_expr(def),
                SpecBlockMember_::Update { lhs, rhs } => {
                    self.collect_expr(lhs);
                    self.collect_expr(rhs);
                }
                SpecBlockMember_::Include { properties: _, exp } => {
                    self.collect_expr(exp);
                }
                SpecBlockMember_::Apply {
                    exp,
                    patterns: _,
                    exclusion_patterns: _,
                } => {
                    self.collect_expr(exp);
                }
                SpecBlockMember_::Pragma { properties: _ } => {}
            }
        }
    }

    fn collect_expr(&mut self, e: &Exp) {
        match &e.value {
            Exp_::Call(_, _, _, es) => {
                es.value.iter().for_each(|e| self.collect_expr(e));
            }
            Exp_::Pack(_, _tys, es) => {
                es.iter().for_each(|e| self.collect_expr(&e.1));
            }
            Exp_::Vector(_, _tys, es) => {
                es.value.iter().for_each(|e| self.collect_expr(e));
            }
            Exp_::IfElse(_, then_, eles_opt) => {
                self.com_if_else.if_else_blk_loc_vec.push(e.loc);
                self.com_if_else.then_loc_vec.push(then_.loc);
                self.collect_expr(then_.as_ref());
                if let Some(el) = eles_opt {
                    self.com_if_else.else_loc_vec.push(el.loc);
                    if let Exp_::IfElse(..) = el.value {
                        self.com_if_else.else_with_if_vec.push(true);
                    } else {
                        self.com_if_else.else_with_if_vec.push(false);
                    }
                    self.collect_expr(el.as_ref());
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
            Exp_::Spec(s) => self.collect_spec(s),
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

impl BranchExtractor {
    pub fn preprocess(&mut self, module_defs: Vec<Definition>) {
        for d in module_defs.iter() {
            self.collect_definition(d);
        }
    }

    fn get_loc_range(&self, loc: Loc) -> lsp_types::Range {
        self.line_mapping.translate(loc.start(), loc.end()).unwrap()
    }

    fn need_new_line_in_then_without_brace(
        &self,
        cur_line: String,
        then_start_pos: ByteIndex,
        config: Config,
    ) -> bool {
        for then_loc in &self.com_if_else.then_loc_vec {
            if then_loc.start() == then_start_pos {
                tracing::debug!("need_new_line_in_then_without_brace -- then_loc = {}", &self.source[then_loc.start() as usize..then_loc.end() as usize]);
                let has_added = cur_line.len() as u32 + then_loc.end() - then_loc.start()
                    > config.max_width() as u32;

                let new_line_cnt = if self
                    .added_new_line_branch
                    .borrow()
                    .contains_key(&then_loc.end())
                {
                    self.added_new_line_branch.borrow_mut()[&then_loc.end()]
                } else {
                    0
                };
                self.added_new_line_branch
                    .borrow_mut()
                    .insert(then_loc.end(), new_line_cnt + has_added as usize);
                return has_added;
            }
        }
        false
    }

    fn need_new_line_after_else(
        &self,
        cur_line: String,
        else_start_pos: ByteIndex,
        config: Config,
    ) -> bool {
        for (else_loc_idx, else_loc) in self.com_if_else.else_loc_vec.iter().enumerate() {
            if else_loc.start() == else_start_pos {
                tracing::debug!("need_new_line_after_else -- else_loc = {}", &self.source[else_loc.start() as usize..else_loc.end() as usize]);
                let mut has_added = cur_line.len() as u32 + else_loc.end() - else_loc.start()
                    > config.max_width() as u32;
                if !has_added && else_loc_idx + 1 < self.com_if_else.else_loc_vec.len() {
                    has_added = self
                        .get_loc_range(self.com_if_else.else_loc_vec[else_loc_idx + 1])
                        .end
                        .line
                        == self.get_loc_range(*else_loc).end.line;
                }

                let new_line_cnt = if self
                    .added_new_line_branch
                    .borrow()
                    .contains_key(&else_loc.end())
                {
                    self.added_new_line_branch.borrow_mut()[&else_loc.end()]
                } else {
                    0
                };

                if self.com_if_else.else_with_if_vec[else_loc_idx] {
                    has_added = false;
                }

                tracing::debug!(
                    "need_new_line_after_else --> has_added[{:?}] = {:?}, new_line_cnt = {}",
                    cur_line,
                    has_added,
                    new_line_cnt
                );
                self.added_new_line_branch
                    .borrow_mut()
                    .insert(else_loc.end(), new_line_cnt + has_added as usize);
                return has_added;
            }
        }
        false
    }

    pub fn need_new_line_after_branch(
        &self,
        cur_line: String,
        branch_start_pos: ByteIndex,
        config: Config,
    ) -> bool {
        self.need_new_line_in_then_without_brace(cur_line.clone(), branch_start_pos, config.clone())
            || self.need_new_line_after_else(cur_line.clone(), branch_start_pos, config.clone())
    }

    fn added_new_line_in_then_without_brace(&self, then_end_pos: ByteIndex) -> usize {
        for then_loc in &self.com_if_else.then_loc_vec {
            if then_loc.end() == then_end_pos
                && self
                    .added_new_line_branch
                    .borrow()
                    .contains_key(&then_loc.end())
            {
                return self.added_new_line_branch.borrow_mut()[&then_loc.end()];
            }
        }
        0
    }

    fn added_new_line_after_else(&self, else_end_pos: ByteIndex) -> usize {
        for else_loc in &self.com_if_else.else_loc_vec {
            if else_loc.end() == else_end_pos
                && self
                    .added_new_line_branch
                    .borrow()
                    .contains_key(&else_loc.end())
            {
                return self.added_new_line_branch.borrow_mut()[&else_loc.end()];
            }
        }
        0
    }

    pub fn added_new_line_after_branch(&self, branch_end_pos: ByteIndex) -> usize {
        self.added_new_line_in_then_without_brace(branch_end_pos)
            + self.added_new_line_after_else(branch_end_pos)
    }

    pub fn is_nested_within_an_outer_else(&self, pos: ByteIndex) -> bool {
        for else_loc in self.com_if_else.else_loc_vec.iter() {
            if else_loc.start() < pos && pos < else_loc.end() {
                return true;
            }
        }
        false
    }
}
