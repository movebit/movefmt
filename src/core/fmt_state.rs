// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

//! Refactored state management design - functional formatter
//!
//! This module provides a formatter implementation based on immutable state,
//! replacing the original mutable state design based on RefCell/Cell.

use crate::core::token_tree::*;
use crate::syntax_fmt::bin_op_fmt::BinOpHandler;
use crate::syntax_fmt::branch_fmt::BranchHandler;
use crate::syntax_fmt::call_fmt::*;
use crate::syntax_fmt::fun_fmt::FunHandler;
use crate::syntax_fmt::let_fmt::LetHandler;
use crate::syntax_fmt::quant_fmt::QuantHandler;
use crate::syntax_fmt::skip_fmt::{SkipHandler, SkipType};
use crate::syntax_fmt::syntax_handler::SyntaxHandler;
use crate::syntax_fmt::{expr_fmt, fun_fmt, spec_fmt};
use crate::tools::utils::*;
use commentfmt::comment::contains_comment;
use commentfmt::{Config, Verbosity};
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::{ast::*, syntax::parse_file_string};
use move_ir_types::location::ByteIndex;
use std::sync::{Arc, LazyLock};
use tracing::debug;

const EXIST_MULTI_MODULE_TAG: &str = "module fmt";
const EXIST_MULTI_ADDRESS_TAG: &str = "address fmt";
const MIN_NESTED_LENGTH: usize = 16;
const MIN_BREAK_LENGTH: usize = 32;
const MAX_ANALYZE_LENGTH: usize = 64;
const BRACE_LEN_BREAK_LIMIT: usize = 46;

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

static MODULE_STR: LazyLock<String> = LazyLock::new(|| Tok::Module.to_string());
static NUMSIGN_STR: LazyLock<String> = LazyLock::new(|| Tok::NumSign.to_string());
static COMMA_STR: LazyLock<String> = LazyLock::new(|| Tok::Comma.to_string());
static FUN_STR: LazyLock<String> = LazyLock::new(|| Tok::Fun.to_string());
static RPAREN_STR: LazyLock<String> = LazyLock::new(|| Tok::RParen.to_string());
static SEMICOLON_STR: LazyLock<String> = LazyLock::new(|| Tok::Semicolon.to_string());

/// Immutable state during the formatting process
#[derive(Clone)]
pub struct FormatState {
    /// Current output content
    pub output: String,
    /// Current line number
    pub cur_line: u32,
    /// Current indentation depth
    pub depth: usize,
    /// Processed comment index
    pub comments_index: usize,
    /// Previous simple token
    pub pre_simple_token: TokenTree,
    /// Previous token tree
    pub pre_token_tree: TokenTree,
    /// Current nested type
    pub cur_nested_kind: NestKind,
    /// Function keyword position in output
    pub cur_fun_key_word_pos: usize,
    /// Whether current block has spec
    pub has_spec: bool,
}

impl FormatState {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            cur_line: 0,
            depth: 0,
            comments_index: 0,
            pre_simple_token: TokenTree::default(),
            pre_token_tree: TokenTree::default(),
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
            cur_fun_key_word_pos: 0,
            has_spec: false,
        }
    }

    /// Create state with estimated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            output: String::with_capacity(capacity),
            cur_line: 0,
            depth: 0,
            comments_index: 0,
            pre_simple_token: TokenTree::default(),
            pre_token_tree: TokenTree::default(),
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
            cur_fun_key_word_pos: 0,
            has_spec: false,
        }
    }

    pub fn push_str(mut self, s: &str) -> Self {
        self.output.push_str(s);
        self
    }

    pub fn inc_depth(mut self) -> Self {
        self.depth += 1;
        self
    }

    pub fn dec_depth(mut self) -> Self {
        if self.depth > 0 {
            self.depth -= 1;
        }
        self
    }

    /// Set current line number
    pub fn set_cur_line(mut self, line: u32) -> Self {
        self.cur_line = line;
        self
    }

    /// Update comment index
    pub fn advance_comment(mut self) -> Self {
        self.comments_index += 1;
        self
    }

    /// Update previous token
    pub fn set_pre_token(mut self, token: TokenTree) -> Self {
        self.pre_simple_token = token;
        self
    }

    /// Update current nested type
    pub fn set_nested_kind(mut self, kind: NestKind) -> Self {
        self.cur_nested_kind = kind;
        self
    }

    /// Get last line content
    pub fn last_line(&self) -> &str {
        self.output.lines().last().unwrap_or("")
    }

    /// Get type of previous simple token
    pub fn get_pre_simple_tok(&self) -> Tok {
        self.pre_simple_token.get_end_tok()
    }
}

impl Default for FormatState {
    fn default() -> Self {
        Self::new()
    }
}

/// Formatting context - contains immutable configuration and data
pub struct FormatContext {
    pub local_cfg: FormatConfig,
    pub global_cfg: Config,
    pub token_tree: Vec<TokenTree>,
    pub comments: Vec<Comment>,
    pub line_mapping: FileLineMappingOneFile,
    pub syntax_handler: SyntaxHandler,
    pub content: String,
}

#[derive(Clone, Default)]
pub struct FormatConfig {
    pub indent_size: usize,
    pub max_len_no_add_line: f32,
}

/// Refactored Format structure - only contains immutable data
pub struct FunctionalFormat {
    pub local_cfg: FormatConfig,
    pub global_cfg: Config,
    pub token_tree: Vec<TokenTree>,
    pub comments: Vec<Comment>,
    pub line_mapping: FileLineMappingOneFile,
    pub syntax_handler: SyntaxHandler,
    pub content: String,
}

// Helper functions
fn is_bin_op(tok: Tok) -> bool {
    BIN_OPS.contains(&tok)
}

fn is_big_block_token(token: &TokenTree, next: Option<&TokenTree>) -> bool {
    matches!(
        (
            token.get_end_tok(),
            token.simple_str(),
            next.map(|t| t.get_start_tok())
        ),
        (
            Tok::NumSign
                | Tok::Struct
                | Tok::Fun
                | Tok::Module
                | Tok::Spec
                | Tok::Public
                | Tok::Native
                | Tok::Inline,
            _,
            _
        ) | (_, Some("package" | "entry"), _)
            | (Tok::Friend, _, Some(Tok::Fun))
            | (Tok::Friend | Tok::Script, _, None)
    )
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

fn tune_module_buf(module_body: &mut String, config: &Config, has_spec: bool) {
    if has_spec {
        spec_fmt::fmt_spec(module_body, config.clone());
    }
    remove_trailing_whitespaces(module_body);
}

impl FunctionalFormat {
    pub fn new(global_cfg: Config, content: &str) -> Self {
        let ce: CommentExtrator = CommentExtrator::new(content).unwrap();
        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(content);
        Self {
            local_cfg: FormatConfig {
                indent_size: global_cfg.indent_size(),
                max_len_no_add_line: global_cfg.max_width() as f32 * 0.75,
            },
            global_cfg,
            token_tree: vec![],
            comments: ce.comments,
            line_mapping,
            syntax_handler: SyntaxHandler::new(content),
            content: content.to_string(),
        }
    }

    pub fn generate_token_tree(
        &mut self,
        defs: Vec<Definition>,
        content: &str,
    ) -> Result<String, Diagnostics> {
        let lexer = Lexer::new(content, FileHash::empty());
        let parse = crate::core::token_tree::Parser::new(lexer, &defs, content);
        self.token_tree = parse.parse_tokens();

        let defs = Arc::new(defs);
        self.syntax_handler.preprocess(&defs);
        Ok("parse ok".to_string())
    }

    pub fn format_token_trees(self) -> String {
        let mut state = FormatState::with_capacity(self.content.len());
        let mut pound_sign_idx = None;

        for (index, t) in self.token_tree.clone().into_iter().enumerate() {
            if t.is_pound() {
                pound_sign_idx = Some(index);
            }
            let new_line = pound_sign_idx.map_or(false, |x| (x + 1) == index);
            let mut apply_fmt = |s: &mut FormatState| -> FormatState {
                let mut s = self.format_token_trees_internal(
                    s.clone(),
                    &t,
                    self.token_tree.get(index + 1),
                    new_line,
                );
                if new_line {
                    s = self.new_line(s, Some(t.end_pos()));
                    pound_sign_idx = None;
                }
                s
            };

            let mut return_buf_cp = state.output.clone();
            let TokenTree::Nested {
                kind: nkind, note, ..
            } = t
            else {
                state = apply_fmt(&mut state);
                continue;
            };
            state = Self::record_spec_token(&t, state);
            let skip_handler = self.syntax_handler.handler_immut::<SkipHandler>();
            let is_mod_blk = skip_handler.is_module_block(&nkind);
            let is_addr_blk = note.map_or(false, |x| x == Note::ModuleAddress);
            if is_mod_blk {
                state.output = EXIST_MULTI_MODULE_TAG.to_string();
            }
            if is_addr_blk {
                state.output = EXIST_MULTI_ADDRESS_TAG.to_string();
            }

            state = apply_fmt(&mut state);
            if nkind.kind == NestKind_::Brace {
                state = self.new_line(state, Some(t.end_pos()));
            }
            let cfg = self.global_cfg.clone();
            // top level
            if is_mod_blk {
                if !skip_handler.has_skipped_module_body(&nkind) {
                    let mut current_content = state.output.clone();
                    let has_spec = state.has_spec;
                    tune_module_buf(&mut current_content, &cfg, has_spec);
                    state.output = current_content;
                }
                let module_body_buf = state.output.clone();
                return_buf_cp.push_str(&module_body_buf[EXIST_MULTI_MODULE_TAG.len()..]);
                state.output = return_buf_cp;
            } else if is_addr_blk {
                let fmt_buf = state.output.clone();
                let def_vec_result =
                    parse_file_string(&mut get_compile_env(), FileHash::empty(), &fmt_buf);
                let def_vec = def_vec_result.unwrap_or_default().0;

                let mut last_mod_end_loc = 0;
                let mut fmt_slice = String::new();
                let Some(Definition::Address(address_def)) = def_vec.first() else {
                    return_buf_cp.push_str(&fmt_buf[EXIST_MULTI_ADDRESS_TAG.len()..]);
                    state.output = return_buf_cp.clone();
                    continue;
                };
                for (mod_idx, mod_def) in address_def.modules.iter().enumerate() {
                    let this_module = &fmt_buf[last_mod_end_loc..mod_def.loc.start() as usize];
                    if mod_idx == 0 {
                        fmt_slice.push_str(this_module);
                    } else {
                        fmt_slice.push_str("\n\n");
                        fmt_slice.push_str(this_module.trim_start());
                    }
                    let m = &fmt_buf[mod_def.loc.start() as usize..mod_def.loc.end() as usize];
                    let mut tuning_mod_body = m.to_string();
                    let has_spec = state.has_spec;
                    tune_module_buf(&mut tuning_mod_body, &cfg, has_spec);
                    fmt_slice.push_str(&tuning_mod_body);
                    last_mod_end_loc = mod_def.loc.end() as usize;
                }

                fmt_slice.push_str(&fmt_buf[last_mod_end_loc..fmt_buf.len()]);

                tracing::debug!("return_buf_cp = {:?}", return_buf_cp);
                tracing::debug!("fmt_slice = {:?}", fmt_slice);
                return_buf_cp.push_str(&fmt_slice[EXIST_MULTI_ADDRESS_TAG.len()..]);
                state.output = return_buf_cp.clone();
            } else if nkind.kind == NestKind_::Brace {
                tracing::debug!("<script> return_buf_cp = {:?}", return_buf_cp);
                tracing::debug!("<script> self.ret = {:?}", &state.output);
                let mut current_content = state.output.clone();
                let has_spec = state.has_spec;
                tune_module_buf(&mut current_content, &cfg, has_spec);
                state.output = current_content;
            }
            state.has_spec = false;
            state = self.process_last_empty_line(state);
        }
        let (state_result, _, _) =
            self.add_comments(state, u32::MAX, "end_of_move_file".to_string());
        state = state_result;
        remove_trailing_whitespaces(&mut state.output);
        state = self.process_last_empty_line(state);
        state.output
    }

    fn record_spec_token(token: &TokenTree, mut state: FormatState) -> FormatState {
        let mut stack = vec![token];
        while let Some(t) = stack.pop() {
            match t {
                TokenTree::SimpleToken { tok, .. } if *tok == Tok::Spec => {
                    state.has_spec = true;
                    return state;
                }
                TokenTree::Nested { elements, .. } => {
                    stack.extend(elements);
                }
                _ => {}
            }
        }
        state
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
        state: &FormatState,
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
                state.last_line().len() + 2,
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
                state.last_line().len(),
            ) != ComplexCallKind::Pack
            {
                return judge_equal_tok_is_long_op_fn();
            }
        }

        false
    }

    fn check_next_token_is_long_bin_op(
        &self,
        state: &FormatState,
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
        let len_plus_cur_token = state.last_line().len() + current_token_len + 2;
        if len_plus_cur_token > self.global_cfg.max_width() {
            return false;
        }

        if let TokenTree::Nested { elements, .. } = current {
            let delimiter = analyze_token_tree_delimiter(elements).0;
            let cur_nested_break_mode = self.get_break_mode_begin_nested(state, current, delimiter);
            if cur_nested_break_mode.0 || cur_nested_break_mode.1 == Some(true) {
                return false;
            }
            for nested_nested_in_current_tree in elements {
                if let TokenTree::Nested { kind: tmp_kind, .. } = nested_nested_in_current_tree
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
                "state.last_line().len() = {:?}, r_exp_len_tuple = {:?}",
                state.last_line().len(),
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
        state: &FormatState,
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

            let len_plus_cur_token = state.last_line().len() + current.token_len() as usize + 2;
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
        state: &FormatState,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
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
            && state
                .pre_simple_token
                .simple_str()
                .and_then(|s| token_to_ability(state.get_pre_simple_tok(), &s))
                .is_some()
            && token_to_ability(next_tok, &next_content).is_some()
        {
            return false;
        }

        let mut new_line = if component_break_mode {
            self.check_new_line_mode_for_cur_tok(kind, delimiter, t, next_t)
                || (cur_is_delimiter && d.is_some() && kind.kind != NestKind_::Type)
        } else {
            self.get_new_line_mode_for_cur_tok(kind, t, next_t)
        };

        if nested_kind_len > MIN_NESTED_LENGTH && kind.kind != NestKind_::Type {
            new_line |= self
                .check_cur_token_is_long_bin_op(state, t, next_t, next_tok, index, kind, &elements);
            if !new_line && next_t.is_some() {
                if self.check_next_token_is_long_bin_op(state, t, next_t, next_tok) {
                    return true;
                }
                if self.check_next_token_is_quant_body(state, t, next_t) {
                    return true;
                }
            }
        }
        new_line
    }

    fn process_fn_header(&self, mut state: FormatState) -> FormatState {
        let cur = state.output.as_str();
        let last_fun_idx = state.cur_fun_key_word_pos;
        if last_fun_idx >= cur.len() {
            return state;
        }
        let indent = " ".repeat((state.depth + 1) * self.local_cfg.indent_size);
        let fun_specifier_fmted_str =
            fun_fmt::fun_header_specifier_fmt(&cur[last_fun_idx..], &indent);

        state.output.truncate(last_fun_idx);
        state.output.push_str(&fun_specifier_fmted_str);
        state
    }

    fn get_break_mode_of_fun_call(
        &self,
        state: &FormatState,
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
            state.last_line().len(),
        ) {
            let next_line_len = " "
                .to_string()
                .repeat((state.depth + 1) * self.local_cfg.indent_size)
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

    fn get_break_mode_begin_paren(
        &self,
        state: &FormatState,
        token: &TokenTree,
    ) -> (bool, Option<bool>) {
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
            + (state.depth + 1) * self.local_cfg.indent_size
            >= self.global_cfg.max_width();
        if matches!(state.get_pre_simple_tok(), Tok::If | Tok::While) {
            return (false, Some(opt_component_break_mode));
        }

        let (is_in_fun_header, fun_len) = self
            .syntax_handler
            .handler_immut::<FunHandler>()
            .is_parameter_paren_in_fun_header(kind);

        if !is_in_fun_header && state.last_line().len() > self.global_cfg.max_width() {
            return (true, Some(opt_component_break_mode));
        }
        if self
            .syntax_handler
            .handler_immut::<CallHandler>()
            .paren_in_call(kind)
        {
            return (
                self.get_break_mode_of_fun_call(
                    state,
                    token,
                    nested_token_len,
                    &mut opt_component_break_mode,
                ),
                Some(opt_component_break_mode),
            );
        }

        let paren_str = &self.content[kind.start_pos as usize..kind.end_pos as usize];
        if contains_comment(&paren_str) && paren_str.find("//").is_some() {
            return (true, Some(true));
        }

        let cur_line_status = get_code_buf_len(state.last_line().to_string());
        let cur_line_len = if cur_line_status.1 {
            cur_line_status.0
        } else {
            state.last_line().len()
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
        state: &FormatState,
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
        let nested_blk_str = &self.content[kind.start_pos as usize..kind.end_pos as usize];
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
                    state.last_line().len() + first_ele_len > max_line_width && first_ele_len > 8;
            }
            NestKind_::ParentTheses => return self.get_break_mode_begin_paren(state, token),
            NestKind_::Bracket => {
                let is_annotation = state.get_pre_simple_tok() == Tok::NumSign;
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
                new_line_mode |= state.last_line().len() + nested_len > MAX_ANALYZE_LENGTH;

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
                    new_line_mode |= state.last_line().len() + nested_len > max_line_width;

                    // case2: has special keyword
                    new_line_mode |= has_special_key(state.last_line().to_string());
                }

                // case3: nested_len too long
                new_line_mode |= nested_len > BRACE_LEN_BREAK_LIMIT;

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
        mut state: FormatState,
        kind: &NestKind,
        elements: &[TokenTree],
        b_new_line_mode: bool,
        b_add_indent: bool,
        b_add_space_at_bound: bool,
    ) -> FormatState {
        // step1 -- format start_token
        state = self.format_token_trees_internal(
            state,
            &kind.start_token_tree(),
            None,
            b_new_line_mode,
        );

        // step2 -- paired effect with step6
        if b_add_indent && b_new_line_mode {
            state = state.inc_depth();
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
            state = self.add_new_line_after_nested_begin(state, kind, elements, b_new_line_mode);
        } else if b_add_space_at_bound {
            state = state.push_str(" ");
        }
        state
    }

    fn bottom_half_before_kind_end(
        &self,
        mut state: FormatState,
        kind: &NestKind,
        b_new_line_mode: bool,
        b_add_indent: bool,
        b_add_space_at_bound: bool,
        nested_token_head: Tok,
    ) -> FormatState {
        // step5
        let (state_result, _, _) = self.add_comments(
            state,
            kind.end_pos,
            kind.end_token_tree()
                .simple_str()
                .unwrap_or_default()
                .to_string(),
        );
        state = state_result;
        let ret_copy = state.output.clone();
        // may be already add_a_new_line in step5 (doc_comment in tail of line)
        state.output = ret_copy.trim_end().to_string();
        let had_rm_added_new_line = state.output.lines().count() < ret_copy.lines().count();

        // step6 -- paired effect with step2
        if b_add_indent && b_new_line_mode {
            state = state.dec_depth();
        }
        // step7
        if b_new_line_mode || had_rm_added_new_line {
            tracing::debug!(
                "end_of_nested_block, had_rm_added_new_line = {}, last_ret = {}",
                had_rm_added_new_line,
                state.last_line()
            );
            let mut b_break_line_before_kind_end = true;
            if nested_token_head == Tok::If || kind.kind == NestKind_::Type {
                // 20240426 -- for [] and <>  don't add new line
                // 20240801 -- for <>  don't add new line
                b_break_line_before_kind_end = false;
            }

            if contains_comment(state.last_line()) && state.last_line().contains("//") {
                b_break_line_before_kind_end = true;
            }
            if b_break_line_before_kind_end {
                tracing::trace!(
                    "end_of_nested_block, new_line(), last_ret = {}",
                    state.last_line()
                );
                state = self.new_line(state, Some(kind.end_pos));
            }
        } else if b_add_space_at_bound {
            state = state.push_str(" ");
        }
        state
    }

    fn add_new_line_after_nested_begin(
        &self,
        mut state: FormatState,
        kind: &NestKind,
        elements: &[TokenTree],
        b_new_line_mode: bool,
    ) -> FormatState {
        if !b_new_line_mode {
            return state;
        }

        if !elements.is_empty() {
            let next_token_start_pos = elements.first().unwrap().start_pos();
            if self.translate_line(next_token_start_pos) > self.translate_line(kind.start_pos) {
                state = self.process_same_line_comment(state, kind.start_pos, true);
                return self.new_line(state, None);
            }
        }
        self.new_line(state, Some(kind.start_pos))
    }

    fn format_single_token(
        &self,
        mut state: FormatState,
        nested_token: &TokenTree,
        token_idx: usize,
        new_line: bool,
    ) -> FormatState {
        let TokenTree::Nested { elements, .. } = nested_token else {
            return state;
        };
        let token = elements.get(token_idx).unwrap();
        let next_t = elements.get(token_idx + 1);

        let pre_tok_is_num_sign = Tok::NumSign == state.get_pre_simple_tok();
        state =
            self.format_token_trees_internal(state, token, next_t, pre_tok_is_num_sign || new_line);

        if pre_tok_is_num_sign {
            tracing::debug!("in loop<TokenTree::Nested> pre_tok_is_num_sign = true");
            state = self.new_line(state, Some(token.end_pos()));
            return state;
        }

        if new_line {
            let process_tail_comment_of_line = match next_t {
                Some(next_token) => {
                    let next_token_start_pos = next_token.start_pos();
                    self.translate_line(next_token_start_pos) > self.translate_line(token.end_pos())
                }
                None => {
                    let remain_code_str = &self.content[token.end_pos() as usize..];
                    let mut remain_code_iter = remain_code_str.split_whitespace().clone();
                    let remain_code_first_word = remain_code_iter.next().unwrap_or_default();
                    remain_code_first_word.starts_with("//")
                        || remain_code_first_word.starts_with("/*")
                }
            };
            state = self.process_same_line_comment(
                state,
                token.end_pos(),
                process_tail_comment_of_line,
            );
            state = self.new_line(state, None);
        }
        state
    }

    fn format_dot_exp_chain(
        &self,
        mut state: FormatState,
        elements: &[TokenTree],
        idx: &mut usize,
        nested_token: &TokenTree,
    ) -> (FormatState, bool) {
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
                state = self.format_single_token(state, nested_token, *idx, false);
                *idx += 1;
            }
            return (state, false);
        }
        debug!("before process_link, last_line = {}", state.last_line());
        state = state.inc_depth();
        while *idx <= new_idx {
            let next_is_dot = elements
                .get(*idx + 1)
                .map_or(false, |t| t.get_start_tok() == Tok::Period);

            state = self.format_single_token(state, nested_token, *idx, next_is_dot);
            *idx += 1;
        }
        state = state.dec_depth();

        (state, true)
    }

    fn format_nested_elements(
        &self,
        mut state: FormatState,
        nested_token: &TokenTree,
        delimiter: Option<Delimiter>,
        component_break_mode: bool,
    ) -> FormatState {
        let TokenTree::Nested { elements, kind, .. } = nested_token else {
            return state;
        };
        let call_handler = self.syntax_handler.handler_immut::<CallHandler>();
        let nestd_kind_len = self.get_kind_len_after_trim_space(kind);
        let old_kind = state.cur_nested_kind;
        state.cur_nested_kind = *kind;
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
                &state,
                nested_token,
                delimiter,
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
                        state.last_line().len(),
                    );
            }

            if token_idx == nested_ele_len - 1 && last_is_comma {
                break;
            }

            if Tok::Period == state.get_pre_simple_tok() {
                let (new_state, processed) =
                    self.format_dot_exp_chain(state, elements, &mut token_idx, nested_token);
                state = new_state;
                if processed {
                    continue;
                }
            }

            state = self.format_single_token(state, nested_token, token_idx, new_line);
            token_idx += 1;
        }

        state.cur_nested_kind = old_kind;
        state
    }

    fn need_space_at_bound(
        &self,
        state: &FormatState,
        nested_token: &TokenTree,
        b_new_line_mode: bool,
    ) -> bool {
        let TokenTree::Nested { elements, kind, .. } = nested_token else {
            return true;
        };
        if b_new_line_mode {
            return false;
        }
        if elements.is_empty() {
            let nested_blk_str = &self.content[kind.start_pos as usize + 1..kind.end_pos as usize];
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
                    let nested_token_head = state.get_pre_simple_tok();
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

    fn need_skip_nested_token(
        &self,
        mut state: FormatState,
        kind: &NestKind,
        note: &Option<Note>,
    ) -> (FormatState, bool) {
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
            let blk_body_str = &self.content[kind.start_pos as usize..kind.end_pos as usize + 1];
            debug!("should_skip_block_body = {:?}", blk_body_str);
            state = state.push_str(blk_body_str);

            for c in &self.comments[state.comments_index..] {
                if c.start_offset > kind.end_pos {
                    break;
                }
                state = state.advance_comment();
            }
            state = state.set_cur_line(self.translate_line(kind.end_pos));
            return (state, true);
        }
        (state, false)
    }

    fn format_nested_token(
        &self,
        mut state: FormatState,
        nested_token: &TokenTree,
        next_token: Option<&TokenTree>,
    ) -> FormatState {
        let TokenTree::Nested {
            elements,
            kind,
            note,
        } = nested_token
        else {
            return state;
        };
        let (state_result, should_skip) = self.need_skip_nested_token(state, &kind, note);
        state = state_result;
        if should_skip {
            state.pre_simple_token = nested_token.clone();
            return state;
        }

        let (delimiter, _) = analyze_token_tree_delimiter(elements);
        if note.map_or(false, |x| x == Note::FunBody) {
            state = self.process_fn_header(state);
        }
        let (mut b_new_line_mode, opt_component_break_mode) =
            self.get_break_mode_begin_nested(&state, nested_token, delimiter);

        let mut b_add_indent = true;
        for i in 0..elements.len() {
            let ele_str = elements[i].simple_str().unwrap_or_default();
            if ele_str == MODULE_STR.as_str() {
                b_add_indent = false;
                b_new_line_mode |= true;
                break;
            } else if !(ele_str == NUMSIGN_STR.as_str() || ele_str.is_empty())
                || i > MIN_NESTED_LENGTH
            {
                break;
            }
        }

        let nested_token_head = state.get_pre_simple_tok();
        let b_add_space_at_bound = self.need_space_at_bound(&state, nested_token, b_new_line_mode);

        // step1-step3
        state = self.top_half_after_kind_start(
            state,
            kind,
            elements,
            b_new_line_mode,
            b_add_indent,
            b_add_space_at_bound,
        );

        // step4 -- format element
        state = self.format_nested_elements(
            state,
            nested_token,
            delimiter,
            opt_component_break_mode.unwrap_or(b_new_line_mode),
        );

        // step5-step7
        state = self.bottom_half_before_kind_end(
            state,
            kind,
            b_new_line_mode,
            b_add_indent,
            b_add_space_at_bound,
            nested_token_head,
        );

        // step8 -- format end_token
        state = self.format_token_trees_internal(state, &kind.end_token_tree(), None, false);
        if expr_fmt::need_space(nested_token, next_token) {
            if nested_token_head == Tok::NumSign && kind.kind == NestKind_::Bracket {
                return state;
            }
            state = state.push_str(" ");
        }
        state
    }

    fn maybe_begin_of_if_else(
        &self,
        mut state: FormatState,
        cur_nested_kind: NestKind,
        token: &TokenTree,
        pre_simple_token: &TokenTree,
        next_token: Option<&TokenTree>,
    ) -> FormatState {
        // updated in 20240517: add condition `NestKind_::Bracket`
        if cur_nested_kind.kind == NestKind_::Bracket {
            return state;
        }
        let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        else {
            return state;
        };

        let pre_tok = pre_simple_token.get_end_tok();
        let branch_handler = self.syntax_handler.handler_immut::<BranchHandler>();
        // optimize in 20241212
        if !matches!(pre_tok, Tok::RParen | Tok::Else) && *tok != Tok::Else {
            return state;
        }

        // added in 20240115
        // updated in 20241212: fix https://github.com/movebit/movefmt/issues/43
        let end_pos_of_if_cond_or_else = pre_simple_token.end_pos();
        if Tok::LBrace != *tok
            && content != "for"
            && branch_handler.need_new_line_after_branch(
                state.last_line().to_string(),
                *pos,
                self.global_cfg.clone(),
                end_pos_of_if_cond_or_else,
            )
        {
            tracing::debug!("need_new_line_after_branch[{:?}], add a new line", content);
            state = state.inc_depth();
            let cur_line = state.last_line();
            if cur_line.trim_start().len() == 0 {
                // maybe already added new line because of judge_cond() is a long nested expr
                state = state.push_str(" ".to_string().repeat(self.local_cfg.indent_size).as_str());
                return state;
            }
            return self.new_line(state, None);
        }

        // updated in 20240516: optimize break line before else
        let mut new_line_before_else = false;
        if *tok == Tok::Else {
            if pre_tok == Tok::RBrace {
                // case1
                if get_code_buf_len(state.last_line().to_string()).1 {
                    // process case:
                    // else if() {} `insert '\n' here` else
                    new_line_before_else = true;
                }
            } else if next_token.is_some() {
                // case2
                if state.last_line().len()
                    + content.len()
                    + 2
                    + next_token.unwrap().token_len() as usize
                    > self.global_cfg.max_width() - MIN_NESTED_LENGTH
                {
                    new_line_before_else = true;
                }

                // case3
                if branch_handler.else_branch_too_long(
                    state.last_line().to_string(),
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
            state = self.new_line(state, None);
        }
        state
    }

    fn maybe_begin_of_big_block(
        &self,
        mut state: FormatState,
        pos: u32,
        pre_token_tree: &TokenTree,
    ) -> FormatState {
        if let TokenTree::Nested { .. } = pre_token_tree {
            state.output = state.output.trim_end().to_string();
            state = self.new_line(state, None);
            if self.translate_line(pos) - state.cur_line == 0 {
                state = self.new_line(state, None);
            }
        }
        state
    }

    fn maybe_end_of_if_else(
        &self,
        mut state: FormatState,
        cur_nested_kind: NestKind,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
    ) -> FormatState {
        // added in 20240115
        // updated in 20240124
        // updated in 20240222: remove condition `if Tok::RBrace != *tok `
        // updated in 20240517: add condition `NestKind_::Bracket`
        if let TokenTree::SimpleToken { content, pos, .. } = token
            && cur_nested_kind.kind != NestKind_::Bracket
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
                state = state.dec_depth();
                nested_branch_depth -= 1;
            }

            if need_add_new_line
                && next_token.is_some()
                && next_token.unwrap().simple_str().unwrap_or_default() != ";"
            {
                state = self.new_line(state, None);
            }
        }
        state
    }

    fn process_blank_lines_before_simple_token(
        &self,
        mut state: FormatState,
        token: &TokenTree,
        pre_token_tree_ty: &TokenTreeType,
        is_normal_token: bool,
        new_line_before_cmt: bool,
        new_line_after_cmt: bool,
    ) -> FormatState {
        let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        else {
            return state;
        };
        let pre_simple_token = &state.pre_simple_token;
        let source = &self.content;

        let pre_simple_token_end_pos = pre_simple_token.end_pos();
        if (pre_simple_token_end_pos as usize) < MIN_NESTED_LENGTH {
            return state;
        }
        let pre_tok = pre_simple_token.get_end_tok();
        let line_diff = self.translate_line(*pos) - state.cur_line;

        let pre_is_big_block = pre_token_tree_ty == &TokenTreeType::SpecialBlkBrace;
        let pre_is_simple_token = pre_token_tree_ty == &TokenTreeType::Simple;
        let pre_is_normal_brace = pre_token_tree_ty == &TokenTreeType::NormalBrace;

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
        if line_diff > 1
            && expr_fmt::need_newline_when_trim_blank_line(&pre_tok, tok)
            && (is_normal_token || tok == &Tok::Spec || pre_is_normal_brace)
        {
            // There are multiple blank lines between the cur_line and the current code simple_token
            tracing::debug!(
                "self.translate_line(*pos) = {}, state.cur_line = {}",
                self.translate_line(*pos),
                state.cur_line
            );
            tracing::debug!("SimpleToken[{:?}], add a new line", content);
            if !is_normal_token && pre_is_normal_brace {
                if !state.output.trim_end_matches(' ').ends_with('\n') {
                    state = self.new_line(state, None);
                }
            }

            state = self.new_line(state, None);
            return state;
        }

        if is_normal_token {
            return state;
        }

        // 1. Collect the output into a Vec<char> (allocate once).
        let chars: Vec<char> = state.output.chars().collect();
        // 2. Take the last 36 characters, or as many as available.
        let start = chars.len().saturating_sub(36);
        let last_36: String = chars[start..].iter().collect();
        let already_added_new_line = last_36.trim_end_matches(' ').ends_with('\n');

        if pre_is_big_block || pre_is_normal_brace {
            if line_diff == 0 && pre_is_big_block {
                // The keyword is on the same line as the last line.
                if !&source[pre_simple_token_end_pos as usize + 1..*pos as usize]
                    .trim()
                    .is_empty()
                {
                    state = self.new_line(state, None);
                }
                return state;
            }

            if line_diff == 0 && pre_is_normal_brace {
                state = self.new_line(state, None);
                return state;
            }

            if new_line_after_cmt {
                // There is a comment between the two blocks.
                if !new_line_before_cmt {
                    state = self.new_line(state, None);
                }
                return state;
            }
            if !new_line_before_cmt {
                state = self.new_line(state, None);
            }
            if !already_added_new_line {
                state = self.new_line(state, None);
            }
            return state;
        }

        if pre_is_simple_token {
            // The previous token is a simple token, or possibly the opening of a NestedTokenTree.
            if pre_tok == Tok::LBrace {
                if already_added_new_line {
                    // A newline has already been emitted.
                    return state;
                }
                state = self.new_line(state, None);
                return state;
            }

            if pre_tok == Tok::Semicolon {
                let maybe_comment = &source[pre_simple_token_end_pos as usize + 1..*pos as usize];
                let has_comment = !maybe_comment.trim().is_empty();
                if already_added_new_line {
                    // A newline has already been emitted.
                    if !has_comment {
                        state = self.new_line(state, None);
                    }
                    return state;
                }

                if line_diff == 0 {
                    if has_comment {
                        state = self.new_line(state, None);
                    }
                } else {
                    // The keyword is far away from the previous token or comment.
                    state = self.new_line(state, None);
                }
            }
        }
        state
    }

    fn may_inc_depth_before_fun_ret_ty(
        &self,
        mut state: FormatState,
        next_token: Option<&TokenTree>,
    ) -> FormatState {
        if next_token.is_none() {
            return state;
        }
        let last_line_len = state.last_line().len();
        let ret_type_len = self
            .syntax_handler
            .handler_immut::<FunHandler>()
            .is_fun_return_colon(next_token.unwrap());
        if ret_type_len == 0 {
            return state;
        }
        if last_line_len > MIN_BREAK_LENGTH
            && ret_type_len + last_line_len >= self.global_cfg.max_width()
        {
            state = state.inc_depth();
            state = self.new_line(state, None);
            state = state.dec_depth();
        } else if state
            .last_line()
            .trim_start_matches(char::is_whitespace)
            .len()
            == 0
        {
            state = self.indent(state);
        }
        state
    }

    fn handle_split_line(
        &self,
        mut state: FormatState,
        cur_nested_kind: NestKind,
        leading_space_cnt: usize,
    ) -> FormatState {
        let need_inc_depth = !matches!(
            cur_nested_kind.kind,
            NestKind_::Bracket | NestKind_::ParentTheses
        );
        if need_inc_depth {
            let cur_indent_cnt = state.depth * self.local_cfg.indent_size;
            if leading_space_cnt + self.local_cfg.indent_size == cur_indent_cnt {
                tracing::debug!("cur_indent_cnt: {}", cur_indent_cnt);
                state = self.new_line(state, None);
            } else {
                state = state.inc_depth();
                state = self.new_line(state, None);
                state = state.dec_depth();
            }
        } else {
            state = self.new_line(state, None);
        }
        state
    }

    fn fmt_simple_token_core(
        &self,
        mut state: FormatState,
        cur_nested_kind: NestKind,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
        pre_tok: Tok,
    ) -> FormatState {
        let TokenTree::SimpleToken {
            content,
            pos,
            tok,
            note,
        } = token
        else {
            return state;
        };

        let leading_space_cnt = self.get_last_line_leading_space_cnt(&mut state);

        // These very long `Tok`s appear after `bin_op`:
        // "[Num]", "[NumTyped]", "[ByteString]", "[Identifier]",
        if content.len() > MAX_ANALYZE_LENGTH && state.last_line().len() < MAX_ANALYZE_LENGTH {
            let need_early_process = if pre_tok != Tok::Equal {
                true
            } else {
                let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
                // if true, means already change new line and increased depth.
                let_handler.is_long_let_assign_rhs_end(token.clone()) > 0
            };
            if need_early_process {
                state = state.push_str(content.as_str());
                return self.update_pos_and_space(state, pos, token, next_token, new_line_after);
            }
        }

        if self.judge_change_new_line_when_over_limits(
            &state,
            content.clone(),
            *tok,
            pre_tok,
            *note,
            next_token,
        ) {
            state = self.handle_split_line(state, cur_nested_kind, leading_space_cnt);
        } else if *tok == Tok::Colon {
            state = self.may_inc_depth_before_fun_ret_ty(state, next_token);
        }

        state = state.push_str(content.as_str());
        self.update_pos_and_space(state, pos, token, next_token, new_line_after)
    }

    fn format_simple_token(
        &self,
        mut state: FormatState,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) -> FormatState {
        if let TokenTree::SimpleToken {
            content, pos, tok, ..
        } = token
        {
            let pre_token_tree_ty = state.pre_token_tree.get_type();
            let is_big_blk_token = is_big_block_token(token, next_token);

            // step1
            let cur_nested_kind = state.cur_nested_kind;
            let pre_simple_token = state.pre_simple_token.clone();
            let pre_token_tree = state.pre_token_tree.clone();
            state = self.maybe_begin_of_if_else(
                state,
                cur_nested_kind,
                token,
                &pre_simple_token,
                next_token,
            );
            if pre_token_tree_ty == TokenTreeType::SpecialBlkBrace && is_big_blk_token {
                state = self.maybe_begin_of_big_block(state, *pos, &pre_token_tree);
            }

            // step2: add comment(xxx) before current simple_token
            let (state_result, new_line_before_cmt, new_line_after_cmt) =
                self.add_comments(state, *pos, content.clone());
            state = state_result;

            // step3
            state = self.process_blank_lines_before_simple_token(
                state,
                token,
                &pre_token_tree_ty,
                !is_big_blk_token,
                new_line_before_cmt,
                new_line_after_cmt,
            );

            // step4
            let cur_nested_kind = state.cur_nested_kind;
            let pre_tok = state.pre_simple_token.get_end_tok();
            state = self.fmt_simple_token_core(
                state,
                cur_nested_kind,
                token,
                next_token,
                new_line_after,
                pre_tok,
            );

            // step5
            let cur_nested_kind = state.cur_nested_kind;
            state = self.maybe_end_of_if_else(state, cur_nested_kind, token, next_token);

            // step6
            state.pre_simple_token = token.clone();
            if tok == &Tok::Fun {
                state.cur_fun_key_word_pos = state.output.len();
            }
        }
        state
    }

    fn need_inc_depth_when_cur_is_nested(
        &self,
        mut state: FormatState,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) -> FormatState {
        if !new_line_after || next_token.is_none() {
            return state;
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
            state = state.inc_depth();
            return state;
        }

        if self
            .syntax_handler
            .handler_immut::<LetHandler>()
            .need_inc_depth_by_long_op(next_token.unwrap().clone())
        {
            state = state.inc_depth();
        }
        state
    }

    fn need_inc_depth_when_cur_is_simple(
        &self,
        mut state: FormatState,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) -> FormatState {
        if !new_line_after || next_token.is_none() {
            return state;
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
            state = state.inc_depth();
            return state;
        }

        let let_handler = self.syntax_handler.handler_immut::<LetHandler>();
        if let_handler.need_inc_depth_by_long_op(token.clone())
            || (is_next_tok_bin_op
                && let_handler.need_inc_depth_by_long_op(next_token.unwrap().clone()))
        {
            state = state.inc_depth();
            return state;
        }

        if self
            .syntax_handler
            .handler_immut::<QuantHandler>()
            .need_inc_depth_by_long_quant_exp(next_token.unwrap().clone())
        {
            state = state.inc_depth();
        }
        state
    }

    fn need_dec_depth_when_cur_is_simple(
        &self,
        mut state: FormatState,
        token: &TokenTree,
    ) -> FormatState {
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
            state = state.dec_depth();
            nested_break_line_depth -= 1;
        }
        state
    }

    fn format_token_trees_internal(
        &self,
        mut state: FormatState,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) -> FormatState {
        match token {
            TokenTree::Nested { .. } => {
                state = self.format_nested_token(state, token, next_token);
                state = self.need_inc_depth_when_cur_is_nested(state, next_token, new_line_after);
            }
            TokenTree::SimpleToken { .. } => {
                state = self.format_simple_token(state, token, next_token, new_line_after);
                state = self.need_inc_depth_when_cur_is_simple(
                    state,
                    token,
                    next_token,
                    new_line_after,
                );
                state = self.need_dec_depth_when_cur_is_simple(state, token);
            }
        }
        state.pre_token_tree = token.clone();
        state
    }

    fn add_comments(
        &self,
        mut state: FormatState,
        pos: u32,
        content: String,
    ) -> (FormatState, bool, bool) {
        let mut comment_nums_before_cur_simple_token = 0;
        let mut last_cmt_is_block_cmt = false;
        let mut last_cmt_start_pos = 0;
        let mut new_line_before_cmt = false;
        let mut new_line_after_cmt = false;
        let pre_tok = state.pre_simple_token.get_end_tok();
        for c in &self.comments[state.comments_index..] {
            if c.start_offset > pos {
                break;
            }
            let this_cmt_start_line = self.translate_line(c.start_offset);
            let line_diff = this_cmt_start_line - state.cur_line;
            let cmt_kind = c.comment_kind();
            if line_diff == 1 {
                let ret_copy = state.output.clone();
                if ret_copy.trim_end_matches(' ').ends_with('\n') {
                    if !new_line_before_cmt {
                        state.output = ret_copy.trim_end().to_string();
                        state = self.new_line(state, None);
                    }
                } else {
                    state = self.new_line(state, None);
                }
                new_line_before_cmt = true;
            }
            if line_diff > 1 {
                tracing::debug!(
                    "the pos[{:?}] of this comment > current line[{:?}]",
                    c.start_offset,
                    state.cur_line
                );
                if pre_tok != Tok::NumSign {
                    state = self.new_line(state, None);
                }
                if !new_line_before_cmt {
                    new_line_before_cmt = true;
                }
            }

            if self.no_space_or_new_line_for_comment(&state) {
                state = state.push_str(" ");
            }

            let fmted_cmt = c.format_comment(
                cmt_kind,
                state.depth * self.local_cfg.indent_size,
                0,
                &self.global_cfg,
            );
            state = state.push_str(&fmted_cmt);

            match cmt_kind {
                CommentKind::BlockComment => {
                    let end = c.start_offset + (c.content.len() as u32);
                    let line_start = this_cmt_start_line;
                    let line_end = self.translate_line(end);

                    let no_space: [&str; 3] = [&RPAREN_STR, &COMMA_STR, &SEMICOLON_STR];
                    if line_start != line_end {
                        state = self.new_line(state, None);
                        new_line_after_cmt = true;
                    } else if !no_space.contains(&content.as_str()) {
                        state = state.push_str(" ");
                        new_line_after_cmt = false;
                    }
                    last_cmt_is_block_cmt = true;
                }
                _ => {
                    state = self.new_line(state, None);
                    last_cmt_is_block_cmt = false;
                    new_line_after_cmt = true;
                }
            }
            state = state.advance_comment();
            state = state
                .set_cur_line(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));
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
                let mut ret_copy = state.output.clone();
                if let Some(last_char) = ret_copy.chars().last()
                    && last_char == ' '
                {
                    ret_copy.pop();
                }
                state.output = ret_copy.trim_end().to_string();
                state = self.new_line(state, None);
            }
        }

        (state, new_line_before_cmt, new_line_after_cmt)
    }

    fn no_space_or_new_line_for_comment(&self, state: &FormatState) -> bool {
        if state.output.chars().last().is_some() {
            !state.output.ends_with('\n')
                && !state.output.ends_with(' ')
                && !state.output.ends_with('(')
        } else {
            false
        }
    }

    fn indent(&self, mut state: FormatState) -> FormatState {
        let indent_str = " ".repeat(state.depth * self.local_cfg.indent_size);
        state = state.push_str(&indent_str);
        state
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
        mut state: FormatState,
        add_line_comment_pos: u32,
        process_tail_comment_of_line: bool,
    ) -> FormatState {
        for c in &self.comments[state.comments_index..] {
            if !process_tail_comment_of_line && c.start_offset > add_line_comment_pos {
                break;
            }

            if self.translate_line(add_line_comment_pos) != self.translate_line(c.start_offset) {
                break;
            }

            let kind = c.comment_kind();
            let fmted_cmt_str = c.format_comment(
                kind,
                state.depth * self.local_cfg.indent_size,
                0,
                &self.global_cfg,
            );
            if self.no_space_or_new_line_for_comment(&state) {
                state = state.push_str(" ");
            }

            state = state.push_str(&fmted_cmt_str);
            state = state.advance_comment();
            state = state
                .set_cur_line(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));

            if let CommentKind::BlockComment = kind {
                let end = c.start_offset + (c.content.len() as u32);
                let line_start = self.translate_line(c.start_offset);
                let line_end = self.translate_line(end);
                if line_start != line_end {
                    tracing::debug!("in new_line, add CommentKind::BlockComment");
                    state = self.new_line(state, None);
                    return state;
                }
            }
        }
        state
    }

    fn new_line(
        &self,
        mut state: FormatState,
        add_line_comment_option: Option<u32>,
    ) -> FormatState {
        let (add_line_comment, b_add_comment) = match add_line_comment_option {
            Some(add_line_comment) => (add_line_comment, true),
            _ => (0, false),
        };
        if b_add_comment {
            state = self.process_same_line_comment(state, add_line_comment, false);
        }
        state = state.push_str("\n");
        self.indent(state)
    }

    fn update_pos_and_space(
        &self,
        mut state: FormatState,
        pos: &u32,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
    ) -> FormatState {
        state = state.set_cur_line(self.translate_line(*pos));
        if new_line_after {
            return state;
        }
        if expr_fmt::need_space(token, next_token) {
            state = state.push_str(" ");
        }
        state
    }

    fn get_kind_len_after_trim_space(&self, kind: &NestKind) -> usize {
        self.content[kind.start_pos as usize..kind.end_pos as usize]
            .replace('\n', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("")
            .len()
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
        state: &FormatState,
        tok_str: String,
        tok: Tok,
        pre_tok: Tok,
        note: Option<Note>,
        next: Option<&TokenTree>,
    ) -> bool {
        if pre_tok == Tok::AtSign {
            return false;
        }

        let len_plus_tok_len = state.last_line().len() + tok_str.len();
        if tok == Tok::AtSign && next.is_some() {
            let next_tok_len = next.unwrap().simple_str().unwrap_or_default().len();
            if next_tok_len > 8 && len_plus_tok_len + next_tok_len > self.global_cfg.max_width() {
                return true;
            }
        }

        len_plus_tok_len > self.global_cfg.max_width()
            && Self::tok_suitable_for_new_line(tok, note, next)
    }

    fn process_last_empty_line(&self, mut state: FormatState) -> FormatState {
        state.output.truncate(state.output.trim_end().len());
        state.output.push('\n');
        state
    }

    fn get_last_line_leading_space_cnt(&self, state: &mut FormatState) -> usize {
        let last_line = state.last_line();
        let trim_leading_space = last_line.trim_start_matches(char::is_whitespace).len();
        let mut leading_space_cnt = last_line.len() - trim_leading_space;
        if leading_space_cnt > self.local_cfg.indent_size && leading_space_cnt % 2 == 1 {
            leading_space_cnt -= 1;
            let remove_pos = state.output.len() - trim_leading_space - self.local_cfg.indent_size;
            state.output.remove(remove_pos);
        }
        leading_space_cnt
    }
}

pub fn format_entry(content: impl AsRef<str>, config: Config) -> Result<String, Diagnostics> {
    let mut timer = Timer::start();
    let content = content.as_ref();

    // https://github.com/movebit/movefmt/issues/2
    let (defs, _) = parse_file_string(&mut get_compile_env(), FileHash::empty(), content)?;

    let mut full_fmt = FunctionalFormat::new(config.clone(), content);

    full_fmt.generate_token_tree(defs, content)?;
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
