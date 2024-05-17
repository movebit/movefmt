// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core::token_tree::TokenTree;
use crate::tools::utils::FileLineMappingOneFile;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use move_ir_types::location::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LetExtractor {
    pub bin_op_exp_vec: Vec<Exp>,
    pub long_bin_op_exp_vec: Vec<Exp>,
    pub let_assign_loc_vec: Vec<Loc>,
    pub let_assign_rhs_exp: Vec<Exp>,
    pub split_bin_op_vec: RefCell<Vec<bool>>,
    pub break_line_by_let_rhs: RefCell<HashMap<ByteIndex, ByteIndex>>,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
}

impl LetExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut this_let_extractor = Self {
            bin_op_exp_vec: vec![],
            long_bin_op_exp_vec: vec![],
            let_assign_loc_vec: vec![],
            let_assign_rhs_exp: vec![],
            split_bin_op_vec: vec![].into(),
            break_line_by_let_rhs: HashMap::default().into(),
            source: fmt_buffer.clone(),
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_let_extractor.line_mapping.update(&fmt_buffer);
        this_let_extractor
    }

    fn collect_seq_item(&mut self, s: &SequenceItem) {
        match &s.value {
            SequenceItem_::Seq(e) => self.collect_expr(e),
            SequenceItem_::Bind(b, _, e) => {
                if b.loc.end() < e.loc.start() {
                    self.let_assign_loc_vec.push(Loc::new(
                        b.loc.file_hash(),
                        b.loc.end(),
                        e.loc.start(),
                    ));
                    self.let_assign_rhs_exp.push(*e.clone());
                }
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
                self.bin_op_exp_vec.push(e.clone());
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
                self.bin_op_exp_vec.push(e.clone());
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

impl LetExtractor {
    pub(crate) fn preprocess(&mut self, module_defs: Vec<Definition>) {
        for d in module_defs.iter() {
            self.collect_definition(d);
        }

        for bin_op_exp in self.bin_op_exp_vec.iter() {
            let bin_op_exp_str =
                &self.source[bin_op_exp.loc.start() as usize..bin_op_exp.loc.end() as usize];

            let bin_op_right_is_long = match &bin_op_exp.value {
                Exp_::BinopExp(_, op, r) => {
                    let bin_op_right_str =
                        &self.source[r.loc.start() as usize..r.loc.end() as usize].len();
                    match op.value {
                        BinOp_::Implies | BinOp_::Iff | BinOp_::Eq => *bin_op_right_str > 16,
                        _ => *bin_op_right_str > 64,
                    }
                }
                Exp_::Assign(_, r) => {
                    self.source[r.loc.start() as usize..r.loc.end() as usize].len() > 64
                }
                _ => false,
            };

            if bin_op_exp_str.len() > 64 && bin_op_right_is_long {
                self.long_bin_op_exp_vec.push(bin_op_exp.clone());
                self.split_bin_op_vec.borrow_mut().push(false);
            }
        }
    }

    pub(crate) fn is_long_bin_op(&self, token: TokenTree) -> bool {
        for bin_op_exp in self.long_bin_op_exp_vec.iter() {
            if let Exp_::BinopExp(_, m, _) = &bin_op_exp.value {
                if token.end_pos() == m.loc.end() {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn is_long_assign(&self, token: TokenTree) -> bool {
        for (idx, let_assign) in self.let_assign_loc_vec.iter().enumerate() {
            tracing::trace!(
                "let_assign: {:?}",
                &self.source[let_assign.start() as usize..let_assign.end() as usize]
            );
            if let_assign.start() <= token.end_pos() && token.end_pos() <= let_assign.end() {
                let rhs_exp_loc = &self.let_assign_rhs_exp[idx].loc;
                let is_long_rhs =
                    self.source[rhs_exp_loc.start() as usize..rhs_exp_loc.end() as usize].len()
                        > 64;
                if is_long_rhs {
                    self.break_line_by_let_rhs
                        .borrow_mut()
                        .insert(rhs_exp_loc.end(), token.end_pos());
                }
                return is_long_rhs;
            }
        }
        false
    }

    pub(crate) fn need_split_long_bin_op_exp(&self, token: TokenTree) -> bool {
        for (idx, bin_op_exp) in self.long_bin_op_exp_vec.iter().enumerate() {
            if let Exp_::BinopExp(_, m, _) = &bin_op_exp.value {
                if token.end_pos() == m.loc.end() {
                    self.split_bin_op_vec.borrow_mut()[idx] = true;
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn is_long_bin_op_exp_end(&self, token: TokenTree) -> bool {
        for (idx, bin_op_exp) in self.long_bin_op_exp_vec.iter().enumerate() {
            if let Exp_::BinopExp(_, _, r) = &bin_op_exp.value {
                if token.end_pos() == r.loc.end() && self.split_bin_op_vec.borrow()[idx] {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn need_split_long_let_assign_rhs(&self, token: TokenTree) -> bool {
        for long_let_rhs_pos in self.break_line_by_let_rhs.borrow().iter() {
            if token.end_pos() == *long_let_rhs_pos.1 {
                return true;
            }
        }
        false
    }

    pub(crate) fn is_long_let_assign_rhs_end(&self, token: TokenTree) -> bool {
        for long_let_rhs_pos in self.break_line_by_let_rhs.borrow().iter() {
            if token.end_pos() == *long_let_rhs_pos.0 {
                return true;
            }
        }
        false
    }

    pub(crate) fn need_inc_depth_by_long_op(&self, token: TokenTree) -> bool {
        self.need_split_long_bin_op_exp(token.clone()) || self.need_split_long_let_assign_rhs(token)
    }

    pub(crate) fn need_dec_depth_by_long_op(&self, token: TokenTree) -> bool {
        self.is_long_bin_op_exp_end(token.clone()) || self.is_long_let_assign_rhs_end(token)
    }
}

#[allow(dead_code)]
fn get_bin_op_exp(fmt_buffer: String) {
    let let_extractor = LetExtractor::new(fmt_buffer.clone());
    for bin_op_exp in let_extractor.bin_op_exp_vec.iter() {
        eprintln!("\n ******************************************************** >>");
        let bin_op_exp_str =
            &let_extractor.source[bin_op_exp.loc.start() as usize..bin_op_exp.loc.end() as usize];
        if bin_op_exp_str.len() < 64 {
            continue;
        }
        eprintln!("bin_op_exp = \n{:?}\n", bin_op_exp_str);

        if let Exp_::BinopExp(l, m, r) = &bin_op_exp.value {
            eprintln!(
                "bin_op_exp LLL = {:?}",
                &let_extractor.source[l.loc.start() as usize..l.loc.end() as usize]
            );
            eprintln!(
                "bin_op_exp MMM= {:?}",
                &let_extractor.source[m.loc.start() as usize..m.loc.end() as usize]
            );
            eprintln!(
                "bin_op_exp RRR = {:?}",
                &let_extractor.source[r.loc.start() as usize..r.loc.end() as usize]
            );
        }

        eprintln!(" ******************************************************** <<\n\n\n");
    }

    for let_assign in let_extractor.let_assign_loc_vec.iter() {
        eprintln!(
            "let_assign: {:?}",
            &let_extractor.source[let_assign.start() as usize..let_assign.end() as usize]
        );
    }
}

#[test]
fn test_get_bin_op_exp() {
    get_bin_op_exp(
        "
        module std::bit_vector {
            spec shift_left_for_verification_only {
                aborts_if false;
                ensures amount >= bitvector.length ==> (forall k in 0..bitvector.length: !bitvector
                    .bit_field[k]);
                ensures amount < bitvector.length ==> (forall i in bitvector.length - amount..bitvector
                    .length: !bitvector.bit_field[i]);
                ensures amount < bitvector.length ==> (forall i in 0..bitvector.length - amount: bitvector
                    .bit_field[i] == old(bitvector).bit_field[i + amount]);
            }
            
            fun test() {
                let input_deposited = 1;
                let output_deposited = 2;
        
                let input_into_output = 100;
                let max_output =
                    if (input_into_output < output_deposited) 0
                    else (input_into_output - output_deposited);
            }
        }
"
        .to_string());
}
