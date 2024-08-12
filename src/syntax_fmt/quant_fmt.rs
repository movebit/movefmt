// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core::token_tree::TokenTree;
use crate::tools::utils::FileLineMappingOneFile;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use move_compiler::shared::ast_debug;
use std::cell::RefCell;


#[derive(Debug, Default)]
pub struct QuantExtractor {
    pub quant_exp_vec: Vec<Exp>,
    pub split_quant_vec: RefCell<Vec<usize>>,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
}

impl QuantExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut this_let_extractor = Self {
            quant_exp_vec: vec![],
            split_quant_vec: vec![].into(),
            source: fmt_buffer.clone(),
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_let_extractor.line_mapping.update(&fmt_buffer);
        this_let_extractor
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
                self.quant_exp_vec.push(e.clone());
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

impl QuantExtractor {
    pub(crate) fn preprocess(&mut self, module_defs: Vec<Definition>) {
        for d in module_defs.iter() {
            self.collect_definition(d);
        }
    }

    pub(crate) fn get_quant_body_len(&self, token: TokenTree) -> (usize, usize) {
        for (idx, quant_exp) in self.quant_exp_vec.iter().enumerate() {
            if let Exp_::Quant(_, _, _, _, quant_body) = &quant_exp.value {
                if token.start_pos() == quant_body.loc.start() {
                    let quant_body_str = ast_debug::display(&quant_body.value);
                    return (idx, quant_body_str.len());
                }
            }
        }
        (0, 0)
    }

    pub(crate) fn record_long_quant_exp(&self, idx: usize) {
        self.split_quant_vec.borrow_mut().push(idx);
    }

    pub(crate) fn need_inc_depth_by_long_quant_exp(&self, token: TokenTree) -> bool {
        for idx in self.split_quant_vec.borrow().iter() {
            let quant_exp = &self.quant_exp_vec[*idx];
            if let Exp_::Quant(_, _, _, _, quant_body) = &quant_exp.value {
                if token.start_pos() == quant_body.loc.start() {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn need_dec_depth_by_long_quant_exp(&self, token: TokenTree) -> usize {
        let mut inc_depth_cnt = 0;
        for idx in self.split_quant_vec.borrow().iter() {
            let quant_exp = &self.quant_exp_vec[*idx];
            if let Exp_::Quant(_, _, _, _, quant_body) = &quant_exp.value {
                if token.end_pos() == quant_body.loc.end() {
                    inc_depth_cnt += 1;
                }
            }
        }
        inc_depth_cnt
    }
}

#[allow(dead_code)]
fn get_quant_exp(fmt_buffer: String) {
    use crate::tools::syntax::parse_file_string;
    use move_command_line_common::files::FileHash;
    use move_compiler::shared::CompilationEnv;
    use move_compiler::Flags;
    use std::collections::BTreeSet;
    let mut quant_extractor = QuantExtractor::new(fmt_buffer.clone());
    let mut env = CompilationEnv::new(Flags::testing(), BTreeSet::new());
    let (defs, _) = parse_file_string(&mut env, FileHash::empty(), &fmt_buffer).unwrap();
    quant_extractor.preprocess(defs);
    for quant_exp in quant_extractor.quant_exp_vec.iter() {
        let quant_exp_str =
            &quant_extractor.source[quant_exp.loc.start() as usize..quant_exp.loc.end() as usize];
        if quant_exp_str.len() < 64 {
            continue;
        }
        eprintln!("\n ******************************************************** >>");
        // eprintln!("quant_exp = \n{:?}\n", quant_exp_str);
        if let Exp_::Quant(_, bind_list, _, _, e2) = &quant_exp.value {
            let bind_list_str = ast_debug::display(&bind_list.value);
            eprintln!("bind_list_str = {:?}", bind_list_str);

            let quant_body_str = ast_debug::display(&e2.value);
            eprintln!("quant_body_str = {:?}", quant_body_str);
        }
        eprintln!(" ******************************************************** <<\n\n\n");
    }
}

#[test]
fn test_get_bin_op_exp() {
    get_quant_exp(
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

            /// # Access Control
            spec schema WithdrawOnlyFromCapAddress<Token> {
                cap: WithdrawCapability;
                ensures forall addr: address where old(exists<Balance<Token>>(addr)) && addr != cap.account_address:
                    balance<Token>(addr) == old(balance<Token>(addr));
                ensures forall k in 0..bitvector.length:
                    balance<Token>(addr) == old(balance<Token>(addr));
            }
        }
"
        .to_string());
}
