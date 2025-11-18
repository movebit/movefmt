// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core::token_tree::*;
use crate::syntax_fmt::bin_op_fmt::BinOpHandler;
use crate::syntax_fmt::branch_fmt::BranchHandler;
use crate::syntax_fmt::call_fmt::*;
use crate::syntax_fmt::fun_fmt::FunHandler;
use crate::syntax_fmt::let_fmt::LetHandler;
use crate::syntax_fmt::quant_fmt::QuantHandler;
use crate::syntax_fmt::skip_fmt::{SkipHandler, SkipType};
use crate::syntax_fmt::syntax_handler::SyntaxHandler;
use crate::syntax_fmt::{big_block_fmt, expr_fmt, fun_fmt, spec_fmt};
use crate::tools::utils::*;
use commentfmt::comment::contains_comment;
use commentfmt::{Config, Verbosity};
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::{ast::*, syntax::parse_file_string};
use move_ir_types::location::ByteIndex;
use std::cell::Cell;
use std::cell::RefCell;
use std::result::Result::*;
use std::sync::{Arc, LazyLock};
use tracing::debug;

const EXIST_MULTI_MODULE_TAG: &str = "module fmt";
const EXIST_MULTI_ADDRESS_TAG: &str = "address fmt";

const MAX_ANALYZE_LENGTH: usize = 64;
const MIN_BREAK_LENGTH: usize = 32;
const MIN_NESTED_LENGTH: usize = 16;

pub struct FormatContext {
    pub content: String,
    pub pre_simple_token: TokenTree,
    pub cur_nested_kind: NestKind,
    pub cur_fun_key_word_pos: usize,
}

impl FormatContext {
    pub fn new(content: String) -> Self {
        FormatContext {
            content,
            pre_simple_token: TokenTree::default(),
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
            cur_fun_key_word_pos: 0,
        }
    }
}

pub struct Format {
    pub(crate) local_cfg: FormatConfig,
    pub(crate) global_cfg: Config,
    pub(crate) depth: Cell<usize>,
    pub(crate) token_tree: Vec<TokenTree>,
    pub(crate) comments: Vec<Comment>,
    pub(crate) line_mapping: FileLineMappingOneFile,
    pub(crate) comments_index: Cell<usize>,
    pub(crate) ret: RefCell<String>,
    pub(crate) cur_line: Cell<u32>,
    pub(crate) format_context: RefCell<FormatContext>,
    pub(crate) syntax_handler: SyntaxHandler,
}

#[derive(Clone, Default)]
pub struct FormatConfig {
    pub(crate) indent_size: usize,
    pub(crate) max_len_no_add_line: f32,
}

const BIN_OPS: [Tok; 22] = [
    Tok::Equal,
    Tok::EqualEqual,
    Tok::ExclaimEqual,
    Tok::Less,
    Tok::Greater,
    Tok::LessEqual,
    Tok::GreaterEqual,
    Tok::PipePipe,
    Tok::AmpAmp,
    Tok::Caret,
    Tok::Pipe,
    Tok::Amp,
    Tok::LessLess,
    Tok::GreaterGreater,
    Tok::Plus,
    Tok::Minus,
    Tok::Star,
    Tok::Slash,
    Tok::Percent,
    Tok::PeriodPeriod,
    Tok::EqualEqualGreater,
    Tok::LessEqualEqualGreater,
];

const STMT_START_TOKS: [Tok; 23] = [
    Tok::Friend,
    Tok::Const,
    Tok::Fun,
    Tok::While,
    Tok::Use,
    Tok::Struct,
    Tok::Spec,
    Tok::Return,
    Tok::Public,
    Tok::Native,
    Tok::Inline,
    Tok::Move,
    Tok::Module,
    Tok::Loop,
    Tok::Let,
    Tok::Invariant,
    Tok::If,
    Tok::Continue,
    Tok::Break,
    Tok::NumSign,
    Tok::Amp,
    Tok::LParen,
    Tok::Abort,
];

static SPEC_STR: LazyLock<String> = LazyLock::new(|| Tok::Spec.to_string());
static COMMA_STR: LazyLock<String> = LazyLock::new(|| Tok::Comma.to_string());
static FUN_STR: LazyLock<String> = LazyLock::new(|| Tok::Fun.to_string());
static PUBLIC_STR: LazyLock<String> = LazyLock::new(|| Tok::Public.to_string());
static RPAREN_STR: LazyLock<String> = LazyLock::new(|| Tok::RParen.to_string());
static SEMICOLON_STR: LazyLock<String> = LazyLock::new(|| Tok::Semicolon.to_string());

fn is_bin_op(tok: Tok) -> bool {
    BIN_OPS.contains(&tok)
}

fn token_to_ability(token: Tok, content: &str) -> Option<Ability_> {
    match (token, content) {
        (Tok::Copy, _) => Some(Ability_::Copy),
        (Tok::Identifier, Ability_::DROP) => Some(Ability_::Drop),
        (Tok::Identifier, Ability_::STORE) => Some(Ability_::Store),
        (Tok::Identifier, Ability_::KEY) => Some(Ability_::Key),
        _ => None,
    }
}

fn tune_module_buf(mut module_body: String, config: &Config) -> String {
    big_block_fmt::fmt_big_block(&mut module_body);
    
    if module_body.contains(&*SPEC_STR) {
        module_body = spec_fmt::fmt_spec(module_body, config.clone());
    }

    remove_trailing_whitespaces(module_body)
}

impl Format {
    fn new(global_cfg: Config, content: &str, format_context: FormatContext) -> Self {
        let ce: CommentExtrator = CommentExtrator::new(content).unwrap();
        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(content);
        Self {
            comments_index: Default::default(),
            local_cfg: FormatConfig {
                indent_size: global_cfg.indent_size(),
                max_len_no_add_line: global_cfg.max_width() as f32 * 0.75,
            },
            global_cfg,
            depth: Default::default(),
            token_tree: vec![],
            comments: ce.comments,
            line_mapping,
            ret: Default::default(),
            cur_line: Default::default(),
            format_context: format_context.into(),
            syntax_handler: SyntaxHandler::new(content),
        }
    }

    fn generate_token_tree(&mut self, content: &str) -> Result<String, Diagnostics> {
        let (defs, _) = parse_file_string(&mut get_compile_env(), FileHash::empty(), content)?;
        let lexer = Lexer::new(content, FileHash::empty());
        let parse = crate::core::token_tree::Parser::new(lexer, &defs, content.to_string());
        self.token_tree = parse.parse_tokens();

        let defs = Arc::new(defs);
        self.syntax_handler.preprocess(&defs);
        Ok("parse ok".to_string())
    }

    pub fn format_token_trees(mut self) -> String {
        let mut pound_sign_idx = None;
        for (index, t) in self.token_tree.clone().into_iter().enumerate() {
            if t.is_pound() {
                pound_sign_idx = Some(index);
            }
            let new_line = pound_sign_idx.map_or(false, |x| (x + 1) == index);

            let mut fmt_operator = || {
                self.format_token_trees_internal(&t, self.token_tree.get(index + 1), new_line);
                if new_line {
                    self.new_line(Some(t.end_pos()));
                    pound_sign_idx = None;
                }
            };

            let mut return_buf_cp = self.ret.clone().into_inner();
            let TokenTree::Nested {
                kind: nkind, note, ..
            } = t
            else {
                fmt_operator();
                continue;
            };
            let skip_handler = self.syntax_handler.handler_immut::<SkipHandler>();
            let is_mod_blk = skip_handler.is_module_block(&nkind);
            let is_addr_blk = note.map_or(false, |x| x == Note::ModuleAddress);
            if is_mod_blk {
                *self.ret.borrow_mut() = EXIST_MULTI_MODULE_TAG.to_string();
            }
            if is_addr_blk {
                *self.ret.borrow_mut() = EXIST_MULTI_ADDRESS_TAG.to_string();
            }

            fmt_operator();

            let cfg = self.global_cfg.clone();
            // top level
            if is_mod_blk {
                self.new_line(Some(t.end_pos()));
                if !skip_handler.has_skipped_module_body(&nkind) {
                    let mut ret_borrowed = self.ret.borrow_mut();
                    let current_content = std::mem::take(&mut *ret_borrowed);
                    *ret_borrowed = update_last_line(tune_module_buf(current_content, &cfg));
                }
                let module_body_buf = self.ret.borrow().clone();
                return_buf_cp.push_str(&module_body_buf[EXIST_MULTI_MODULE_TAG.len()..]);
                *self.ret.borrow_mut() = return_buf_cp;
            } else if is_addr_blk {
                self.new_line(Some(t.end_pos()));
                let mut fmt_buf = self.ret.borrow_mut();
                let def_vec_result =
                    parse_file_string(&mut get_compile_env(), FileHash::empty(), &*fmt_buf);
                let def_vec = def_vec_result.unwrap_or_default().0;

                let mut last_mod_end_loc = 0;
                let mut fmt_slice = String::new();
                let Some(Definition::Address(address_def)) = def_vec.first() else {
                    return_buf_cp.push_str(&fmt_buf[EXIST_MULTI_ADDRESS_TAG.len()..]);
                    *fmt_buf = return_buf_cp.clone();
                    continue;
                };
                for mod_def in &address_def.modules {
                    let m = &fmt_buf[mod_def.loc.start() as usize..mod_def.loc.end() as usize];
                    let tuning_mod_body = tune_module_buf(m.to_string(), &cfg);
                    fmt_slice.push_str(&fmt_buf[last_mod_end_loc..mod_def.loc.start() as usize]);
                    fmt_slice.push_str(&tuning_mod_body);
                    last_mod_end_loc = mod_def.loc.end() as usize;
                }

                fmt_slice.push_str(&fmt_buf[last_mod_end_loc..fmt_buf.len()]);

                tracing::debug!("return_buf_cp = {:?}", return_buf_cp);
                tracing::debug!("fmt_slice = {:?}", fmt_slice);
                return_buf_cp.push_str(&fmt_slice[EXIST_MULTI_ADDRESS_TAG.len()..]);
                *fmt_buf = return_buf_cp.clone();
            } else if nkind.kind == NestKind_::Brace {
                self.new_line(Some(t.end_pos()));
                tracing::debug!("<script> return_buf_cp = {:?}", return_buf_cp);
                tracing::debug!("<script> self.ret = {:?}", &self.ret);
                let mut ret_borrowed = self.ret.borrow_mut();
                let current_content = std::mem::take(&mut *ret_borrowed);
                *ret_borrowed = update_last_line(tune_module_buf(current_content, &cfg));
            }
        }
        self.add_comments(u32::MAX, "end_of_move_file".to_string());
        self.remove_trailing_whitespaces();
        self.process_last_empty_line();
        self.ret.into_inner()
    }

    fn is_long_nested_token(current: &TokenTree) -> (bool, usize) {
        if let TokenTree::Nested { elements, kind, .. } = current {
            return (
                matches!(kind.kind, NestKind_::Brace | NestKind_::ParentTheses)
                    && analyze_token_tree_length(elements, MAX_ANALYZE_LENGTH) > MIN_BREAK_LENGTH,
                elements.len(),
            );
        }
        (false, 0)
    }

    fn check_next_tok_canbe_break(next: Option<&TokenTree>) -> bool {
        if let Some((next_tok, next_content)) = next.map(|x| match x {
            TokenTree::SimpleToken {
                content,
                pos: _,
                tok,
                ..
            } => (*tok, content.clone()),
            TokenTree::Nested { kind, .. } => {
                (kind.kind.start_tok(), kind.kind.start_tok().to_string())
            }
        }) {
            if STMT_START_TOKS.contains(&next_tok) {
                true
            } else if next_tok == Tok::Identifier {
                next_content.as_str() == "entry"
            } else {
                false
            }
        } else {
            true
        }
    }

    fn check_cur_token_is_long_bin_op(
        &self,
        current: &TokenTree,
        next: Option<&TokenTree>,
        next_tok: Tok,
        index: usize,
        kind: &NestKind,
        elements: &[TokenTree],
    ) -> bool {
        let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
        if matches!(
            current.get_start_tok(),
            Tok::EqualEqualGreater | Tok::LessEqualEqualGreater
        ) && let_handler.is_long_bin_op(current.clone())
        {
            return true;
        }

        let judge_equal_tok_is_long_op_fn = || {
            let_handler.is_long_assign(
                current.clone(),
                next.clone(),
                self.global_cfg.clone(),
                self.last_line().len() + 2,
            )
        };

        // updated in 20240607: fix https://github.com/movebit/movefmt/issues/7
        if current.get_start_tok() == Tok::Equal
            && next.unwrap().simple_str().unwrap_or_default() != "vector"
            && next_tok != Tok::LBrace
        {
            let call_handler = self.syntax_handler.handler_immut::<CallHandler>();
            if call_handler.component_is_complex_blk(
                self.global_cfg.clone(),
                kind,
                elements,
                index as i64,
                self.last_line().len(),
            ) != ComplexCallKind::Pack
            {
                return judge_equal_tok_is_long_op_fn();
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
        let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
        let bin_op_handler = self.syntax_handler.handler_immut::<BinOpHandler>();
        if matches!(next_token, Tok::AmpAmp | Tok::PipePipe)
            && let_handler.is_long_bin_op(next_t.unwrap().clone())
        {
            return true;
        }
        if matches!(
            next_token,
            Tok::EqualEqualGreater | Tok::LessEqualEqualGreater | Tok::Equal
        ) {
            return false;
        }
        let current_token_len =
            analyze_token_tree_length(&[current.clone()], self.global_cfg.max_width());
        let len_plus_cur_token = self.last_line().len() + current_token_len + 2;
        if len_plus_cur_token > self.global_cfg.max_width() {
            return false;
        }

        if let TokenTree::Nested { elements, .. } = current {
            let delimiter = analyze_token_tree_delimiter(elements).0;
            let cur_nested_break_mode = self.get_break_mode_begin_nested(current, delimiter);
            if cur_nested_break_mode.0 || cur_nested_break_mode.1 == Some(true) {
                return false;
            }
            for nested_nested_in_current_tree in elements {
                if let TokenTree::Nested {
                    elements: _ele,
                    kind: tmp_kind,
                    ..
                } = nested_nested_in_current_tree
                    && nested_nested_in_current_tree.token_len() as usize > MIN_BREAK_LENGTH
                    && tmp_kind.kind == NestKind_::Brace
                {
                    return false;
                }
            }
        };

        if is_bin_op(next_token) {
            let r_exp_len_tuple = bin_op_handler.get_bin_op_right_part_len(next_t.unwrap().clone());
            if r_exp_len_tuple.0 == 0 && r_exp_len_tuple.1 < 8 {
                return false;
            }
            tracing::trace!(
                "self.last_line().len() = {:?}, r_exp_len_tuple = {:?}",
                self.last_line().len(),
                r_exp_len_tuple
            );
            let len_bin_op_full = len_plus_cur_token
                + 2
                + next_t.unwrap().simple_str().unwrap_or_default().len()
                + r_exp_len_tuple.1;
            if len_bin_op_full >= self.global_cfg.max_width() {
                bin_op_handler.record_long_op(r_exp_len_tuple.0);
                return true;
            }
        }
        false
    }

    fn check_next_token_is_quant_body(
        &self,
        current: &TokenTree,
        next_t: Option<&TokenTree>,
    ) -> bool {
        let quant_handler = self.syntax_handler.handler_immut::<QuantHandler>();
        if current.get_end_tok() == Tok::Colon {
            let (quant_exp_idx, quant_body_len) =
                quant_handler.get_quant_body_len(next_t.unwrap().clone());
            if quant_body_len < 8 {
                return false;
            }

            let len_plus_cur_token = self.last_line().len() + current.token_len() as usize + 2;
            if len_plus_cur_token > self.global_cfg.max_width() {
                return false;
            }
            if len_plus_cur_token + quant_body_len > self.global_cfg.max_width() {
                quant_handler.record_long_quant_exp(quant_exp_idx);
                return true;
            }
        }
        false
    }

    fn check_new_line_mode_for_cur_tok(
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

        let b_judge_next_token = Self::check_next_tok_canbe_break(next);

        // special case for `}}`
        if let TokenTree::Nested { kind, .. } = current
            && kind.kind == NestKind_::Brace
            && kind_outer.kind == NestKind_::Brace
            && b_judge_next_token
        {
            return true;
        }

        // added in 20240426: special case for current is long nested type
        if matches!(kind_outer.kind, NestKind_::Brace | NestKind_::ParentTheses) {
            let result_inner = Self::is_long_nested_token(current);
            if b_judge_next_token && result_inner.0 && result_inner.1 > 4 {
                return true;
            }
        }

        // added in 20240911
        // special case: there are no comma between ENUM fields
        if current.get_end_tok() == Tok::RBrace
            && next.is_some()
            && next.unwrap().get_start_tok() == Tok::Identifier
            && !matches!(next.unwrap().simple_str().unwrap_or_default(), "to" | "for")
        {
            return true;
        }
        false
    }

    fn get_new_line_mode_for_cur_tok(
        &self,
        kind_outer: &NestKind,
        current: &TokenTree,
        next: Option<&TokenTree>,
    ) -> bool {
        if kind_outer.end_pos - current.end_pos() < MIN_NESTED_LENGTH.try_into().unwrap() {
            return false;
        }
        let b_judge_next_token = next.is_some() && Self::check_next_tok_canbe_break(next);
        if matches!(kind_outer.kind, NestKind_::Brace | NestKind_::ParentTheses)
            && b_judge_next_token
            && Self::is_long_nested_token(current).0
        {
            return true;
        }
        false
    }

    fn need_new_line_after_cur_tok_finished(
        &self,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        index: usize,
        component_break_mode: bool,
        nested_kind_len: usize,
    ) -> bool {
        let TokenTree::Nested { elements, kind, .. } = nested_token else {
            return false;
        };

        fn token_tree_start(tt: &TokenTree) -> (Tok, String) {
            match tt {
                TokenTree::SimpleToken { tok, content, .. } => (*tok, content.clone()),
                TokenTree::Nested { kind, .. } => {
                    (kind.kind.start_tok(), kind.kind.start_tok().to_string())
                }
            }
        }

        let t = elements.get(index).unwrap();
        let next_t = elements.get(index + 1);
        let d = delimiter.map(Delimiter::to_static_str);
        let t_str = t.simple_str();
        let is_comma = d == Some(&COMMA_STR);
        let cur_is_delimiter = d == t_str;
        let (next_tok, next_content) = next_t
            .map(token_tree_start)
            .unwrap_or((Tok::EOF, String::new()));

        // comma in fun resource access specifier not change new line
        if is_comma
            && elements[..index]
                .iter()
                .rev()
                .take_while(|ele| ele.simple_str() != Some(&FUN_STR))
                .any(|ele| {
                    matches!(
                        ele.simple_str(),
                        Some("acquires" | "reads" | "writes" | "pure")
                    )
                })
        {
            return false;
        }

        // ablility not change new line
        // optimize in 20240510: maybe like variable name or struct field name are ability, like "key"
        // fixed bug in 20240718: you can see case [tests/bug/input4.move]
        if cur_is_delimiter
            && is_comma
            && self
                .format_context
                .borrow()
                .pre_simple_token
                .simple_str()
                .and_then(|s| token_to_ability(self.get_pre_simple_tok(), &s))
                .is_some()
            && token_to_ability(next_tok, &next_content).is_some()
        {
            return false;
        }

        let mut new_line = if component_break_mode {
            self.check_new_line_mode_for_cur_tok(kind, delimiter, has_colon, t, next_t)
                || (cur_is_delimiter && d.is_some() && kind.kind != NestKind_::Type)
        } else {
            self.get_new_line_mode_for_cur_tok(kind, t, next_t)
        };

        if nested_kind_len > MIN_NESTED_LENGTH && kind.kind != NestKind_::Type {
            new_line |=
                self.check_cur_token_is_long_bin_op(t, next_t, next_tok, index, kind, &elements);
            if !new_line && next_t.is_some() {
                if self.check_next_token_is_long_bin_op(t, next_t, next_tok) {
                    return true;
                }
                if self.check_next_token_is_quant_body(t, next_t) {
                    return true;
                }
            }
        }
        new_line
    }

    fn process_fn_header(&self) {
        let mut ret = self.ret.borrow_mut();
        let cur = ret.as_str();
        let last_fun_idx = self.format_context.borrow().cur_fun_key_word_pos;
        if last_fun_idx >= cur.len() {
            return;
        }
        let indent = " ".repeat((self.depth.get() + 1) * self.local_cfg.indent_size);
        let fun_specifier_fmted_str =
            fun_fmt::fun_header_specifier_fmt(&cur[last_fun_idx..], &indent);

        *ret = format!("{}{}", &cur[..last_fun_idx], fun_specifier_fmted_str);
    }

    fn get_break_mode_of_fun_call(
        &self,
        token: &TokenTree,
        nested_token_len: usize,
        opt_component_break_mode: &mut bool,
    ) -> bool {
        let TokenTree::Nested { elements, kind, .. } = token else {
            return false;
        };
        let call_handler = self.syntax_handler.handler_immut::<CallHandler>();
        if call_handler.need_split_call_component(
            self.global_cfg.clone(),
            kind,
            &elements,
            nested_token_len,
            self.last_line().len(),
        ) {
            let next_line_len = " "
                .to_string()
                .repeat((self.depth.get() + 1) * self.local_cfg.indent_size)
                .len();

            let (nested_dep, comma_cnt) = expr_fmt::get_nested_and_comma_num(elements);
            if comma_cnt > 2 || nested_dep > 2 {
                if self.global_cfg.prefer_one_line_for_short_call_para_list() {
                    *opt_component_break_mode =
                        nested_dep > 2 || nested_token_len > MIN_BREAK_LENGTH;
                } else {
                    *opt_component_break_mode = true;
                }
            } else if next_line_len + nested_token_len > self.global_cfg.max_width()
                || nested_token_len > MAX_ANALYZE_LENGTH
            {
                *opt_component_break_mode = true;
            }
            return true;
        }
        false
    }

    fn get_break_mode_begin_paren(&self, token: &TokenTree) -> (bool, Option<bool>) {
        let TokenTree::Nested { elements, kind, .. } = token else {
            return (false, None);
        };
        if NestKind_::ParentTheses != kind.kind {
            return (false, None);
        }

        let first_ele_is_nested = elements[0].simple_str().is_none();
        if elements.len() == 1 && first_ele_is_nested {
            return (false, None);
        }

        let nested_token_len = self.get_kind_len_after_trim_space(kind);
        let mut opt_component_break_mode = nested_token_len
            + (self.depth.get() + 1) * self.local_cfg.indent_size
            >= self.global_cfg.max_width();
        if matches!(self.get_pre_simple_tok(), Tok::If | Tok::While) {
            return (false, Some(opt_component_break_mode));
        }

        let (is_in_fun_header, fun_len) = self
            .syntax_handler
            .handler_immut::<FunHandler>()
            .is_parameter_paren_in_fun_header(kind);

        if !is_in_fun_header && self.last_line().len() > self.global_cfg.max_width() {
            return (true, Some(opt_component_break_mode));
        }
        if self
            .syntax_handler
            .handler_immut::<CallHandler>()
            .paren_in_call(kind)
        {
            return (
                self.get_break_mode_of_fun_call(
                    token,
                    nested_token_len,
                    &mut opt_component_break_mode,
                ),
                Some(opt_component_break_mode),
            );
        }

        let paren_str =
            &self.format_context.borrow().content[kind.start_pos as usize..kind.end_pos as usize];
        if contains_comment(&paren_str) && paren_str.find("//").is_some() {
            return (true, Some(opt_component_break_mode));
        }

        let cur_line_status = get_code_buf_len(self.last_line());
        let cur_line_len = if cur_line_status.1 {
            cur_line_status.0
        } else {
            self.last_line().len()
        };

        let (nested_dep, comma_cnt) = expr_fmt::get_nested_and_comma_num(elements);
        if is_in_fun_header
            && !self
                .global_cfg
                .prefer_one_line_for_short_fn_header_para_list()
        {
            opt_component_break_mode |= comma_cnt > 1;
        }
        opt_component_break_mode |= (nested_dep >= 4 || comma_cnt > 2)
            && nested_token_len as f32 > self.local_cfg.max_len_no_add_line;

        let mut new_line_mode = fun_len > self.global_cfg.max_width();
        // Reserve 25% space for return ty and specifier
        new_line_mode |= is_in_fun_header && cur_line_len + nested_token_len > MAX_ANALYZE_LENGTH;
        new_line_mode |= opt_component_break_mode && comma_cnt > 2;
        if !is_in_fun_header && !new_line_mode {
            if !first_ele_is_nested {
                new_line_mode |= cur_line_len + nested_token_len > self.global_cfg.max_width()
                    && nested_token_len > 8;
            } else {
                let first_ele_len =
                    analyze_token_tree_length(&[elements[0].clone()], self.global_cfg.max_width());
                new_line_mode |=
                    cur_line_len + first_ele_len > self.global_cfg.max_width() && first_ele_len > 8;
            }
            new_line_mode |= comma_cnt > 2 && nested_token_len > MIN_BREAK_LENGTH;
            new_line_mode |= nested_dep > 2 && nested_token_len > MAX_ANALYZE_LENGTH;
            new_line_mode |= opt_component_break_mode && comma_cnt > 0;
        }
        return (new_line_mode, Some(opt_component_break_mode));
    }

    fn get_control_blk_cnt(&self, elements: &[TokenTree]) -> usize {
        let mut control_blk_cnt = 0;
        for ele in elements {
            if matches!(
                ele.get_start_tok(),
                Tok::If | Tok::Else | Tok::Loop | Tok::While
            ) {
                control_blk_cnt += 1;
            }
        }
        control_blk_cnt
    }

    fn get_break_mode_begin_branch_blk(&self, kind: &NestKind) -> bool {
        let branch_handler = self.syntax_handler.handler_immut::<BranchHandler>();
        if branch_handler
            .com_if_else
            .then_loc_vec
            .iter()
            .any(|&x| x.start() == kind.start_pos)
            || branch_handler
                .com_if_else
                .else_loc_vec
                .iter()
                .any(|&x| x.start() == kind.start_pos)
        {
            if self.global_cfg.prefer_one_line_for_short_branch_blk() {
                return self.get_kind_len_after_trim_space(kind) > 8;
            } else {
                return true;
            }
        }
        false
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
        let max_len_no_add_line = self.local_cfg.max_len_no_add_line;
        let max_line_width = self.global_cfg.max_width();
        let nested_blk_str =
            &self.format_context.borrow().content[kind.start_pos as usize..kind.end_pos as usize];
        let nested_len = self.get_kind_len_after_trim_space(kind);
        if elements.is_empty() {
            let should_break = nested_len as f32 > max_len_no_add_line
                || (contains_comment(nested_blk_str) && nested_blk_str.lines().count() > 1);
            return (should_break, None);
        }

        // 20240329 updated
        // fun body brace always change new line;
        // if ParentTheses is empty, not change new line;
        // 20240425 updated
        // The value of new_line_mode here is not associated with Paren, only with Brace.
        // Because Paren may come from fn_para or call or expression statements...
        let is_stct_def = *note == Some(Note::StructDefinition);
        let is_fun_body = *note == Some(Note::FunBody);
        if (delimiter == Some(Delimiter::Semicolon) || is_stct_def || is_fun_body)
            && kind.kind != NestKind_::Type
        {
            return if is_stct_def {
                (true, Some(true))
            } else {
                (true, None)
            };
        }

        let mut new_line_mode = false;
        match kind.kind {
            NestKind_::Type => {
                // added in 20240112: if type in fun header, not change new line
                if self
                    .syntax_handler
                    .handler_immut::<FunHandler>()
                    .is_generic_ty_in_fun_header(kind)
                {
                    return (false, None);
                }

                let first_ele_len =
                    analyze_token_tree_length(&[elements[0].clone()], max_line_width);
                new_line_mode =
                    self.last_line().len() + first_ele_len > max_line_width && first_ele_len > 8;
            }
            NestKind_::ParentTheses => return self.get_break_mode_begin_paren(token),
            NestKind_::Bracket => {
                let is_annotation = self.get_pre_simple_tok() == Tok::NumSign;
                new_line_mode = (is_annotation && nested_len > max_line_width)
                    || (!is_annotation && nested_len > MAX_ANALYZE_LENGTH);
                if elements.len() > MIN_BREAK_LENGTH {
                    let mut bin_op_cnt = 0;
                    let mut complex_ele_cnt = 0;
                    for ele in elements {
                        let result_inner = Self::is_long_nested_token(ele);
                        if result_inner.0 && result_inner.1 > 4 {
                            complex_ele_cnt += 1;
                        }
                        if is_bin_op(ele.get_start_tok()) {
                            bin_op_cnt += 1;
                        }
                    }
                    return (new_line_mode, Some(complex_ele_cnt > 4 || bin_op_cnt > 4));
                }
            }
            NestKind_::Lambda => {
                if nested_len < MIN_BREAK_LENGTH {
                    return (false, None);
                }
                new_line_mode |= self.last_line().len() + nested_len > MAX_ANALYZE_LENGTH;

                let nested_and_comma_pair = expr_fmt::get_nested_and_comma_num(elements);
                let opt_component_break_mode =
                    if self.global_cfg.prefer_one_line_for_short_lambda_para_list() {
                        (nested_and_comma_pair.0 >= 4 || nested_and_comma_pair.1 > 2)
                            && token.token_len() as f32 > max_len_no_add_line
                    } else {
                        nested_and_comma_pair.1 > 1
                    };

                new_line_mode |= opt_component_break_mode;
            }
            NestKind_::Brace => {
                if nested_len > 4 {
                    // case1: over max width
                    new_line_mode |= self.last_line().len() + nested_len > max_line_width;

                    // case2: has special keyword
                    new_line_mode |= has_special_key(self.last_line());
                }

                // case3: nested_len too long
                new_line_mode |= nested_len as f32 > 46.0;

                // case4: contains comment
                new_line_mode |=
                    contains_comment(nested_blk_str) && nested_blk_str.lines().count() > 1;

                // case5: has too much nested blks
                let (nested_cnt, _) = expr_fmt::get_nested_and_comma_num(elements);
                new_line_mode |= nested_cnt >= 2 && nested_len > MIN_BREAK_LENGTH;

                // case6: has too much control blks
                new_line_mode |= self.get_control_blk_cnt(elements) >= 2;

                // case7: maybe in branch blk
                new_line_mode |= self.get_break_mode_begin_branch_blk(&kind);
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
        b_add_space_at_bound: bool,
    ) {
        // step1 -- format start_token
        self.format_token_trees_internal(&kind.start_token_tree(), None, b_new_line_mode);

        // step2 -- paired effect with step6
        if b_add_indent && b_new_line_mode {
            self.inc_depth();
        }
        if b_new_line_mode && !elements.is_empty() {
            tracing::debug!(
                "top_half_after_kind_start -- add a new line before {:?}; b_add_indent = {:?}",
                elements.first().unwrap().simple_str(),
                b_add_indent
            );
        }

        // step3
        if b_new_line_mode {
            self.add_new_line_after_nested_begin(kind, elements, b_new_line_mode);
        } else if b_add_space_at_bound {
            self.push_str(" ");
        }
    }

    fn bottom_half_before_kind_end(
        &self,
        kind: &NestKind,
        b_new_line_mode: bool,
        b_add_indent: bool,
        b_add_space_at_bound: bool,
        nested_token_head: Tok,
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
        if b_add_indent && b_new_line_mode {
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
            if nested_token_head == Tok::If || kind.kind == NestKind_::Type {
                // 20240426 -- for [] and <>  don't add new line
                // 20240801 -- for <>  don't add new line
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
        } else if b_add_space_at_bound {
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

    fn format_single_token(&self, nested_token: &TokenTree, token_idx: usize, new_line: bool) {
        let TokenTree::Nested { elements, .. } = nested_token else {
            return;
        };
        let token = elements.get(token_idx).unwrap();
        let next_t = elements.get(token_idx + 1);

        let pre_tok_is_num_sign = Tok::NumSign == self.get_pre_simple_tok();
        self.format_token_trees_internal(token, next_t, pre_tok_is_num_sign || new_line);

        if pre_tok_is_num_sign {
            tracing::debug!("in loop<TokenTree::Nested> pre_tok_is_num_sign = true");
            self.new_line(Some(token.end_pos()));
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

    fn format_dot_exp_chain(
        &self,
        elements: &[TokenTree],
        idx: &mut usize,
        nested_token: &TokenTree,
    ) -> bool {
        let chain_result = expr_fmt::parse_dot_chain(&elements.split_at(*idx).1);
        debug!("chain_result = {:?}", chain_result);

        let (members, last_dot_idx) = chain_result.unwrap_or_default();
        let new_idx = *idx + last_dot_idx;
        debug!("new_idx = {}, last_dot_idx = {}", new_idx, last_dot_idx);

        let dist = elements[new_idx].end_pos() - elements[*idx].start_pos();
        let b_process_link =
            members.len() > 3 && new_idx > *idx && dist as usize > MIN_BREAK_LENGTH;
        if !b_process_link {
            while *idx < new_idx {
                self.format_single_token(nested_token, *idx, false);
                *idx += 1;
            }
            return false;
        }
        debug!("before process_link, last_line = {}", self.last_line());
        self.inc_depth();
        while *idx <= new_idx {
            let next_is_dot = elements
                .get(*idx + 1)
                .map_or(false, |t| t.get_start_tok() == Tok::Period);

            self.format_single_token(nested_token, *idx, next_is_dot);
            *idx += 1;
        }
        self.dec_depth();

        true
    }

    fn format_nested_elements(
        &self,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        component_break_mode: bool,
    ) {
        let TokenTree::Nested { elements, kind, .. } = nested_token else {
            return;
        };
        let call_handler = self.syntax_handler.handler_immut::<CallHandler>();
        let nestd_kind_len = self.get_kind_len_after_trim_space(kind);
        let old_kind = self.format_context.borrow_mut().cur_nested_kind;
        self.format_context.borrow_mut().cur_nested_kind = *kind;
        let nested_ele_len = elements.len();
        let mut token_idx = 0;

        let is_call = kind.kind == NestKind_::ParentTheses && call_handler.paren_in_call(kind);
        let mut need_get_break_mode_on_component = component_break_mode;
        if nested_ele_len > MIN_BREAK_LENGTH
            && kind.kind == NestKind_::Bracket
            && !component_break_mode
        {
            need_get_break_mode_on_component = false;
        }
        let last_is_comma = elements
            .last()
            .map_or(false, |t| t.get_start_tok() == Tok::Comma);
        while token_idx < nested_ele_len {
            let mut new_line = self.need_new_line_after_cur_tok_finished(
                nested_token,
                delimiter,
                has_colon,
                token_idx,
                need_get_break_mode_on_component,
                nestd_kind_len,
            );
            if is_call {
                new_line |= component_break_mode
                    && call_handler.should_call_component_split(
                        self.global_cfg.clone(),
                        kind,
                        elements,
                        token_idx,
                        self.last_line().len(),
                    );
            }

            if token_idx == nested_ele_len - 1 && last_is_comma {
                break;
            }

            if Tok::Period == self.get_pre_simple_tok()
                && self.format_dot_exp_chain(elements, &mut token_idx, nested_token)
            {
                continue;
            }

            self.format_single_token(&nested_token, token_idx, new_line);
            token_idx += 1;
        }

        self.format_context.borrow_mut().cur_nested_kind = old_kind;
    }

    fn need_space_at_bound(&self, nested_token: &TokenTree, b_new_line_mode: bool) -> bool {
        let TokenTree::Nested { elements, kind, .. } = nested_token else {
            return true;
        };
        if b_new_line_mode {
            return false;
        }
        // let mut add_space;
        if elements.is_empty() {
            let nested_blk_str = &self.format_context.borrow().content
                [kind.start_pos as usize + 1..kind.end_pos as usize];
            contains_comment(nested_blk_str)
        } else {
            match kind.kind {
                NestKind_::Brace => {
                    // optimize in 20240425
                    // there are 2 cases which not add space
                    // eg1: When braces are used for arithmetic operations
                    // let intermediate3: u64 = (a * {c + d}) - (b / {e - 2});
                    // shouldn't formated like `let intermediate3: u64 = (a * { c + d }) - (b / { e - 2 });`
                    // eg2: When the braces are used for use
                    // use A::B::{C, D}
                    // shouldn't formated like `use A::B::{ C, D }`
                    let nested_token_head = self.get_pre_simple_tok();
                    let is_arithmetic_op = matches!(
                        nested_token_head,
                        Tok::Plus | Tok::Minus | Tok::Star | Tok::Slash | Tok::Percent
                    );
                    let b_not_use_brace = Tok::ColonColon != nested_token_head;
                    !is_arithmetic_op && b_not_use_brace && !elements.is_empty()
                }
                NestKind_::Lambda => {
                    matches!(elements[0].get_start_tok(), Tok::Pipe | Tok::PipePipe)
                }
                _ => false,
            }
        }
    }

    fn need_skip_nested_token(&self, kind: &NestKind, note: &Option<Note>) -> bool {
        let block_body_ty = match note.unwrap_or_default() {
            Note::StructDefinition => SkipType::SkipStructBody,
            Note::FunBody => SkipType::SkipFunBody,
            Note::ModuleDef => SkipType::SkipModuleBody,
            _ => SkipType::SkipNone,
        };
        if self
            .syntax_handler
            .handler_immut::<SkipHandler>()
            .should_skip_block_body(kind, block_body_ty)
        {
            let blk_body_str = &self.format_context.borrow().content
                [kind.start_pos as usize..kind.end_pos as usize + 1];
            debug!("should_skip_block_body = {:?}", blk_body_str);
            self.push_str(blk_body_str);

            for c in &self.comments[self.comments_index.get()..] {
                if c.start_offset > kind.end_pos {
                    break;
                }
                self.comments_index.set(self.comments_index.get() + 1);
            }
            self.cur_line.set(self.translate_line(kind.end_pos));
            return true;
        }
        false
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
        if self.need_skip_nested_token(&kind, note) {
            return;
        }

        let (delimiter, has_colon) = analyze_token_tree_delimiter(elements);
        if note.map_or(false, |x| x == Note::FunBody) {
            self.process_fn_header();
        }
        let (mut b_new_line_mode, opt_component_break_mode) =
            self.get_break_mode_begin_nested(nested_token, delimiter);

        let mut b_add_indent = true;
        for i in 0..elements.len() {
            let ele_str = elements[i].simple_str().unwrap_or_default();
            if !matches!(ele_str, "#" | "" | "module") || i > MIN_NESTED_LENGTH {
                break;
            }
            if elements[i].simple_str().unwrap_or_default() == "module" {
                b_add_indent = false;
                b_new_line_mode |= true;
                break;
            }
        }

        let nested_token_head = self.get_pre_simple_tok();
        let b_add_space_at_bound = self.need_space_at_bound(nested_token, b_new_line_mode);

        // step1-step3
        self.top_half_after_kind_start(
            kind,
            elements,
            b_new_line_mode,
            b_add_indent,
            b_add_space_at_bound,
        );

        // step4 -- format element
        self.format_nested_elements(
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
            b_add_space_at_bound,
            nested_token_head,
        );

        // step8 -- format end_token
        self.format_token_trees_internal(&kind.end_token_tree(), None, false);
        if expr_fmt::need_space(nested_token, next_token) {
            if nested_token_head == Tok::NumSign && kind.kind == NestKind_::Bracket {
                return;
            }
            self.push_str(" ");
        }
    }

    fn maybe_begin_of_if_else(&self, token: &TokenTree, next_token: Option<&TokenTree>) {
        // updated in 20240517: add condition `NestKind_::Bracket`
        if self.format_context.borrow().cur_nested_kind.kind == NestKind_::Bracket {
            return;
        }
        let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        else {
            return;
        };

        let branch_handler = self.syntax_handler.handler_immut::<BranchHandler>();
        // optimize in 20241212
        let pre_tok = self.get_pre_simple_tok();
        if !matches!(pre_tok, Tok::RParen | Tok::Else) && *tok != Tok::Else {
            return;
        }

        // added in 20240115
        // updated in 20241212: fix https://github.com/movebit/movefmt/issues/43
        let end_pos_of_if_cond_or_else = self.format_context.borrow().pre_simple_token.end_pos();
        if Tok::LBrace != *tok
            && content != "for"
            && branch_handler.need_new_line_after_branch(
                self.last_line(),
                *pos,
                self.global_cfg.clone(),
                end_pos_of_if_cond_or_else,
            )
        {
            tracing::debug!("need_new_line_after_branch[{:?}], add a new line", content);
            self.inc_depth();
            let cur_line = self.last_line();
            if cur_line.trim_start().len() == 0 {
                // maybe already added new line because of judge_cond() is a long nested expr
                self.push_str(" ".to_string().repeat(self.local_cfg.indent_size).as_str());
                return;
            }
            return self.new_line(None);
        }

        // updated in 20240516: optimize break line before else
        let mut new_line_before_else = false;
        if *tok == Tok::Else {
            if self.get_pre_simple_tok() == Tok::RBrace {
                // case1
                if get_code_buf_len(self.last_line()).1 {
                    // process case:
                    // else if() {} `insert '\n' here` else
                    new_line_before_else = true;
                }
            } else if next_token.is_some() {
                // case2
                if self.last_line().len()
                    + content.len()
                    + 2
                    + next_token.unwrap().token_len() as usize
                    > self.global_cfg.max_width() - MIN_NESTED_LENGTH
                {
                    new_line_before_else = true;
                }

                // case3
                if branch_handler.else_branch_too_long(
                    self.last_line(),
                    next_token.unwrap().start_pos() as ByteIndex,
                    self.global_cfg.clone(),
                ) {
                    new_line_before_else = true;
                }

                // case4 -- process `else if`
                let is_in_nested_else_branch = branch_handler.is_nested_within_an_outer_else(*pos);
                if next_token.unwrap().simple_str().unwrap_or_default() == "if"
                    || is_in_nested_else_branch
                {
                    new_line_before_else = true;
                }
            }
        }
        if new_line_before_else {
            self.new_line(None);
        }
    }

    fn maybe_end_of_if_else(&self, token: &TokenTree, next_token: Option<&TokenTree>) {
        // added in 20240115
        // updated in 20240124
        // updated in 20240222: remove condition `if Tok::RBrace != *tok `
        // updated in 20240517: add condition `NestKind_::Bracket`
        if let TokenTree::SimpleToken { content, pos, .. } = token
            && self.format_context.borrow().cur_nested_kind.kind != NestKind_::Bracket
        {
            let tok_end_pos = *pos + content.len() as u32;
            let mut nested_branch_depth = self
                .syntax_handler
                .handler_immut::<BranchHandler>()
                .added_new_line_after_branch(tok_end_pos);

            let mut need_add_new_line = false;
            if nested_branch_depth > 0 {
                tracing::debug!(
                    "nested_branch_depth[{:?}] = [{:?}]",
                    content,
                    nested_branch_depth
                );
                need_add_new_line = true;
            }
            while nested_branch_depth > 0 {
                self.dec_depth();
                nested_branch_depth -= 1;
            }

            if need_add_new_line
                && next_token.is_some()
                && next_token.unwrap().simple_str().unwrap_or_default() != ";"
            {
                self.new_line(None);
            }
        }
    }

    fn process_blank_lines_before_simple_token(&self, token: &TokenTree) {
        let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        else {
            return;
        };
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
            && expr_fmt::need_newline_when_trim_blank_line(&self.get_pre_simple_tok(), tok)
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
        if token.simple_str() == Some(&FUN_STR)
            && !matches!(
                self.get_pre_simple_tok(),
                Tok::Public | Tok::Identifier | Tok::RParen | Tok::Inline | Tok::Spec | Tok::Native
            )
            && !self
                .last_line()
                .clone()
                .trim_start_matches(char::is_whitespace)
                .is_empty()
        {
            self.new_line(None);
        }

        if token.simple_str() == Some(&PUBLIC_STR)
            && !matches!(
                self.get_pre_simple_tok(),
                Tok::Identifier | Tok::RParen | Tok::Native
            )
            && !self
                .last_line()
                .clone()
                .trim_start_matches(char::is_whitespace)
                .is_empty()
        {
            self.new_line(None);
        }
    }

    fn may_inc_depth_before_fun_ret_ty(&self, next_token: Option<&TokenTree>) {
        if next_token.is_none() {
            return;
        }
        let last_line_len = self.last_line().len();
        let ret_type_len = self
            .syntax_handler
            .handler_immut::<FunHandler>()
            .is_fun_return_colon(next_token.unwrap());
        if ret_type_len == 0 {
            return;
        }
        if last_line_len > MIN_BREAK_LENGTH
            && ret_type_len + last_line_len >= self.global_cfg.max_width()
        {
            self.inc_depth();
            self.new_line(None);
            self.dec_depth();
        } else if self
            .last_line()
            .clone()
            .trim_start_matches(char::is_whitespace)
            .len()
            == 0
        {
            self.indent();
        }
    }

    fn handle_split_line(&self, leading_space_cnt: usize) {
        let need_inc_depth = !matches!(
            self.format_context.borrow().cur_nested_kind.kind,
            NestKind_::Bracket | NestKind_::ParentTheses
        );
        if need_inc_depth {
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

    fn fmt_simple_token_core(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        let TokenTree::SimpleToken {
            content,
            pos,
            tok,
            note,
        } = token
        else {
            return;
        };

        let leading_space_cnt = self.get_last_line_leading_space_cnt();

        // These very long `Tok`s appear after `bin_op`:
        // "[Num]", "[NumTyped]", "[ByteString]", "[Identifier]",
        if content.len() > MAX_ANALYZE_LENGTH && self.last_line().len() < MAX_ANALYZE_LENGTH {
            let need_early_process = if self.get_pre_simple_tok() != Tok::Equal {
                true
            } else {
                let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
                // if true, means already change new line and increased depth.
                let_handler.is_long_let_assign_rhs_end(token.clone()) > 0
            };
            if need_early_process {
                self.push_str(content.as_str());
                return self.update_pos_and_space(pos, token, next_token, new_line_after);
            }
        }

        if self.judge_change_new_line_when_over_limits(content.clone(), *tok, *note, next_token) {
            self.handle_split_line(leading_space_cnt);
        } else if *tok == Tok::Colon {
            self.may_inc_depth_before_fun_ret_ty(next_token);
        }

        self.push_str(content.as_str());
        self.update_pos_and_space(pos, token, next_token, new_line_after);
    }

    fn format_simple_token(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        {
            // step1
            self.maybe_begin_of_if_else(token, next_token);

            // step2: add comment(xxx) before current simple_token
            self.add_comments(*pos, content.clone());

            // step3
            self.process_blank_lines_before_simple_token(token);

            // step4
            self.fmt_simple_token_core(token, next_token, new_line_after);

            // step5
            self.maybe_end_of_if_else(token, next_token);

            // step6
            self.format_context.borrow_mut().pre_simple_token = token.clone();
            if tok == &Tok::Fun {
                self.format_context.borrow_mut().cur_fun_key_word_pos = self.ret.borrow().len();
            }
        }
    }

    fn need_inc_depth_when_cur_is_nested(
        &self,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        if !new_line_after || next_token.is_none() {
            return;
        }
        if self
            .syntax_handler
            .handler_immut::<BinOpHandler>()
            .need_inc_depth_by_long_op(next_token.unwrap().clone())
        {
            tracing::debug!(
                "bin_op_handler.need_inc_depth_by_long_op({:?})",
                next_token.unwrap().simple_str()
            );
            self.inc_depth();
            return;
        }

        if self
            .syntax_handler
            .handler_immut::<LetHandler>()
            .need_inc_depth_by_long_op(next_token.unwrap().clone())
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
        if !new_line_after || next_token.is_none() {
            return;
        }
        let bin_op_handler = self.syntax_handler.handler_immut::<BinOpHandler>();
        let is_cur_tok_bin_op = is_bin_op(token.get_end_tok());
        let is_next_tok_bin_op = is_bin_op(next_token.unwrap().get_start_tok());
        if (is_cur_tok_bin_op && bin_op_handler.need_inc_depth_by_long_op(token.clone()))
            || bin_op_handler.need_inc_depth_by_long_op(next_token.unwrap().clone())
        {
            tracing::debug!(
                "bin_op_handler.need_inc_depth_by_long_op22({:?})",
                next_token.unwrap().simple_str()
            );
            self.inc_depth();
            return;
        }

        let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
        if let_handler.need_inc_depth_by_long_op(token.clone())
            || (is_next_tok_bin_op
                && let_handler.need_inc_depth_by_long_op(next_token.unwrap().clone()))
        {
            self.inc_depth();
            return;
        }

        if self
            .syntax_handler
            .handler_immut::<QuantHandler>()
            .need_inc_depth_by_long_quant_exp(next_token.unwrap().clone())
        {
            self.inc_depth();
        }
    }

    fn need_dec_depth_when_cur_is_simple(&self, token: &TokenTree) {
        let bin_op_handler = self.syntax_handler.handler_immut::<BinOpHandler>();
        let let_handler = self.syntax_handler.handler_immut::<LetHandler>();

        if bin_op_handler.need_dec_depth_by_long_op(token.clone()) > 0 {
            tracing::debug!(
                "bin_op_handler.need_dec_depth_by_long_op({:?}), dec = {}",
                token.simple_str(),
                bin_op_handler.need_dec_depth_by_long_op(token.clone())
            );
        }

        let mut nested_break_line_depth = bin_op_handler.need_dec_depth_by_long_op(token.clone())
            + let_handler.need_dec_depth_by_long_op(token.clone())
            + self
                .syntax_handler
                .handler_immut::<QuantHandler>()
                .need_dec_depth_by_long_quant_exp(token.clone());

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
            TokenTree::Nested { .. } => {
                self.format_nested_token(token, next_token);
                self.need_inc_depth_when_cur_is_nested(next_token, new_line_after);
            }
            TokenTree::SimpleToken { .. } => {
                self.format_simple_token(token, next_token, new_line_after);
                self.need_inc_depth_when_cur_is_simple(token, next_token, new_line_after);
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
                if self.get_pre_simple_tok() != Tok::NumSign {
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
                    self.new_line(None);
                    last_cmt_is_block_cmt = false;
                }
                _ => {
                    let end = c.start_offset + (c.content.len() as u32);
                    let line_start = this_cmt_start_line;
                    let line_end = self.translate_line(end);

                    let no_space = [
                        &*RPAREN_STR,
                        &*COMMA_STR,
                        &*SEMICOLON_STR,
                    ];
                    if line_start != line_end {
                        self.new_line(None);
                    } else if !no_space.contains(&&content) {
                        self.push_str(" ");
                    }
                    last_cmt_is_block_cmt = true;
                }
            }
            self.comments_index.set(self.comments_index.get() + 1);
            self.cur_line
                .set(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));
            comment_nums_before_cur_simple_token += 1;
            last_cmt_start_pos = c.start_offset;
        }
        if comment_nums_before_cur_simple_token > 0 {
            if last_cmt_is_block_cmt
                && self.translate_line(pos) - self.translate_line(last_cmt_start_pos) == 1
            {
                // process this case:
                // line[i]: /*comment1*/ /*comment2*/
                // line[i+1]: code // located in `pos`
                let mut ret_copy = self.ret.clone().into_inner();
                if let Some(last_char) = ret_copy.chars().last()
                    && last_char == ' '
                {
                    ret_copy.pop();
                }
                *self.ret.borrow_mut() = ret_copy.trim_end().to_string();
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

            let kind = c.comment_kind();
            let fmted_cmt_str = c.format_comment(
                kind,
                self.depth.get() * self.local_cfg.indent_size,
                0,
                &self.global_cfg,
            );
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

    fn update_pos_and_space(
        &self,
        pos: &u32,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) {
        self.cur_line.set(self.translate_line(*pos));
        if new_line_after {
            return;
        }
        if expr_fmt::need_space(token, next_token) {
            self.push_str(" ");
        }
    }
}

impl Format {
    fn get_kind_len_after_trim_space(&self, kind: &NestKind) -> usize {
        self.format_context.borrow().content[kind.start_pos as usize..kind.end_pos as usize]
            .replace('\n', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("")
            .len()
    }

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
                TokenTree::Nested { kind, .. } => Some(kind.kind == NestKind_::Type),
            })
            .unwrap_or_default()
        {
            // not break for generic <Type>
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
            | Tok::EqualEqual
            | Tok::GreaterEqual
            | Tok::GreaterGreater
            | Tok::Pipe
            | Tok::PipePipe
            | Tok::NumValue
            | Tok::NumTypedValue => true,
            _ => false,
        };
        tracing::trace!("tok_suitable_for_new_line ret = {}", ret);
        ret
    }

    fn judge_change_new_line_when_over_limits(
        &self,
        tok_str: String,
        tok: Tok,
        note: Option<Note>,
        next: Option<&TokenTree>,
    ) -> bool {
        if self.get_pre_simple_tok() == Tok::AtSign {
            return false;
        }

        let len_plus_tok_len = self.last_line().len() + tok_str.len();
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
        *self.ret.borrow_mut() = remove_trailing_whitespaces(self.ret.clone().into_inner());
    }

    fn process_last_empty_line(&mut self) {
        *self.ret.borrow_mut() = update_last_line(self.ret.clone().into_inner());
    }

    fn get_pre_simple_tok(&self) -> Tok {
        self.format_context.borrow().pre_simple_token.get_end_tok()
    }

    fn get_last_line_leading_space_cnt(&self) -> usize {
        let trim_leading_space = self
            .last_line()
            .clone()
            .trim_start_matches(char::is_whitespace)
            .len();
        let mut leading_space_cnt = self.last_line().len() - trim_leading_space;
        if leading_space_cnt > self.local_cfg.indent_size && leading_space_cnt % 2 == 1 {
            leading_space_cnt -= 1;
            let remove_pos =
                self.ret.borrow().len() - trim_leading_space - self.local_cfg.indent_size;
            self.ret.borrow_mut().remove(remove_pos);
        }
        leading_space_cnt
    }
}

pub fn format_entry(content: impl AsRef<str>, config: Config) -> Result<String, Diagnostics> {
    let mut timer = Timer::start();
    let content = content.as_ref();

    {
        // https://github.com/movebit/movefmt/issues/2
        let _ = parse_file_string(&mut get_compile_env(), FileHash::empty(), content)?;
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
