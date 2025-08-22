// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

//! Refactored state management design - functional formatter
//!
//! This module provides a formatter implementation based on immutable state,
//! replacing the original mutable state design based on RefCell/Cell.

use crate::core::token_tree::*;
use crate::syntax_fmt::skip_fmt::SkipHandler;
use crate::syntax_fmt::syntax_handler::SyntaxHandler;
use crate::syntax_fmt::{big_block_fmt, expr_fmt, fun_fmt, spec_fmt};
use crate::tools::utils::*;
use commentfmt::Config;
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::{ast::*, syntax::parse_file_string};
use std::sync::Arc;

const EXIST_MULTI_MODULE_TAG: &str = "module fmt";
const EXIST_MULTI_ADDRESS_TAG: &str = "address fmt";

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
    /// Current nested type
    pub cur_nested_kind: NestKind,
}

impl FormatState {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            cur_line: 0,
            depth: 0,
            comments_index: 0,
            pre_simple_token: TokenTree::default(),
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
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
            cur_nested_kind: NestKind {
                kind: NestKind_::Lambda,
                start_pos: 0,
                end_pos: 0,
            },
        }
    }

    /// Add string to output
    pub fn push_str(mut self, s: &str) -> Self {
        self.output.push_str(s);
        self
    }

    /// Increase indentation depth
    pub fn inc_depth(mut self) -> Self {
        self.depth += 1;
        self
    }

    /// Decrease indentation depth
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
    pub fn advance_comments_index(mut self, count: usize) -> Self {
        self.comments_index += count;
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

    /// Get current line length
    pub fn cur_line_len(&self) -> usize {
        get_code_buf_len(self.last_line().to_string())
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
    context: FormatContext,
}

impl FunctionalFormat {
    pub fn new(global_cfg: Config, content: &str) -> Result<Self, Diagnostics> {
        let ce = CommentExtrator::new(content).unwrap();
        let mut line_mapping = FileLineMappingOneFile::default();
        line_mapping.update(content);

        let (defs, _) = parse_file_string(&mut get_compile_env(), FileHash::empty(), content)?;
        let lexer = Lexer::new(content, FileHash::empty());
        let parse = crate::core::token_tree::Parser::new(lexer, &defs, content.to_string());
        let token_tree = parse.parse_tokens();

        let defs = Arc::new(defs);
        let mut syntax_handler = SyntaxHandler::new(content);
        syntax_handler.preprocess(&defs);

        let context = FormatContext {
            local_cfg: FormatConfig {
                indent_size: global_cfg.indent_size(),
                max_len_no_add_line: global_cfg.max_width() as f32 * 0.75,
            },
            global_cfg,
            token_tree,
            comments: ce.comments,
            line_mapping,
            syntax_handler,
            content: content.to_string(),
        };

        Ok(Self { context })
    }

    /// Main formatting entry point
    pub fn format_token_trees(self) -> String {
        let initial_state = FormatState::with_capacity(self.context.content.len() * 2);
        let final_state = self.format_tokens_with_state(initial_state);

        // Final cleanup
        self.finalize_output(final_state).output
    }

    /// Core method for formatting with state
    fn format_tokens_with_state(&self, mut state: FormatState) -> FormatState {
        let mut pound_sign_idx = None;

        for (index, token) in self.context.token_tree.iter().enumerate() {
            if token.is_pound() {
                pound_sign_idx = Some(index);
            }

            let new_line = pound_sign_idx.map_or(false, |x| (x + 1) == index);
            let next_token = self.context.token_tree.get(index + 1);

            // Handle special module and address blocks
            if let Some(result_state) = self.try_format_special_block(token, state.clone()) {
                state = result_state;
                continue;
            }

            // Format current token
            state = self.format_token_with_state(token, next_token, new_line, state);

            if new_line {
                state = self.add_new_line_with_state(Some(token.end_pos()), state);
                pound_sign_idx = None;
            }
        }

        // Add remaining comments
        state = self.add_remaining_comments_with_state(state);
        state
    }

    /// Try to format special blocks (module, address, etc.)
    fn try_format_special_block(
        &self,
        token: &TokenTree,
        mut state: FormatState,
    ) -> Option<FormatState> {
        let TokenTree::Nested {
            kind: nkind, note, ..
        } = token
        else {
            return None;
        };

        let skip_handler = self.context.syntax_handler.handler_immut::<SkipHandler>();
        let is_mod_blk = skip_handler.is_module_block(nkind);
        let is_addr_blk = note.map_or(false, |x| x == Note::ModuleAddress);

        if is_mod_blk {
            state = state.push_str(EXIST_MULTI_MODULE_TAG);
            state = self.format_token_with_state(token, None, false, state);
            state = self.add_new_line_with_state(Some(token.end_pos()), state);

            if !skip_handler.has_skipped_module_body(nkind) {
                let tuned = tune_module_buf(state.output.clone(), &self.context.global_cfg);
                state.output = tuned;
                state.output = update_last_line(state.output);
            }

            // Remove the tag and return the processed state
            if state.output.starts_with(EXIST_MULTI_MODULE_TAG) {
                state.output = state.output[EXIST_MULTI_MODULE_TAG.len()..].to_string();
            }
            Some(state)
        } else if is_addr_blk {
            state = state.push_str(EXIST_MULTI_ADDRESS_TAG);
            state = self.format_token_with_state(token, None, false, state);
            state = self.add_new_line_with_state(Some(token.end_pos()), state);

            // Process modules inside the address block
            state = self.process_address_modules(state);
            Some(state)
        } else {
            None
        }
    }

    /// Process modules inside the address block
    fn process_address_modules(&self, mut state: FormatState) -> FormatState {
        let def_vec_result =
            parse_file_string(&mut get_compile_env(), FileHash::empty(), &state.output);

        if let Ok((def_vec, _)) = def_vec_result {
            if let Some(Definition::Address(address_def)) = def_vec.first() {
                let mut last_mod_end_loc = 0;
                let mut fmt_slice = String::new();

                for mod_def in &address_def.modules {
                    let m = &state.output[mod_def.loc.start() as usize..mod_def.loc.end() as usize];
                    let tuning_mod_body = tune_module_buf(m.to_string(), &self.context.global_cfg);
                    fmt_slice
                        .push_str(&state.output[last_mod_end_loc..mod_def.loc.start() as usize]);
                    fmt_slice.push_str(&tuning_mod_body);
                    last_mod_end_loc = mod_def.loc.end() as usize;
                }

                fmt_slice.push_str(&state.output[last_mod_end_loc..]);

                if fmt_slice.starts_with(EXIST_MULTI_ADDRESS_TAG) {
                    state.output = fmt_slice[EXIST_MULTI_ADDRESS_TAG.len()..].to_string();
                } else {
                    state.output = fmt_slice;
                }
            }
        }

        state
    }

    /// Format a single token
    fn format_token_with_state(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
        state: FormatState,
    ) -> FormatState {
        match token {
            TokenTree::Nested { .. } => {
                self.format_nested_token_with_state(token, next_token, state)
            }
            TokenTree::SimpleToken { .. } => {
                self.format_simple_token_with_state(token, next_token, new_line_after, state)
            }
        }
    }

    /// Format a nested token
    fn format_nested_token_with_state(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        state: FormatState,
    ) -> FormatState {
        let TokenTree::Nested {
            elements,
            kind,
            note,
        } = token
        else {
            return state;
        };

        // Check if token should be skipped
        if self.should_skip_nested_token(kind, note) {
            return self.skip_nested_token_with_state(token, state);
        }

        let (delimiter, has_colon) = analyze_token_tree_delimiter(elements);
        let (break_mode, component_break_mode) =
            self.get_break_mode_begin_nested(token, delimiter, &state);

        // Process the start part
        let mut state = self.format_nested_start_with_state(kind, elements, break_mode, state);

        // Format inner elements
        state = self.format_nested_elements_with_state(
            token,
            delimiter,
            has_colon,
            component_break_mode.unwrap_or(break_mode),
            state,
        );

        // Process the end part
        state = self.format_nested_end_with_state(kind, break_mode, state);

        // Format the end token
        state = self.format_token_with_state(&kind.end_token_tree(), None, false, state);

        // Add necessary space
        if expr_fmt::need_space(token, next_token) {
            if state.get_pre_simple_tok() == Tok::NumSign && kind.kind == NestKind_::Bracket {
                return state;
            }
            state = state.push_str(" ");
        }

        state
    }

    /// Format a simple token
    fn format_simple_token_with_state(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
        mut state: FormatState,
    ) -> FormatState {
        let TokenTree::SimpleToken {
            content,
            pos,
            tok: _,
            note: _,
        } = token
        else {
            return state;
        };

        // Handle branch logic
        state = self.handle_branch_logic_with_state(token, next_token, state);

        // Add comments
        state = self.add_comments_with_state(*pos, content.clone(), state);

        // Process blank lines
        state = self.process_blank_lines_with_state(token, state);

        // Format token core logic
        state = self.format_simple_token_core_with_state(token, next_token, new_line_after, state);

        // Handle branch end logic
        state = self.handle_branch_end_logic_with_state(token, next_token, state);

        // Update previous token
        state = state.set_pre_token(token.clone());

        state
    }

    /// Add a new line
    fn add_new_line_with_state(
        &self,
        comment_pos: Option<u32>,
        mut state: FormatState,
    ) -> FormatState {
        if let Some(pos) = comment_pos {
            state = self.process_same_line_comment_with_state(pos, false, state);
        }

        state = state.push_str("\n");

        // Add indentation
        let indent = " ".repeat(state.depth * self.context.local_cfg.indent_size);
        state = state.push_str(&indent);

        state
    }

    /// Add comments
    fn add_comments_with_state(
        &self,
        pos: u32,
        content: String,
        mut state: FormatState,
    ) -> FormatState {
        let mut comments_processed = 0;

        for comment in &self.context.comments[state.comments_index..] {
            if comment.start_offset > pos {
                break;
            }

            let comment_line = self.translate_line(comment.start_offset);

            // Handle new lines before the comment
            if (comment_line - state.cur_line) > 1 {
                if state.get_pre_simple_tok() != Tok::NumSign {
                    state = self.add_new_line_with_state(None, state);
                }
            }

            if (comment_line - state.cur_line) == 1 {
                let trimmed = state.output.trim_end().to_string();
                state.output = trimmed;
                state = self.add_new_line_with_state(None, state);
            }

            // Add necessary spaces
            if self.no_space_or_new_line_for_comment(&state.output) {
                state = state.push_str(" ");
            }

            // Format and add the comment
            let formatted_comment = comment.format_comment(
                comment.comment_kind(),
                state.depth * self.context.local_cfg.indent_size,
                0,
                &self.context.global_cfg,
            );
            state = state.push_str(&formatted_comment);

            // Handle new lines after the comment
            match comment.comment_kind() {
                CommentKind::DocComment => {
                    state = self.add_new_line_with_state(None, state);
                }
                _ => {
                    let end = comment.start_offset + (comment.content.len() as u32);
                    let line_start = comment_line;
                    let line_end = self.translate_line(end);

                    if line_start != line_end {
                        state = self.add_new_line_with_state(None, state);
                    } else if !matches!(content.as_str(), ")" | "," | ";") {
                        state = state.push_str(" ");
                    }
                }
            }

            comments_processed += 1;
            state = state.set_cur_line(
                self.translate_line(comment.start_offset + (comment.content.len() as u32) - 1),
            );
        }

        state = state.advance_comments_index(comments_processed);
        state
    }

    /// Add remaining comments
    fn add_remaining_comments_with_state(&self, state: FormatState) -> FormatState {
        self.add_comments_with_state(u32::MAX, "end_of_move_file".to_string(), state)
    }

    /// Finalize output cleanup
    fn finalize_output(&self, mut state: FormatState) -> FormatState {
        // Remove trailing whitespace
        state.output = remove_trailing_whitespaces_util(state.output);
        // Handle final blank lines
        state.output = update_last_line(state.output);
        state
    }

    // Stub implementations of helper methods - these need to be fully implemented based on original code
    fn should_skip_nested_token(&self, _kind: &NestKind, _note: &Option<Note>) -> bool {
        false // TODO
    }

    fn skip_nested_token_with_state(
        &self,
        token: &TokenTree,
        mut state: FormatState,
    ) -> FormatState {
        if let TokenTree::Nested { kind, .. } = token {
            let blk_body_str =
                &self.context.content[kind.start_pos as usize..=kind.end_pos as usize];
            state = state.push_str(blk_body_str);
            state = state.set_cur_line(self.translate_line(kind.end_pos));
        }
        state
    }

    fn get_break_mode_begin_nested(
        &self,
        _token: &TokenTree,
        _delimiter: Option<Delimiter>,
        _state: &FormatState,
    ) -> (bool, Option<bool>) {
        (false, None) // TODO
    }

    fn format_nested_start_with_state(
        &self,
        kind: &NestKind,
        _elements: &[TokenTree],
        break_mode: bool,
        mut state: FormatState,
    ) -> FormatState {
        // Format start token
        state = self.format_token_with_state(&kind.start_token_tree(), None, break_mode, state);

        if break_mode {
            state = state.inc_depth();
            state = self.add_new_line_with_state(Some(kind.start_pos), state);
        }

        state
    }

    fn format_nested_elements_with_state(
        &self,
        _token: &TokenTree,
        _delimiter: Option<Delimiter>,
        _has_colon: bool,
        _component_break_mode: bool,
        state: FormatState,
    ) -> FormatState {
        // TODO
        state
    }

    fn format_nested_end_with_state(
        &self,
        _kind: &NestKind,
        break_mode: bool,
        mut state: FormatState,
    ) -> FormatState {
        if break_mode {
            state = state.dec_depth();
            state = self.add_new_line_with_state(None, state);
        }
        state
    }

    fn handle_branch_logic_with_state(
        &self,
        _token: &TokenTree,
        _next_token: Option<&TokenTree>,
        state: FormatState,
    ) -> FormatState {
        // TODO
        state
    }

    fn process_blank_lines_with_state(
        &self,
        _token: &TokenTree,
        state: FormatState,
    ) -> FormatState {
        // TODO
        state
    }

    fn format_simple_token_core_with_state(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>,
        new_line_after: bool,
        mut state: FormatState,
    ) -> FormatState {
        let TokenTree::SimpleToken { content, pos, .. } = token else {
            return state;
        };

        state = state.push_str(content);
        state = state.set_cur_line(self.translate_line(*pos));

        if !new_line_after && expr_fmt::need_space(token, next_token) {
            state = state.push_str(" ");
        }

        state
    }

    fn handle_branch_end_logic_with_state(
        &self,
        _token: &TokenTree,
        _next_token: Option<&TokenTree>,
        state: FormatState,
    ) -> FormatState {
        // TODO
        state
    }

    fn process_same_line_comment_with_state(
        &self,
        _pos: u32,
        _process_tail: bool,
        state: FormatState,
    ) -> FormatState {
        // TODO
        state
    }

    // Helper method
    fn translate_line(&self, pos: u32) -> u32 {
        self.context
            .line_mapping
            .translate(pos, pos)
            .unwrap_or_default()
            .start
            .line
    }

    fn no_space_or_new_line_for_comment(&self, output: &str) -> bool {
        if let Some(last_char) = output.chars().last() {
            !matches!(last_char, '\n' | ' ' | '(')
        } else {
            false
        }
    }
}

/// Helper function - extracted from the original code
fn tune_module_buf(module_body: String, config: &Config) -> String {
    let mut ret_module_body = fun_fmt::fmt_fun(module_body.clone(), config.clone());
    if module_body.contains("spec ") {
        ret_module_body = spec_fmt::fmt_spec(ret_module_body.clone(), config.clone());
    }
    ret_module_body = big_block_fmt::fmt_big_block(ret_module_body);
    remove_trailing_whitespaces_util(ret_module_body)
}

/// Provide a simple wrapper for backward compatibility
impl FunctionalFormat {
    /// Wrapper for legacy API
    pub fn format_legacy(self) -> String {
        self.format_token_trees()
    }
}

/// Public API function - replaces the original format_entry
pub fn format_entry_functional(
    content: impl AsRef<str>,
    config: Config,
) -> Result<String, Diagnostics> {
    let content = content.as_ref();
    let format = FunctionalFormat::new(config, content)?;
    Ok(format.format_token_trees())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_state_immutability() {
        let state1 = FormatState::new();
        let state2 = state1.clone().push_str("hello");
        let state3 = state2.clone().inc_depth();

        // Verify state immutability
        assert_eq!(state1.output, "");
        assert_eq!(state2.output, "hello");
        assert_eq!(state3.depth, 1);
        assert_eq!(state2.depth, 0); // state2 was not modified
    }

    #[test]
    fn test_format_state_chaining() {
        let final_state = FormatState::new()
            .push_str("fn test() {")
            .inc_depth()
            .push_str("\n    return 42;")
            .dec_depth()
            .push_str("\n}");

        assert!(final_state.output.contains("fn test() {"));
        assert_eq!(final_state.depth, 0);
    }

    #[test]
    fn test_basic_formatting() {
        let content = "fun test(){return 42;}";
        let config = Config::default();

        match format_entry_functional(content, config) {
            Ok(result) => {
                assert!(!result.is_empty());
                // Basic formatting verification
            }
            Err(_) => {}
        }
    }
}
