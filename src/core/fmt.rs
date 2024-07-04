// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core::token_tree::*;
use crate::syntax_fmt::bin_op_fmt::BinOpExtractor;
use crate::syntax_fmt::branch_fmt::BranchExtractor;
use crate::syntax_fmt::call_fmt::CallExtractor;
use crate::syntax_fmt::fun_fmt::FunExtractor;
use crate::syntax_fmt::let_fmt::LetExtractor;
use crate::syntax_fmt::{big_block_fmt, expr_fmt, fun_fmt, spec_fmt};
use crate::tools::syntax::{self, parse_file_string};
use crate::tools::utils::{FileLineMappingOneFile, Timer};
use commentfmt::comment::contains_comment;
use commentfmt::{Config, Verbosity};
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::result::Result::*;

const BREAK_LINE_FOR_LOGIC_OP_NUM: u32 = 2;

pub struct FormatContext {
    pub content: String,
    pub cur_tok: Tok,
    pub cur_nested_kind: NestKind,
}

impl FormatContext {
    pub fn new(content: String) -> Self {
        FormatContext {
            content,
            cur_tok: Tok::EOF,
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
        }
    }
}

pub struct SyntaxExtractor {
    pub branch_extractor: BranchExtractor,
    pub fun_extractor: FunExtractor,
    pub call_extractor: CallExtractor,
    pub let_extractor: LetExtractor,
    pub bin_op_extractor: BinOpExtractor,
}

pub struct Format {
    pub local_cfg: FormatConfig,
    pub global_cfg: Config,
    pub depth: Cell<usize>,
    pub token_tree: Vec<TokenTree>,
    pub comments: Vec<Comment>,
    pub line_mapping: FileLineMappingOneFile,
    pub comments_index: Cell<usize>,
    pub ret: RefCell<String>,
    pub cur_line: Cell<u32>,
    pub format_context: RefCell<FormatContext>,
    pub syntax_extractor: SyntaxExtractor,
}

#[derive(Clone, Default)]
pub struct FormatConfig {
    pub max_with: usize,
    pub indent_size: usize,
}

impl Format {
    fn new(global_cfg: Config, content: &str, format_context: FormatContext) -> Self {
        let ce: CommentExtrator = CommentExtrator::new(content).unwrap();
        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(content);
        let syntax_extractor = SyntaxExtractor {
            branch_extractor: BranchExtractor::new(content.to_string()),
            fun_extractor: FunExtractor::new(content.to_string()),
            call_extractor: CallExtractor::new(content.to_string()),
            let_extractor: LetExtractor::new(content.to_string()),
            bin_op_extractor: BinOpExtractor::new(content.to_string()),
        };
        Self {
            comments_index: Default::default(),
            local_cfg: FormatConfig {
                max_with: global_cfg.max_width(),
                indent_size: global_cfg.indent_size(),
            },
            global_cfg,
            depth: Default::default(),
            token_tree: vec![],
            comments: ce.comments,
            line_mapping,
            ret: Default::default(),
            cur_line: Default::default(),
            format_context: format_context.into(),
            syntax_extractor,
        }
    }

    fn generate_token_tree(&mut self, content: &str) -> Result<String, Diagnostics> {
        // let attrs: BTreeSet<String> = BTreeSet::new();
        let mut env = CompilationEnv::new(Flags::testing(), BTreeSet::new());
        let (defs, _) = parse_file_string(&mut env, FileHash::empty(), content)?;
        let lexer = Lexer::new(content, FileHash::empty());
        let parse = crate::core::token_tree::Parser::new(lexer, &defs, content.to_string());
        self.token_tree = parse.parse_tokens();
        self.syntax_extractor
            .branch_extractor
            .preprocess(defs.clone());
        self.syntax_extractor.fun_extractor.preprocess(defs.clone());
        self.syntax_extractor
            .call_extractor
            .preprocess(defs.clone());
        self.syntax_extractor.let_extractor.preprocess(defs.clone());
        self.syntax_extractor
            .bin_op_extractor
            .preprocess(defs.clone());
        Ok("parse ok".to_string())
    }

    fn post_process(&mut self) {
        tracing::debug!("post_process >> meet Brace");
        self.remove_trailing_whitespaces();
        tracing::debug!("post_process -- fmt_fun");
        *self.ret.borrow_mut() =
            fun_fmt::fmt_fun(self.ret.clone().into_inner(), self.global_cfg.clone());
        tracing::debug!("post_process -- split_if_else_in_let_block");
        if self.ret.clone().into_inner().contains("spec ") {
            *self.ret.borrow_mut() =
                spec_fmt::fmt_spec(self.ret.clone().into_inner(), self.global_cfg.clone());
        }
        tracing::debug!("post_process -- fmt_big_block");
        *self.ret.borrow_mut() = big_block_fmt::fmt_big_block(self.ret.clone().into_inner());
        self.remove_trailing_whitespaces();

        {
            // this step must before add_comments. because there maybe some comments before new module
            // https://github.com/movebit/movefmt/issues/1
            self.process_last_empty_line();
        }

        tracing::debug!("post_process << done !!!");
    }

    pub fn format_token_trees(mut self) -> String {
        let length = self.token_tree.len();
        let mut index = 0;
        let mut pound_sign = None;
        while index < length {
            let t = self.token_tree.get(index).unwrap();
            if t.is_pound() {
                pound_sign = Some(index);
            }
            let new_line = pound_sign.map(|x| (x + 1) == index).unwrap_or_default();
            self.format_token_trees_internal(t, self.token_tree.get(index + 1), new_line);
            if new_line {
                self.new_line(Some(t.end_pos()));
                pound_sign = None;
            }
            // top level
            match t {
                TokenTree::SimpleToken { .. } => {}
                TokenTree::Nested {
                    elements: _, kind, ..
                } => {
                    if kind.kind == NestKind_::Brace {
                        self.new_line(Some(t.end_pos()));
                        self.post_process();
                    }
                }
            }
            index += 1;
        }
        self.add_comments(u32::MAX, "end_of_move_file".to_string());
        self.process_last_empty_line();
        self.ret.into_inner()
    }

    fn is_long_nested_token(current: &TokenTree) -> (bool, usize) {
        let (mut result, mut elements_len) = (false, 0);
        if let TokenTree::Nested {
            elements,
            kind,
            note: _,
        } = current
        {
            result = matches!(kind.kind, NestKind_::Brace | NestKind_::ParentTheses)
                && analyze_token_tree_length(elements, 64) > 32;
            elements_len = elements.len();
        }
        (result, elements_len)
    }

    fn check_next_token_canbe_break_in_nested(next: Option<&TokenTree>) -> bool {
        if let Some((next_tok, next_content)) = next.map(|x| match x {
            TokenTree::SimpleToken {
                content,
                pos: _,
                tok,
                ..
            } => (*tok, content.clone()),
            TokenTree::Nested {
                elements: _, kind, ..
            } => (kind.kind.start_tok(), kind.kind.start_tok().to_string()),
        }) {
            match next_tok {
                Tok::Friend
                | Tok::Const
                | Tok::Fun
                | Tok::While
                | Tok::Use
                | Tok::Struct
                | Tok::Spec
                | Tok::Return
                | Tok::Public
                | Tok::Native
                | Tok::Move
                | Tok::Module
                | Tok::Loop
                | Tok::Let
                | Tok::Invariant
                | Tok::If
                | Tok::Continue
                | Tok::Break
                | Tok::NumSign
                | Tok::Amp
                | Tok::LParen
                | Tok::Abort => true,
                Tok::Identifier => next_content.as_str() == "entry",
                _ => false,
            }
        } else {
            true
        }
    }

    fn check_cur_token_is_long_bin_op(
        &self,
        current: &TokenTree,
        next: Option<&TokenTree>,
    ) -> bool {
        if matches!(current.simple_str().unwrap_or_default(), "==>" | "<==>") {
            let next_token_start_pos = next.unwrap().start_pos();
            if self.translate_line(next_token_start_pos)
                <= self.translate_line(current.end_pos()) + 1
                && self
                    .syntax_extractor
                    .let_extractor
                    .is_long_bin_op(current.clone())
            {
                return true;
            }
        }
        false
    }

    fn check_next_token_is_long_bin_op(
        &self,
        current: &TokenTree,
        next_t: Option<&TokenTree>,
        next_token: Tok,
    ) -> bool {
        if matches!(next_token, |Tok::AmpAmp| Tok::PipePipe)
            && self
                .syntax_extractor
                .let_extractor
                .is_long_bin_op(next_t.unwrap().clone())
        {
            return true;
        }
        if matches!(
            next_token,
            Tok::EqualEqualGreater | Tok::LessEqualEqualGreater | Tok::Identifier
        ) {
            return false;
        }
        let len_plus_cur_token = self.last_line().len() + current.token_len() as usize + 2;
        if len_plus_cur_token > self.global_cfg.max_width() {
            return false;
        }
        if next_token != Tok::Identifier {
            let r_exp_len_tuple = self
                .syntax_extractor
                .bin_op_extractor
                .get_bin_op_right_part_len(next_t.unwrap().clone());
            if r_exp_len_tuple == (0, 0) {
                return false;
            }
            if next_token == Tok::AmpAmp {
                tracing::debug!(
                    "self.last_line().len() = {:?}, r_exp_len_tuple = {:?}",
                    self.last_line().len(),
                    r_exp_len_tuple
                );
            }
            if len_plus_cur_token + r_exp_len_tuple.1 > self.global_cfg.max_width() {
                self.syntax_extractor
                    .bin_op_extractor
                    .record_long_op(r_exp_len_tuple.0);
                return true;
            }
        }
        false
    }

    fn check_new_line_mode_for_each_token_in_nested(
        &self,
        kind_outer: &NestKind,
        delimiter: Option<Delimiter>,
        _has_colon: bool,
        current: &TokenTree,
        next: Option<&TokenTree>,
    ) -> bool {
        if next.and_then(|x| x.simple_str()) == delimiter.map(|x| x.to_static_str()) {
            return false;
        }

        let b_judge_next_token = Self::check_next_token_canbe_break_in_nested(next);

        // special case for `}}`
        if let TokenTree::Nested {
            elements: _,
            kind,
            note: _,
        } = current
        {
            if kind.kind == NestKind_::Brace
                && kind_outer.kind == NestKind_::Brace
                && b_judge_next_token
            {
                return true;
            }
        }

        // added in 20240426: special case for current is long nested type
        if matches!(kind_outer.kind, NestKind_::Brace | NestKind_::ParentTheses) {
            let result_inner = Self::is_long_nested_token(current);
            if b_judge_next_token && result_inner.0 && result_inner.1 > 4 {
                return true;
            }
        }
        false
    }

    fn get_new_line_mode_for_each_token_in_nested(
        &self,
        kind_outer: &NestKind,
        current: &TokenTree,
        next: Option<&TokenTree>,
    ) -> bool {
        if kind_outer.end_pos - current.end_pos() < 16 {
            return false;
        }
        let b_judge_next_token =
            next.is_some() && Self::check_next_token_canbe_break_in_nested(next);
        if matches!(kind_outer.kind, NestKind_::Brace | NestKind_::ParentTheses)
            && b_judge_next_token
            && Self::is_long_nested_token(current).0
        {
            return true;
        }
        false
    }

    fn need_new_line_for_each_token_finished_in_nested(
        &self,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        index: usize,
        component_break_mode: bool,
    ) -> bool {
        let TokenTree::Nested {
            elements,
            kind,
            note,
        } = nested_token
        else {
            return false;
        };
        // updated in 20240517: not break line for big vec[]
        if kind.kind == NestKind_::Bracket && elements.len() > 128 {
            return false;
        }

        let t = elements.get(index).unwrap();
        let next_t = elements.get(index + 1);
        let d = delimiter.map(|x| x.to_static_str());
        let t_str = t.simple_str();
        let stct_def_or_fn_body = note
            .map(|x| x == Note::StructDefinition || x == Note::FunBody)
            .unwrap_or_default();

        let mut new_line = if component_break_mode {
            self.check_new_line_mode_for_each_token_in_nested(kind, delimiter, has_colon, t, next_t)
                || (d == t_str && d.is_some())
        } else {
            self.get_new_line_mode_for_each_token_in_nested(kind, t, next_t)
        };
        new_line |= self.check_cur_token_is_long_bin_op(t, next_t);

        // comma in fun resource access specifier not change new line
        if d == t_str && d.is_some() {
            if let Some(deli_str) = d {
                if deli_str.contains(',') {
                    let mut idx = index;
                    while idx != 0 {
                        let ele = elements.get(idx).unwrap();
                        idx -= 1;
                        if let Some(key) = ele.simple_str() {
                            if key.contains("fun") {
                                break;
                            }
                        }
                        if ele.simple_str().is_none() {
                            continue;
                        }
                        if matches!(
                            ele.simple_str().unwrap(),
                            "acquires" | "reads" | "writes" | "pure"
                        ) {
                            new_line = false;
                            break;
                        }
                    }
                }
            }
        }

        // ablility not change new line
        // optimize in 20240510: maybe like variable name or struct field name are ability, like "key"
        let mut next_token = Tok::EOF;
        if let Some((next_tok, next_content)) = next_t.map(|x| match x {
            TokenTree::SimpleToken {
                content,
                pos: _,
                tok,
                ..
            } => (*tok, content.clone()),
            TokenTree::Nested {
                elements: _, kind, ..
            } => (kind.kind.start_tok(), kind.kind.start_tok().to_string()),
        }) {
            if !stct_def_or_fn_body && syntax::token_to_ability(next_tok, &next_content).is_some() {
                new_line = false;
            }
            next_token = next_tok;
        }

        // added in 20240430: check expression with (&&, ||, ...)
        if kind.kind == NestKind_::ParentTheses {
            let elements_str = serde_json::to_string(&elements).unwrap_or_default();
            let and_op_num = elements_str.matches("&&").count() as u32;
            let or_op_num = elements_str.matches("||").count() as u32;
            let mut b_need_check_logic_op = false;
            if and_op_num > BREAK_LINE_FOR_LOGIC_OP_NUM
                || or_op_num > BREAK_LINE_FOR_LOGIC_OP_NUM
                || and_op_num + or_op_num > BREAK_LINE_FOR_LOGIC_OP_NUM
            {
                b_need_check_logic_op = true;
            }

            let elements_len = analyze_token_tree_length(elements, 100);
            let elements_limit = 75;
            if (and_op_num == BREAK_LINE_FOR_LOGIC_OP_NUM
                || or_op_num == BREAK_LINE_FOR_LOGIC_OP_NUM
                || and_op_num + or_op_num == BREAK_LINE_FOR_LOGIC_OP_NUM)
                && elements_len > elements_limit
            {
                b_need_check_logic_op = true;
            }

            if b_need_check_logic_op && matches!(next_token, Tok::PipePipe | Tok::AmpAmp) {
                new_line = true;
            }
        }

        if !new_line && next_t.is_some() {
            let next_token_start_pos = next_t.unwrap().start_pos();
            if self.translate_line(next_token_start_pos) != self.translate_line(t.end_pos())
                && next_token != Tok::If
            {
                return new_line;
            }
            if self.check_next_token_is_long_bin_op(t, next_t, next_token) {
                return true;
            }
            // updated in 20240607: fix https://github.com/movebit/movefmt/issues/7
            if t.simple_str().unwrap_or_default() == "="
                && next_t.unwrap().simple_str().unwrap_or_default() != "vector"
                && next_token != Tok::LBrace
                && self
                    .syntax_extractor
                    .call_extractor
                    .component_is_complex_blk(
                        self.global_cfg.clone(),
                        kind,
                        elements,
                        index as i64,
                        self.get_cur_line_len(),
                    )
                    != 2
                && self
                    .syntax_extractor
                    .let_extractor
                    .is_long_assign(t.clone())
            {
                return true;
            }
        }
        new_line
    }

    fn process_fn_header(&self) {
        let cur_ret = self.ret.clone().into_inner();
        // tracing::debug!("fun_header = {:?}", &self.format_context.content[(kind.start_pos as usize)..(kind.end_pos as usize)]);
        if let Some(last_fun_idx) = cur_ret.rfind("fun") {
            let fun_header: &str = &cur_ret[last_fun_idx..];
            if let Some(specifier_idx) = fun_header.rfind("fun") {
                let indent_str = " "
                    .to_string()
                    .repeat((self.depth.get() + 1) * self.local_cfg.indent_size);
                let fun_specifier_fmted_str = fun_fmt::fun_header_specifier_fmt(
                    &fun_header[specifier_idx + 1..],
                    &indent_str,
                );

                let ret_copy = &self.ret.clone().into_inner()[0..last_fun_idx + specifier_idx + 1];
                let mut new_ret = ret_copy.to_string();
                new_ret.push_str(fun_specifier_fmted_str.as_str());
                *self.ret.borrow_mut() = new_ret.to_string();
            }
        }
        if self.ret.clone().into_inner().contains("writes") {
            tracing::debug!("self.last_line = {:?}", self.last_line());
        }
    }

    fn get_break_mode_begin_nested(
        &self,
        token: &TokenTree,
        delimiter: Option<Delimiter>,
    ) -> (bool, Option<bool>) {
        let TokenTree::Nested {
            elements,
            kind,
            note,
        } = token
        else {
            return (false, None);
        };
        let stct_def = note
            .map(|x| x == Note::StructDefinition)
            .unwrap_or_default();
        let fun_body = note.map(|x| x == Note::FunBody).unwrap_or_default();
        let max_len_when_no_add_line = self.global_cfg.max_width() as f32 * 0.75;
        let nested_blk_str =
            &self.format_context.borrow().content[kind.start_pos as usize..kind.end_pos as usize];
        let nested_blk_str_trim_multi_space = nested_blk_str
            .replace('\n', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        let nested_token_len = nested_blk_str_trim_multi_space.len();

        if fun_body {
            self.process_fn_header();
        }

        if elements.is_empty() {
            return (nested_token_len as f32 > max_len_when_no_add_line, None);
        }

        // 20240329 updated
        // fun body brace always change new line;
        // if ParentTheses is empty, not change new line;
        // 20240425 updated
        // The value of new_line_mode here is not associated with Paren, only with Brace.
        // Because Paren may come from fn_para or call or expression statements...
        let mut new_line_mode = {
            delimiter
                .map(|x| x == Delimiter::Semicolon)
                .unwrap_or_default()
                || stct_def
                || fun_body
                || (self.get_cur_line_len() + nested_token_len > self.global_cfg.max_width()
                    && nested_token_len > 8
                    && kind.kind == NestKind_::Brace)
        };
        if new_line_mode && kind.kind != NestKind_::Type {
            if stct_def {
                return (true, Some(true));
            }
            return (true, None);
        }

        let first_ele_len =
            analyze_token_tree_length(&[elements[0].clone()], self.global_cfg.max_width());
        match kind.kind {
            NestKind_::Type => {
                // added in 20240112: if type in fun header, not change new line
                if self
                    .syntax_extractor
                    .fun_extractor
                    .is_generic_ty_in_fun_header(kind)
                {
                    return (false, None);
                }
                new_line_mode = self.get_cur_line_len() + first_ele_len
                    > self.global_cfg.max_width()
                    && first_ele_len > 8;
            }
            NestKind_::ParentTheses => {
                let nested_and_comma_pair = expr_fmt::get_nested_and_comma_num(elements);
                let mut opt_component_break_mode = nested_token_len >= self.global_cfg.max_width();
                if matches!(self.format_context.borrow().cur_tok, Tok::If | Tok::While) {
                    new_line_mode = false;
                } else if self
                    .syntax_extractor
                    .fun_extractor
                    .is_parameter_paren_in_fun_header(kind)
                {
                    let header_str = &self.format_context.borrow().content
                        [token.start_pos() as usize..token.end_pos() as usize];
                    if header_str.matches("\n").count() > 2 {
                        opt_component_break_mode |= nested_and_comma_pair.1 >= 3;
                        new_line_mode = true;
                    }

                    // Reserve 25% space for return ty and specifier
                    new_line_mode |= (self.get_cur_line_len() + nested_token_len) as f32
                        > max_len_when_no_add_line;

                    opt_component_break_mode |= (nested_and_comma_pair.0 >= 4
                        || nested_and_comma_pair.1 >= 4)
                        && token.token_len() as f32 > max_len_when_no_add_line;
                    new_line_mode |= opt_component_break_mode;
                } else if self.get_cur_line_len() > self.global_cfg.max_width() {
                    new_line_mode = true;
                } else {
                    let elements_str = serde_json::to_string(&elements).unwrap_or_default();
                    let has_multi_para = elements_str.matches("\"content\":\",\"").count() >= 4;
                    let is_in_fun_call = self.syntax_extractor.call_extractor.paren_in_call(kind);
                    if is_in_fun_call {
                        if self
                            .syntax_extractor
                            .call_extractor
                            .get_call_component_split_mode(
                                self.global_cfg.clone(),
                                kind,
                                &elements,
                                self.last_line().len(),
                            )
                        {
                            new_line_mode = true;

                            let next_line_len = " "
                                .to_string()
                                .repeat(self.depth.get() * (self.local_cfg.indent_size) + 1)
                                .len();
                            if self
                                .syntax_extractor
                                .call_extractor
                                .get_call_component_split_mode(
                                    self.global_cfg.clone(),
                                    kind,
                                    &elements,
                                    next_line_len,
                                )
                            {
                                opt_component_break_mode = true;
                            }
                        }

                        if has_multi_para && nested_token_len as f32 > max_len_when_no_add_line {
                            opt_component_break_mode = true;
                        }
                    } else {
                        new_line_mode |= has_multi_para
                            && self.format_context.borrow().cur_tok == Tok::Identifier;
                    }

                    let is_plus_first_ele_over_width = self.get_cur_line_len() + first_ele_len
                        > self.global_cfg.max_width()
                        && first_ele_len > 8;
                    let is_plus_nested_over_width = self.get_cur_line_len() + nested_token_len
                        > self.global_cfg.max_width()
                        && nested_token_len > 8;
                    let is_nested_len_too_large =
                        nested_token_len as f32 > 2.0 * max_len_when_no_add_line;

                    new_line_mode |= is_plus_first_ele_over_width;
                    new_line_mode |= is_nested_len_too_large;
                    new_line_mode |= is_plus_nested_over_width;
                    // 20240619: remove `first_para_is_complex_blk`
                }
                return (new_line_mode, Some(opt_component_break_mode));
            }
            NestKind_::Bracket => {
                new_line_mode = nested_token_len as f32 > max_len_when_no_add_line;
                if elements.len() > 32 {
                    return (new_line_mode, Some(false));
                }
            }
            NestKind_::Lambda => {
                if delimiter.is_none() && nested_token_len as f32 <= max_len_when_no_add_line {
                    new_line_mode = false;
                }
            }
            NestKind_::Brace => {
                new_line_mode = (has_special_key_for_break_line_in_code_buf(self.last_line())
                    && nested_token_len > 4)
                    || nested_token_len as f32 > max_len_when_no_add_line
                    || (self.last_line().len() + nested_token_len > self.global_cfg.max_width()
                        && nested_token_len > 4)
                    || (contains_comment(nested_blk_str) && nested_blk_str.lines().count() > 1);

                if self
                    .syntax_extractor
                    .branch_extractor
                    .com_if_else
                    .then_loc_vec
                    .iter()
                    .any(|&x| x.start() == kind.start_pos)
                    || self
                        .syntax_extractor
                        .branch_extractor
                        .com_if_else
                        .else_loc_vec
                        .iter()
                        .any(|&x| x.start() == kind.start_pos)
                {
                    if self.global_cfg.prefer_one_line_for_short_branch_blk() {
                        new_line_mode |= nested_token_len > 8;
                    } else {
                        new_line_mode |= true;
                    }
                }
            }
        }
        (new_line_mode, None)
    }

    fn top_half_after_kind_start(
        &self,
        kind: &NestKind,
        elements: &[TokenTree],
        b_new_line_mode: bool,
        b_add_indent: bool,
        b_add_space_around_brace: bool,
    ) {
        // step1 -- format start_token
        self.format_token_trees_internal(&kind.start_token_tree(), None, b_new_line_mode);

        // step2 -- paired effect with step6
        if b_add_indent {
            self.inc_depth();
        }

        // step3
        if b_new_line_mode {
            self.add_new_line_after_nested_begin(kind, elements, b_new_line_mode);
        } else if b_add_space_around_brace {
            self.push_str(" ");
        }
    }

    fn bottom_half_before_kind_end(
        &self,
        kind: &NestKind,
        b_new_line_mode: bool,
        b_add_indent: bool,
        b_add_space_around_brace: bool,
        nested_token_head: Tok,
        _opt_component_break_mode: bool,
    ) {
        // step5 -- add_comments which before kind.end_pos
        self.add_comments(
            kind.end_pos,
            kind.end_token_tree()
                .simple_str()
                .unwrap_or_default()
                .to_string(),
        );
        let ret_copy = self.ret.clone().into_inner();
        // may be already add_a_new_line in step5 by add_comments(doc_comment in tail of line)
        *self.ret.borrow_mut() = ret_copy.trim_end().to_string();
        let had_rm_added_new_line =
            self.ret.clone().into_inner().lines().count() < ret_copy.lines().count();

        // step6 -- paired effect with step2
        if b_add_indent {
            self.dec_depth();
        }

        // step7
        if b_new_line_mode || had_rm_added_new_line {
            tracing::debug!(
                "end_of_nested_block, had_rm_added_new_line = {}, last_ret = {}",
                had_rm_added_new_line,
                self.last_line()
            );
            let mut b_break_line_before_kind_end = true;
            if nested_token_head == Tok::If
                || kind.kind == NestKind_::Bracket
                || kind.kind == NestKind_::Type
            {
                // 20240426 -- for [] and <>  don't add new line
                b_break_line_before_kind_end = false;
            }

            if contains_comment(&self.last_line()) && self.last_line().contains("//") {
                b_break_line_before_kind_end = true;
            }
            if b_break_line_before_kind_end {
                tracing::trace!(
                    "end_of_nested_block, new_line(), last_ret = {}",
                    self.last_line()
                );
                self.new_line(Some(kind.end_pos));
            }
        } else if b_add_space_around_brace {
            self.push_str(" ");
        }
    }

    fn add_new_line_after_nested_begin(
        &self,
        kind: &NestKind,
        elements: &[TokenTree],
        b_new_line_mode: bool,
    ) {
        if !b_new_line_mode {
            return;
        }

        if !elements.is_empty() {
            let next_token_start_pos = elements.first().unwrap().start_pos();
            if self.translate_line(next_token_start_pos) > self.translate_line(kind.start_pos) {
                self.process_same_line_comment(kind.start_pos, true);
                return self.new_line(None);
            }
        }
        self.new_line(Some(kind.start_pos));
    }

    fn format_single_token(
        &self,
        nested_token: &TokenTree,
        internal_token_idx: usize,
        pound_sign_new_line: bool,
        new_line: bool,
        pound_sign: &mut Option<usize>,
        need_add_comma_after_last_ele: bool,
    ) {
        let TokenTree::Nested {
            elements,
            kind: _,
            note: _,
        } = nested_token
        else {
            return;
        };
        let token = elements.get(internal_token_idx).unwrap();
        let next_t = elements.get(internal_token_idx + 1);

        self.format_token_trees_internal(token, next_t, pound_sign_new_line || new_line);
        if need_add_comma_after_last_ele {
            self.push_str(",");
        }

        if pound_sign_new_line {
            tracing::debug!("in loop<TokenTree::Nested> pound_sign_new_line = true");
            self.new_line(Some(token.end_pos()));
            *pound_sign = None;
            return;
        }

        if new_line {
            let process_tail_comment_of_line = match next_t {
                Some(next_token) => {
                    let next_token_start_pos = next_token.start_pos();
                    self.translate_line(next_token_start_pos) > self.translate_line(token.end_pos())
                }
                None => {
                    let remain_code_str =
                        &self.format_context.borrow().content[token.end_pos() as usize..];
                    let mut remain_code_iter = remain_code_str.split_whitespace().clone();
                    let remain_code_first_word = remain_code_iter.next().unwrap_or_default();
                    remain_code_first_word.starts_with("//")
                        || remain_code_first_word.starts_with("/*")
                }
            };
            self.process_same_line_comment(token.end_pos(), process_tail_comment_of_line);
            self.new_line(None);
        }
    }

    fn format_each_token_in_nested_elements(
        &self,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        component_break_mode: bool,
    ) {
        let TokenTree::Nested {
            elements,
            kind,
            note: _,
        } = nested_token
        else {
            return;
        };
        let old_kind = self.format_context.borrow_mut().cur_nested_kind;
        self.format_context.borrow_mut().cur_nested_kind = *kind;
        let mut pound_sign = None;
        let len = elements.len();
        let mut internal_token_idx = 0;
        while internal_token_idx < len {
            let pound_sign_new_line = pound_sign
                .map(|x| (x + 1) == internal_token_idx)
                .unwrap_or_default();

            let mut new_line = self.need_new_line_for_each_token_finished_in_nested(
                nested_token,
                delimiter,
                has_colon,
                internal_token_idx,
                component_break_mode,
            );

            if kind.kind == NestKind_::ParentTheses {
                new_line |= self
                    .syntax_extractor
                    .call_extractor
                    .should_call_component_split(
                        self.global_cfg.clone(),
                        kind,
                        elements,
                        internal_token_idx,
                        self.get_cur_line_len(),
                    );
            }

            // This code determines whether a comma should be added after the last argument in a function call
            // if the arguments are split across multiple lines.
            // Add a comment if the last argument needs to be on a new line and there is no comma after the last argument.
            let mut need_add_comma_after_last_ele = false;
            if internal_token_idx == len - 1
                && kind.kind == NestKind_::ParentTheses
                && self.syntax_extractor.call_extractor.paren_in_call(kind)
            {
                if component_break_mode
                    && elements
                        .get(internal_token_idx)
                        .unwrap()
                        .simple_str()
                        .unwrap_or_default()
                        != ","
                {
                    need_add_comma_after_last_ele = true;
                }

                if !component_break_mode
                    && elements
                        .get(internal_token_idx)
                        .unwrap()
                        .simple_str()
                        .unwrap_or_default()
                        == ","
                {
                    internal_token_idx += 1;
                    continue;
                }
            }

            if elements.get(internal_token_idx).unwrap().is_pound() {
                pound_sign = Some(internal_token_idx)
            }

            if Tok::Period == self.format_context.borrow().cur_tok {
                let in_link_access =
                    expr_fmt::process_link_access(elements, internal_token_idx + 1);
                let mut last_dot_idx = in_link_access.1;
                let mut need_process_link =
                    in_link_access.0 > 3 && last_dot_idx > internal_token_idx;
                if !need_process_link {
                    let in_link_call = self
                        .syntax_extractor
                        .call_extractor
                        .is_in_link_call(elements, internal_token_idx + 1);
                    last_dot_idx = in_link_call.1;
                    if in_link_call.0 && last_dot_idx > internal_token_idx {
                        tracing::trace!(
                            "in_link_call, in_link_call = {:?}, last_line = {}",
                            in_link_call,
                            self.last_line()
                        );
                        need_process_link = true;
                    }
                }

                if need_process_link {
                    tracing::debug!("before process_link, last_line = {}", self.last_line());
                    self.inc_depth();
                    let mut is_dot_new_line;
                    while internal_token_idx <= last_dot_idx + 1 {
                        is_dot_new_line = match elements.get(internal_token_idx + 1) {
                            None => false,
                            Some(next_t) => next_t.simple_str().unwrap_or_default().contains('.'),
                        };
                        self.format_single_token(
                            &nested_token,
                            internal_token_idx,
                            false,
                            is_dot_new_line,
                            &mut pound_sign,
                            need_add_comma_after_last_ele,
                        );
                        internal_token_idx += 1;
                    }
                    // tracing::debug!("after processed link access, ret = {}", self.ret.clone().into_inner());
                    // tracing::debug!("after processed link access, internal_token_idx = {}, len = {}", internal_token_idx, len);
                    self.dec_depth();
                    continue;
                }
            }

            self.format_single_token(
                &nested_token,
                internal_token_idx,
                pound_sign_new_line,
                new_line,
                &mut pound_sign,
                need_add_comma_after_last_ele,
            );
            internal_token_idx += 1;
        }

        self.format_context.borrow_mut().cur_nested_kind = old_kind;
    }

    fn judge_add_space_around_brace(
        &self,
        nested_token: &TokenTree,
        b_new_line_mode: bool,
    ) -> bool {
        let TokenTree::Nested {
            elements,
            kind,
            note: _,
        } = nested_token
        else {
            return true;
        };
        let nested_token_head = self.format_context.borrow().cur_tok;
        // optimize in 20240425
        // there are 2 cases which not add space
        // eg1: When braces are used for arithmetic operations
        // let intermediate3: u64 = (a * {c + d}) - (b / {e - 2});
        // shouldn't formated like `let intermediate3: u64 = (a * { c + d }) - (b / { e - 2 });`
        // eg2: When the braces are used for use
        // use A::B::{C, D}
        // shouldn't formated like `use A::B::{ C, D }`
        let b_not_arithmetic_op_brace = Tok::Plus != nested_token_head
            && Tok::Minus != nested_token_head
            && Tok::Star != nested_token_head
            && Tok::Slash != nested_token_head
            && Tok::Percent != nested_token_head
            && kind.kind == NestKind_::Brace;
        let b_not_use_brace = Tok::ColonColon != nested_token_head && kind.kind == NestKind_::Brace;
        let nested_blk_str = &self.format_context.borrow().content
            [kind.start_pos as usize + 1..kind.end_pos as usize];
        (elements.is_empty() && contains_comment(nested_blk_str))
            || (b_not_arithmetic_op_brace
                && b_not_use_brace
                && !b_new_line_mode
                && !elements.is_empty())
    }

    fn format_nested_token(&self, nested_token: &TokenTree, next_token: Option<&TokenTree>) {
        let TokenTree::Nested {
            elements,
            kind,
            note,
        } = nested_token
        else {
            return;
        };
        let (delimiter, has_colon) = analyze_token_tree_delimiter(elements);
        let (b_new_line_mode, opt_component_break_mode) =
            self.get_break_mode_begin_nested(nested_token, delimiter);

        let b_add_indent = elements.is_empty()
            || elements.first().unwrap().simple_str().unwrap_or_default() != "module";
        let nested_token_head = self.format_context.borrow().cur_tok;
        if Tok::NumSign == nested_token_head {
            self.push_str(fun_fmt::process_fun_annotation(*kind, elements.to_vec()));
            return;
        }

        let fun_body = note.map(|x| x == Note::FunBody).unwrap_or_default();
        if fun_body
            && self
                .syntax_extractor
                .fun_extractor
                .should_skip_this_fun_body(kind)
        {
            let fun_body_str = &self.format_context.borrow().content
                [kind.start_pos as usize..kind.end_pos as usize + 1];
            tracing::trace!("should_skip_this_fun_body = {:?}", fun_body_str);
            self.push_str(fun_body_str);

            for c in &self.comments[self.comments_index.get()..] {
                if c.start_offset > kind.end_pos {
                    break;
                }
                self.comments_index.set(self.comments_index.get() + 1);
            }
            self.cur_line.set(self.translate_line(kind.end_pos));
            return;
        }

        if b_new_line_mode && !elements.is_empty() {
            tracing::debug!(
                "nested_token_head = [{:?}], add a new line before {:?}; opt_component_break_mode = {:?}; b_add_indent = {:?};",
                nested_token_head,
                elements.first().unwrap().simple_str(),
                opt_component_break_mode,
                b_add_indent
            );
        }
        let b_add_space_around_brace =
            self.judge_add_space_around_brace(nested_token, b_new_line_mode);

        // step1-step3
        self.top_half_after_kind_start(
            kind,
            elements,
            b_new_line_mode,
            b_add_indent,
            b_add_space_around_brace,
        );

        // step4 -- format element
        self.format_each_token_in_nested_elements(
            nested_token,
            delimiter,
            has_colon,
            opt_component_break_mode.unwrap_or(b_new_line_mode),
        );

        // step5-step7
        self.bottom_half_before_kind_end(
            kind,
            b_new_line_mode,
            b_add_indent,
            b_add_space_around_brace,
            nested_token_head,
            opt_component_break_mode.unwrap_or(b_new_line_mode),
        );

        // step8 -- format end_token
        self.format_token_trees_internal(&kind.end_token_tree(), None, false);
        if expr_fmt::need_space(nested_token, next_token) {
            self.push_str(" ");
        }
    }

    fn top_half_process_branch_new_line_when_fmt_simple_token(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
    ) {
        // updated in 20240517: add condition `NestKind_::Bracket`
        if self.format_context.borrow().cur_nested_kind.kind == NestKind_::Bracket {
            return;
        }
        if let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        {
            // added in 20240115
            // updated in 20240124
            if Tok::LBrace != *tok
                && content != "for"
                && self
                    .syntax_extractor
                    .branch_extractor
                    .need_new_line_after_branch(self.last_line(), *pos, self.global_cfg.clone())
            {
                tracing::debug!("need_new_line_after_branch[{:?}], add a new line", content);
                self.inc_depth();
                self.new_line(None);
            }

            // process `else if`
            // updated in 20240516: optimize break line before else
            if *tok == Tok::Else {
                let get_cur_line_len = self.get_cur_line_len();
                let has_special_key = get_cur_line_len != self.last_line().len();
                if self.format_context.borrow().cur_tok == Tok::RBrace {
                    if has_special_key {
                        // process case:
                        // else if() {} `insert '\n' here` else
                        self.new_line(None);
                    }
                } else if next_token.is_some() {
                    if self.last_line().len()
                        + content.len()
                        + 2
                        + next_token.unwrap().token_len() as usize
                        > self.global_cfg.max_width()
                    {
                        self.new_line(None);
                        return;
                    }
                    let is_in_nested_else_branch = self
                        .syntax_extractor
                        .branch_extractor
                        .is_nested_within_an_outer_else(*pos);
                    if next_token.unwrap().simple_str().unwrap_or_default() == "if"
                        || is_in_nested_else_branch
                    {
                        self.new_line(None);
                    }
                }
            }
        }
    }

    fn bottom_half_process_branch_new_line_when_fmt_simple_token(&self, token: &TokenTree) {
        if let TokenTree::SimpleToken { content, pos, .. } = token {
            // added in 20240115
            // updated in 20240124
            // updated in 20240222: remove condition `if Tok::RBrace != *tok `
            // updated in 20240517: add condition `NestKind_::Bracket`
            if self.format_context.borrow().cur_nested_kind.kind != NestKind_::Bracket {
                let tok_end_pos = *pos + content.len() as u32;
                let mut nested_branch_depth = self
                    .syntax_extractor
                    .branch_extractor
                    .added_new_line_after_branch(tok_end_pos);

                if nested_branch_depth > 0 {
                    tracing::debug!(
                        "nested_branch_depth[{:?}] = [{:?}]",
                        content,
                        nested_branch_depth
                    );
                }
                while nested_branch_depth > 0 {
                    self.dec_depth();
                    nested_branch_depth -= 1;
                }
            }
        }
    }

    fn process_blank_lines_before_simple_token(&self, token: &TokenTree) {
        if let TokenTree::SimpleToken {
            content,
            pos,
            tok,
            note: _,
        } = token
        {
            /*
            ** simple1:
            self.translate_line(*pos) = 6
            after processed xxx, self.cur_line.get() = 5;
            self.translate_line(*pos) - self.cur_line.get() == 1
            """
            line5: // comment xxx
            line6: simple_token
            """
            */
            if (self.translate_line(*pos) - self.cur_line.get()) > 1
                && expr_fmt::need_break_cur_line_when_trim_blank_lines(
                    &self.format_context.borrow().cur_tok,
                    tok,
                )
            {
                // There are multiple blank lines between the cur_line and the current code simple_token
                tracing::debug!(
                    "self.translate_line(*pos) = {}, self.cur_line.get() = {}",
                    self.translate_line(*pos),
                    self.cur_line.get()
                );
                tracing::debug!("SimpleToken[{:?}], add a new line", content);
                self.new_line(None);
            }
        }
    }

    fn fmt_simple_token_core(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if let TokenTree::SimpleToken {
            content,
            pos,
            tok,
            note,
        } = token
        {
            let mut split_line_after_content = false;
            if self.judge_change_new_line_when_over_limits(content.clone(), *tok, *note, next_token)
            {
                tracing::trace!("last_line = {:?}", self.last_line());
                tracing::trace!(
                    "SimpleToken {:?} too long, add a new line because of split line",
                    content
                );

                if matches!(
                    *tok,
                    Tok::Equal
                        | Tok::EqualEqual
                        | Tok::EqualEqualGreater
                        | Tok::LessEqualEqualGreater
                ) {
                    self.push_str(content.as_str());
                    split_line_after_content = true;
                }

                let need_inc_depth = self.format_context.borrow().cur_nested_kind.kind
                    != NestKind_::Bracket
                    && self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses;
                if need_inc_depth {
                    let leading_space_cnt = self.last_line().len()
                        - self
                            .last_line()
                            .clone()
                            .trim_start_matches(char::is_whitespace)
                            .len();
                    let cur_indent_cnt = self.depth.get() * self.local_cfg.indent_size;
                    if leading_space_cnt + self.local_cfg.indent_size == cur_indent_cnt {
                        tracing::debug!("cur_indent_cnt: {}", cur_indent_cnt);
                        self.new_line(None);
                    } else {
                        self.inc_depth();
                        self.new_line(None);
                        self.dec_depth();
                    }
                } else {
                    self.new_line(None);
                }
            }

            if !split_line_after_content {
                self.push_str(content.as_str());
            }

            self.cur_line.set(self.translate_line(*pos));
            if new_line_after {
                return;
            }
            if expr_fmt::need_space(token, next_token) {
                self.push_str(" ");
            }
        }
    }

    fn format_simple_token(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if let TokenTree::SimpleToken {
            content,
            pos,
            tok,
            note: _,
        } = token
        {
            // step1
            self.top_half_process_branch_new_line_when_fmt_simple_token(token, next_token);

            // step2: add comment(xxx) before current simple_token
            self.add_comments(*pos, content.clone());

            // step3
            self.process_blank_lines_before_simple_token(token);

            // step4
            self.bottom_half_process_branch_new_line_when_fmt_simple_token(token);

            // step5
            self.format_context.borrow_mut().cur_tok = *tok;

            // step6
            self.fmt_simple_token_core(token, next_token, new_line_after);
        }
    }

    fn need_inc_depth_when_cur_is_nested(
        &self,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if new_line_after
            && next_token.is_some()
            && self
                .syntax_extractor
                .bin_op_extractor
                .need_inc_depth_by_long_op(next_token.unwrap().clone())
        // && self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses
        {
            tracing::debug!(
                "bin_op_extractor.need_inc_depth_by_long_op({:?})",
                next_token.unwrap().simple_str()
            );
            self.inc_depth();
        }

        if new_line_after
            && next_token.is_some()
            && self
                .syntax_extractor
                .let_extractor
                .need_inc_depth_by_long_op(next_token.unwrap().clone())
            && self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses
        {
            self.inc_depth();
        }
    }

    fn need_inc_depth_when_cur_is_simple(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if new_line_after
            && next_token.is_some()
            && (self
                .syntax_extractor
                .bin_op_extractor
                .need_inc_depth_by_long_op(token.clone())
                || self
                    .syntax_extractor
                    .bin_op_extractor
                    .need_inc_depth_by_long_op(next_token.unwrap().clone()))
        // && self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses
        {
            tracing::debug!(
                "bin_op_extractor.need_inc_depth_by_long_op22({:?})",
                next_token.unwrap().simple_str()
            );
            self.inc_depth();
        }

        // updated in 20240517: add condition `ParentTheses | Brace`
        if new_line_after
            && next_token.is_some()
            && (self
                .syntax_extractor
                .let_extractor
                .need_inc_depth_by_long_op(token.clone())
                || self
                    .syntax_extractor
                    .let_extractor
                    .need_inc_depth_by_long_op(next_token.unwrap().clone()))
            && self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses
        {
            self.inc_depth();
        }
    }

    fn need_dec_depth_when_cur_is_simple(&self, token: &TokenTree) {
        if self
            .syntax_extractor
            .bin_op_extractor
            .need_dec_depth_by_long_op(token.clone())
            > 0
        {
            tracing::debug!(
                "bin_op_extractor.need_dec_depth_by_long_op({:?})",
                token.simple_str()
            );
        }
        let mut nested_break_line_depth = self
            .syntax_extractor
            .bin_op_extractor
            .need_dec_depth_by_long_op(token.clone());
        if self.format_context.borrow().cur_nested_kind.kind != NestKind_::ParentTheses {
            nested_break_line_depth += self
                .syntax_extractor
                .let_extractor
                .need_dec_depth_by_long_op(token.clone());
        }
        if nested_break_line_depth > 0 {
            tracing::debug!(
                "nested_break_line_depth[{:?}] = [{:?}]",
                token.simple_str(),
                nested_break_line_depth
            );
        }
        while nested_break_line_depth > 0 {
            self.dec_depth();
            nested_break_line_depth -= 1;
        }
    }

    fn format_token_trees_internal(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        match token {
            TokenTree::Nested {
                elements: _,
                kind: _,
                note: _,
            } => {
                self.need_inc_depth_when_cur_is_nested(next_token, new_line_after);
                self.format_nested_token(token, next_token);
            }
            TokenTree::SimpleToken { .. } => {
                self.need_inc_depth_when_cur_is_simple(token, next_token, new_line_after);
                self.format_simple_token(token, next_token, new_line_after);
                self.need_dec_depth_when_cur_is_simple(token);
            }
        }
    }

    fn add_comments(&self, pos: u32, content: String) {
        let mut comment_nums_before_cur_simple_token = 0;
        let mut last_cmt_is_block_cmt = false;
        let mut last_cmt_start_pos = 0;
        for c in &self.comments[self.comments_index.get()..] {
            if c.start_offset > pos {
                break;
            }

            let this_cmt_start_line = self.translate_line(c.start_offset);
            if (this_cmt_start_line - self.cur_line.get()) > 1 {
                tracing::debug!(
                    "the pos[{:?}] of this comment > current line[{:?}]",
                    c.start_offset,
                    self.cur_line.get()
                );
                // 20240318: process case as follows
                //
                /*
                #[test(econia = @econia, integrator = @user)]

                // comment
                fun func() {}
                */
                if self.format_context.borrow().cur_tok != Tok::NumSign {
                    self.new_line(None);
                }
            }

            if (this_cmt_start_line - self.cur_line.get()) == 1 {
                // if located after nestedToken start, maybe already chanedLine
                let ret_copy = self.ret.clone().into_inner();
                *self.ret.borrow_mut() = ret_copy.trim_end().to_string();
                self.new_line(None);
            }

            // tracing::debug!("-- add_comments: line(c.start_offset) - cur_line = {:?}",
            //     this_cmt_start_line - self.cur_line.get());
            // tracing::debug!("c.content.as_str() = {:?}\n", c.content.as_str());
            if self.no_space_or_new_line_for_comment() {
                self.push_str(" ");
            }

            self.push_str(c.format_comment(
                c.comment_kind(),
                self.depth.get() * self.local_cfg.indent_size,
                0,
                &self.global_cfg,
            ));

            match c.comment_kind() {
                CommentKind::DocComment => {
                    // let buffer = self.ret.clone();
                    // let len: usize = c.content.len();
                    // let x: usize = buffer.borrow().len();
                    // if len + 2 < x {
                    //     if let Some(ch) = buffer.clone().borrow().chars().nth(x - len - 2) {
                    //         if !ch.is_ascii_whitespace() {
                    //             // insert black space after '//'
                    //             self.ret.borrow_mut().insert(x - len - 1, ' ');
                    //         }
                    //     }
                    // }
                    self.new_line(None);
                    last_cmt_is_block_cmt = false;
                }
                _ => {
                    let end = c.start_offset + (c.content.len() as u32);
                    let line_start = this_cmt_start_line;
                    let line_end = self.translate_line(end);

                    if !content.contains(')') && !content.contains(',') && !content.contains(';') {
                        self.push_str(" ");
                    }
                    if line_start != line_end {
                        self.new_line(None);
                    }
                    last_cmt_is_block_cmt = true;
                }
            }
            self.comments_index.set(self.comments_index.get() + 1);
            self.cur_line
                .set(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));
            comment_nums_before_cur_simple_token += 1;
            last_cmt_start_pos = c.start_offset;
            // tracing::debug!("in add_comments for loop: self.cur_line = {:?}\n", self.cur_line);
        }
        if comment_nums_before_cur_simple_token > 0 {
            if last_cmt_is_block_cmt
                && self.translate_line(pos) - self.translate_line(last_cmt_start_pos) == 1
            {
                // process this case:
                // line[i]: /*comment1*/ /*comment2*/
                // line[i+1]: code // located in `pos`
                self.new_line(None);
            }
            tracing::debug!(
                "add_comments[{:?}] before pos[{:?}] = {:?} return <<<<<<<<<\n",
                comment_nums_before_cur_simple_token,
                pos,
                content
            );
        }
    }
}

impl Format {
    fn inc_depth(&self) {
        let old = self.depth.get();
        self.depth.set(old + 1);
    }
    fn dec_depth(&self) {
        let old = self.depth.get();
        if old == 0 {
            eprintln!("old depth is zero, return");
            return;
        }
        self.depth.set(old - 1);
    }
    fn push_str(&self, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.ret.borrow_mut().push_str(s);
    }

    fn no_space_or_new_line_for_comment(&self) -> bool {
        if self.ret.borrow().chars().last().is_some() {
            !self.ret.borrow().ends_with('\n')
                && !self.ret.borrow().ends_with(' ')
                && !self.ret.borrow().ends_with('(')
        } else {
            false
        }
    }

    fn indent(&self) {
        self.push_str(
            " ".to_string()
                .repeat(self.depth.get() * self.local_cfg.indent_size)
                .as_str(),
        );
    }

    fn translate_line(&self, pos: u32) -> u32 {
        self.line_mapping
            .translate(pos, pos)
            .unwrap_or_default()
            .start
            .line
    }

    fn process_same_line_comment(
        &self,
        add_line_comment_pos: u32,
        process_tail_comment_of_line: bool,
    ) {
        for c in &self.comments[self.comments_index.get()..] {
            if !process_tail_comment_of_line && c.start_offset > add_line_comment_pos {
                break;
            }

            if self.translate_line(add_line_comment_pos) != self.translate_line(c.start_offset) {
                break;
            }
            // tracing::debug!("self.translate_line(c.start_offset) = {}, self.cur_line.get() = {}", self.translate_line(c.start_offset), self.cur_line.get());
            // tracing::debug!("add a new line[{:?}], meet comment", c.content);
            // if (self.translate_line(c.start_offset) - self.cur_line.get()) > 1 {
            //     tracing::debug!("add a black line");
            //     self.new_line(None);
            // }
            // self.push_str(c.content.as_str());
            let kind = c.comment_kind();
            let fmted_cmt_str = c.format_comment(
                kind,
                self.depth.get() * self.local_cfg.indent_size,
                0,
                &self.global_cfg,
            );
            // tracing::debug!("fmted_cmt_str in same_line = \n{}", fmted_cmt_str);
            /*
            let buffer = self.ret.clone();
            if !buffer.clone().borrow().chars().last().unwrap_or(' ').is_ascii_whitespace()
            && !buffer.clone().borrow().chars().last().unwrap_or(' ').eq(&'('){
                self.push_str(" ");
                // insert 2 black space before '//'
                // if let Some(_) = fmted_cmt_str.find("//") {
                //     self.push_str(" ");
                // }
            }
            */
            if self.no_space_or_new_line_for_comment() {
                self.push_str(" ");
            }

            self.push_str(fmted_cmt_str);
            self.comments_index.set(self.comments_index.get() + 1);
            self.cur_line
                .set(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));

            if let CommentKind::BlockComment = kind {
                let end = c.start_offset + (c.content.len() as u32);
                let line_start = self.translate_line(c.start_offset);
                let line_end = self.translate_line(end);
                if line_start != line_end {
                    tracing::debug!("in new_line, add CommentKind::BlockComment");
                    self.new_line(None);
                    return;
                }
            }
        }
    }

    fn new_line(&self, add_line_comment_option: Option<u32>) {
        let (add_line_comment, b_add_comment) = match add_line_comment_option {
            Some(add_line_comment) => (add_line_comment, true),
            _ => (0, false),
        };
        if b_add_comment {
            self.process_same_line_comment(add_line_comment, false);
        }
        self.push_str("\n");
        self.indent();
    }
}

impl Format {
    fn last_line(&self) -> String {
        self.ret
            .borrow()
            .lines()
            .last()
            .map(|x| x.to_string())
            .unwrap_or_default()
    }

    fn tok_suitable_for_new_line(tok: Tok, note: Option<Note>, next: Option<&TokenTree>) -> bool {
        if next
            .and_then(|x| match x {
                TokenTree::SimpleToken { .. } => None,
                TokenTree::Nested {
                    elements: _, kind, ..
                } => Some(kind.kind == NestKind_::Type),
            })
            .unwrap_or_default()
        {
            return false;
        }
        let is_bin = note.map(|x| x == Note::BinaryOP).unwrap_or_default();
        let ret = match tok {
            Tok::Less | Tok::Amp | Tok::Star | Tok::Greater if is_bin => true,
            Tok::ExclaimEqual
            | Tok::Percent
            | Tok::AmpAmp
            | Tok::Plus
            | Tok::Minus
            | Tok::Period
            | Tok::Slash
            | Tok::LessEqual
            | Tok::LessLess
            | Tok::Equal
            | Tok::EqualEqual
            | Tok::EqualEqualGreater
            | Tok::LessEqualEqualGreater
            | Tok::GreaterEqual
            | Tok::GreaterGreater
            | Tok::NumValue
            | Tok::NumTypedValue => true,
            _ => false,
        };
        tracing::debug!("tok_suitable_for_new_line ret = {}", ret);
        ret
    }

    fn get_cur_line_len(&self) -> usize {
        get_code_buf_len(self.last_line())
    }

    fn judge_change_new_line_when_over_limits(
        &self,
        tok_str: String,
        tok: Tok,
        note: Option<Note>,
        next: Option<&TokenTree>,
    ) -> bool {
        let len_plus_tok_len = self.get_cur_line_len() + tok_str.len();
        if tok == Tok::AtSign && next.is_some() {
            let next_tok_len = next.unwrap().simple_str().unwrap_or_default().len();
            if next_tok_len > 8 && len_plus_tok_len + next_tok_len > self.global_cfg.max_width() {
                return true;
            }
        }

        len_plus_tok_len > self.global_cfg.max_width()
            && Self::tok_suitable_for_new_line(tok, note, next)
    }

    fn remove_trailing_whitespaces(&mut self) {
        let ret_copy = self.ret.clone().into_inner();
        let lines = ret_copy.lines();
        let result: String = lines
            .collect::<Vec<_>>()
            .iter()
            .map(|line| line.trim_end_matches(|c| c == ' '))
            .collect::<Vec<_>>()
            .join("\n");
        *self.ret.borrow_mut() = result;
    }

    fn process_last_empty_line(&mut self) {
        let ret_copy = self.ret.clone().into_inner();
        let mut lines = ret_copy.lines().collect::<Vec<&str>>();
        let last_line = lines.last().unwrap_or(&"");

        if last_line.is_empty() {
            while lines.len() > 1 && lines[lines.len() - 2].is_empty() {
                lines.pop();
            }
        } else {
            lines.push("");
        }

        *self.ret.borrow_mut() = lines.join("\n");
    }
}

pub fn format_entry(content: impl AsRef<str>, config: Config) -> Result<String, Diagnostics> {
    let mut timer = Timer::start();
    let content = content.as_ref();

    {
        // https://github.com/movebit/movefmt/issues/2
        let mut env = CompilationEnv::new(Flags::testing(), BTreeSet::new());
        let _ = parse_file_string(&mut env, FileHash::empty(), content)?;
    }

    let mut full_fmt = Format::new(
        config.clone(),
        content,
        FormatContext::new(content.to_string()),
    );

    full_fmt.generate_token_tree(content)?;
    timer = timer.done_parsing();

    let result = full_fmt.format_token_trees();
    timer = timer.done_formatting();
    if config.verbose() == Verbosity::Verbose {
        println!(
            "Spent {0:.3} secs in the parsing phase, and {1:.3} secs in the formatting phase",
            timer.get_parse_time(),
            timer.get_format_time(),
        );
    }
    Ok(result)
}
