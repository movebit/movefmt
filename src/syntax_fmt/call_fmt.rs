// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::core::token_tree::*;
use crate::tools::utils::FileLineMappingOneFile;
use commentfmt::Config;
use commentfmt::comment::contains_comment;
use move_compiler::parser::ast::*;
use move_compiler::parser::lexer::Tok;
use move_ir_types::location::*;

use super::syntax_trait::{Preprocessor, SingleSyntaxExtractor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexCallKind {
    None,
    Call,
    Pack,
    LambdaBrace,
}

#[derive(Clone, Debug, Default)]
pub struct CallHandler {
    pub call_loc_vec: Vec<Loc>,
    pub call_paren_loc_vec: Vec<Loc>,
    pub pack_in_call_loc_vec: Vec<Loc>,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
}

impl SingleSyntaxExtractor for CallHandler {
    fn new(fmt_buffer: &str) -> Self {
        let mut this_call_extractor = Self {
            call_loc_vec: vec![],
            call_paren_loc_vec: vec![],
            pack_in_call_loc_vec: vec![],
            source: fmt_buffer.to_string(),
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_call_extractor.line_mapping.update(&fmt_buffer);
        this_call_extractor
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
            // Zax 20241217 issue45
            Exp_::While(_, e, then_) => {
                self.collect_expr(e.as_ref());
                self.collect_expr(then_.as_ref());
            }
            // Zax 20241217 issue45
            Exp_::Loop(_, b) => {
                self.collect_expr(b.as_ref());
            }
            Exp_::Block(b) => self.collect_seq(b),
            // Zax 20241217 issue45
            Exp_::Lambda(_, e, _, _) => {
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
            // Zax 20241217 issue45
            Exp_::Assign(l, _bin_op, r) => {
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
            }
            _ => {}
        }
    }

    fn collect_const(&mut self, c: &Constant) {
        self.collect_expr(&c.value);
    }

    fn collect_struct(&mut self, _s: &StructDefinition) {}

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
            if let ModuleMember::Constant(con) = &m {
                self.collect_const(con);
            }
        }
    }

    fn collect_script(&mut self, d: &Script) {
        for const_data in &d.constants {
            self.collect_const(const_data);
        }
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

impl Preprocessor for CallHandler {
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

impl CallHandler {
    // fn_call(comp1, comp2, nested_call_maybe_too_long(...), comp4);
    // >>
    // fn_call(comp1, comp2,
    //     nested_call_maybe_too_long(...), comp4);
    fn should_split_call_component(
        &self,
        next_t_start_pos: u32,
        config: Config,
        cur_ret_last_len: usize,
    ) -> bool {
        for call_in_call_loc in &self.call_loc_vec {
            // updated in 20240605: maybe has '&' before call name,
            // like this: fun_call(para1, &call_name2(), !call_name3());
            if next_t_start_pos == call_in_call_loc.start()
                || next_t_start_pos + 1 == call_in_call_loc.start()
            {
                let start_line = self
                    .line_mapping
                    .translate(call_in_call_loc.start(), call_in_call_loc.start())
                    .unwrap()
                    .start
                    .line;
                let end_line = self
                    .line_mapping
                    .translate(call_in_call_loc.end(), call_in_call_loc.end())
                    .unwrap()
                    .start
                    .line;
                let call_component_str = &self.source
                    [call_in_call_loc.start() as usize..call_in_call_loc.end() as usize];
                let component_lenth = get_code_buf_len(call_component_str.to_string()).0;
                if (cur_ret_last_len + component_lenth > config.max_width() && component_lenth > 8)
                    || end_line - start_line > 2
                {
                    tracing::debug!(
                        "should_split_call_component -- cur_ret_last_len: {}, component_lenth: {}",
                        cur_ret_last_len,
                        component_lenth
                    );
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
    fn should_split_pack_component(
        &self,
        next_t_start_pos: u32,
        config: Config,
        cur_ret_last_len: usize,
    ) -> bool {
        for pack_in_call_loc in &self.pack_in_call_loc_vec {
            if next_t_start_pos == pack_in_call_loc.start() {
                let start_line = self
                    .line_mapping
                    .translate(pack_in_call_loc.start(), pack_in_call_loc.start())
                    .unwrap()
                    .start
                    .line;
                let end_line = self
                    .line_mapping
                    .translate(pack_in_call_loc.end(), pack_in_call_loc.end())
                    .unwrap()
                    .start
                    .line;
                let call_component_str = &self.source
                    [pack_in_call_loc.start() as usize..pack_in_call_loc.end() as usize];
                let component_lenth = get_code_buf_len(call_component_str.to_string()).0;
                if cur_ret_last_len + component_lenth > config.max_width()
                    || end_line - start_line > 2
                {
                    tracing::debug!(
                        "should_split_pack_component -- cur_ret_last_len: {}, component_lenth: {}",
                        cur_ret_last_len,
                        component_lenth
                    );
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
        cur_ret_last_len: usize,
    ) -> bool {
        let current = elements.get(index).unwrap();
        let next_t = elements.get(index + 1);
        if current.get_end_tok() != Tok::Comma || next_t.is_none() {
            return false;
        }
        let component_lenth = analyze_token_tree_length(&[next_t.unwrap().clone()], 10);
        if cur_ret_last_len + component_lenth > config.max_width() && component_lenth > 4 {
            return true;
        }
        self.component_is_complex_blk(config, kind, elements, index as i64, cur_ret_last_len)
            != ComplexCallKind::None
    }

    pub(crate) fn paren_in_call(&self, kind: &NestKind) -> bool {
        for call_loc in self.call_paren_loc_vec.iter() {
            if kind.end_pos + 1 == call_loc.end() {
                return true;
            }
        }
        false
    }

    pub(crate) fn component_is_complex_blk(
        &self,
        config: Config,
        kind: &NestKind,
        elements: &[TokenTree],
        index: i64,
        cur_ret_last_len: usize,
    ) -> ComplexCallKind {
        let next_t = elements.get((index + 1) as usize);
        let next_next_t = elements.get((index + 2) as usize);
        if next_t.is_none() {
            return ComplexCallKind::None;
        }
        let next_t_start_pos = next_t.unwrap().start_pos();
        for call_loc in self.call_paren_loc_vec.iter() {
            if kind.start_pos > call_loc.start() || call_loc.end() > kind.end_pos {
                continue;
            }

            if self.should_split_call_component(next_t_start_pos, config.clone(), cur_ret_last_len)
            {
                tracing::debug!(
                    "should split call: next_t = {:?}",
                    next_t.unwrap().simple_str()
                );
                return ComplexCallKind::Call;
            }

            if self.should_split_pack_component(next_t_start_pos, config.clone(), cur_ret_last_len)
            {
                tracing::debug!(
                    "should split pack: next_t = {:?}",
                    next_t.unwrap().simple_str()
                );
                return ComplexCallKind::Pack;
            }
        }

        // added in 20240605: judge if the next parameter is a lambda exp block
        // eg:
        // V::enumerate_ref(&filtered_v, |i, x| { ... });
        if let TokenTree::Nested {
            elements: _lambda_ele,
            kind: next_kind,
            ..
        } = next_t.unwrap()
        {
            if next_kind.kind == NestKind_::Lambda && next_next_t.is_some() {
                if let TokenTree::Nested {
                    elements: lambda_brace_ele,
                    kind: next_next_kind,
                    ..
                } = next_next_t.unwrap()
                {
                    if next_next_kind.kind == NestKind_::Brace {
                        tracing::debug!("next_next_kind.kind == NestKind_::Brace");
                        let mut new_line_mode = cur_ret_last_len
                            + (next_t.unwrap().token_len() + next_next_t.unwrap().token_len())
                                as usize
                            > config.max_width();

                        let (delimiter, _) = analyze_token_tree_delimiter(lambda_brace_ele);
                        new_line_mode |= delimiter
                            .map(|x| x == Delimiter::Semicolon)
                            .unwrap_or_default();
                        if new_line_mode {
                            return ComplexCallKind::LambdaBrace;
                        }
                    }
                }
            }
        }

        ComplexCallKind::None
    }
}

pub(crate) fn parse_nested_token_nums(
    elements: &[TokenTree],
    simple_token_cnt: &mut i32,
    comma_cnt: &mut i32,
    bin_op_cnt: &mut i32,
    nested_token_cnt: &mut i32,
) {
    for ele in elements {
        match ele {
            TokenTree::SimpleToken {
                content: _,
                pos: _,
                tok,
                note,
            } => {
                *simple_token_cnt += 1;
                if *tok == Tok::Comma {
                    *comma_cnt += 1;
                }
                if note.map(|x| x == Note::BinaryOP).unwrap_or_default() {
                    *bin_op_cnt += 1;
                }
            }
            TokenTree::Nested { elements, .. } => {
                parse_nested_token_nums(
                    elements,
                    simple_token_cnt,
                    comma_cnt,
                    bin_op_cnt,
                    nested_token_cnt,
                );
                *nested_token_cnt += 1;
            }
        }
    }
}

impl CallHandler {
    pub(crate) fn need_split_call_component(
        &self,
        config: Config,
        kind: &NestKind,
        elements: &[TokenTree],
        call_len: usize,
        cur_ret_last_len: usize,
    ) -> bool {
        let call_origin = &self.source[kind.start_pos as usize..kind.end_pos as usize];
        let mut len = cur_ret_last_len + call_len;
        len += if call_origin.len() == call_len { 0 } else { 4 };

        let mut simple_token_cnt = 0;
        let mut nested_token_cnt = 0;
        let mut comma_cnt = 0;
        let mut bin_op_cnt = 0;
        parse_nested_token_nums(
            elements,
            &mut simple_token_cnt,
            &mut comma_cnt,
            &mut bin_op_cnt,
            &mut nested_token_cnt,
        );
        tracing::debug!(
            "nested_token_cnt = {}, comma_cnt = {}, bin_op_cnt = {}, simple_token_cnt = {}, len = {}",
            nested_token_cnt,
            comma_cnt,
            bin_op_cnt,
            simple_token_cnt,
            len
        );
        if len < config.max_width()
            && nested_token_cnt <= 2
            && comma_cnt < 3
            && bin_op_cnt < 2
            && simple_token_cnt < 32
            && !call_origin.contains("//")
        {
            return false;
        }

        if len <= config.max_width()
            && call_len < config.max_width() / 2
            && !contains_comment(call_origin)
            && call_origin.matches("}").count() < 2
        {
            return false;
        }
        true
    }
}

#[allow(dead_code)]
fn get_call(fmt_buffer: String) {
    use crate::tools::utils::*;
    use move_command_line_common::files::FileHash;
    use move_compiler::parser::syntax::parse_file_string;
    let mut call_extractor = CallHandler::new(&fmt_buffer);
    let (defs, _) = parse_file_string(
        &mut get_compile_env(),
        FileHash::empty(),
        &fmt_buffer.clone(),
    )
    .unwrap();
    call_extractor.preprocess(&Arc::new(defs));

    for call_loc in call_extractor.call_paren_loc_vec.iter() {
        eprintln!(
            "call_exp = \n{}\n\n",
            &call_extractor.source[call_loc.start() as usize..call_loc.end() as usize]
        );
    }
}

#[test]
fn test_get_call() {
    get_call(
        "
        module 0x42::m {
            fun plus_with(self: &S, append: u64): S {
                let token_data_collection = &mut borrow_global_mut<TokenDataCollection<TokenType>>(signer::address_of(
                    account
                )).tokens;
                self.x = self.x + append;
                S { x: self.x }
            }
        }
"
        .to_string());
}

#[test]
fn test_get_call2() {
    get_call(
        "
module 0x42::M {

    fun fmt_error(address: address, x: u64, address2: address): () {

        let taker1_expected_fill_sizes = vector::empty<u64>();
        let taker1_total_fill_size = taker1_expected_fill_sizes.fold(
            0, |acc, x| acc + x
        );
    }
}
"
        .to_string(),
    );
}
