#![allow(dead_code)]
use std::cell::RefCell;
use std::result::Result::*;
use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
// use stderrlog::new;
use std::cell::Cell;
use std::collections::BTreeSet;
use crate::core::token_tree::{
    Comment, CommentExtrator, CommentKind, Delimiter, NestKind, NestKind_, Note, TokenTree,
};
use crate::utils::FileLineMappingOneFile;
use crate::syntax::{parse_file_string, self};
use crate::syntax_fmt::{expr_fmt, fun_fmt, spec_fmt};
pub enum FormatEnv {
    FormatUse,
    FormatStruct,
    FormatExp,
    FormatTuple,
    FormatList,
    FormatLambda,
    FormatFun,
    FormatSpecModule,
    FormatSpecStruct,
    FormatSpecFun,
    FormatDefault,
}
pub struct FormatContext {
    pub content: String,
    pub env: FormatEnv,
}
  
impl FormatContext {
    pub fn new(content: String, env: FormatEnv) -> Self {  
        FormatContext { content, env }
    }

    pub fn set_env(&mut self, env: FormatEnv) {  
        self.env = env;  
    }  
}

pub struct Format {
    pub config: FormatConfig,
    pub depth: Cell<usize>,
    pub token_tree: Vec<TokenTree>,
    pub comments: Vec<Comment>,
    pub line_mapping: FileLineMappingOneFile,
    pub comments_index: Cell<usize>,
    pub ret: RefCell<String>,
    pub cur_line: Cell<u32>,
    pub format_context: FormatContext,
}

pub struct FormatConfig {
    pub indent_size: usize,
}

impl Format {
    fn new(
        config: FormatConfig,
        comments: CommentExtrator,
        line_mapping: FileLineMappingOneFile,
        token_tree: Vec<TokenTree>,
        format_context: FormatContext,
    ) -> Self {
        Self {
            comments_index: Default::default(),
            config,
            depth: Default::default(),
            token_tree,
            comments: comments.comments,
            line_mapping,
            ret: Default::default(),
            cur_line: Default::default(),
            format_context,
        }
    }

    fn post_process(&mut self) {
        eprintln!("post_process >> meet Brace");
        *self.ret.borrow_mut() = fun_fmt::process_block_comment_before_fun_header(self.ret.clone().into_inner());
        *self.ret.borrow_mut() = fun_fmt::add_blank_row_in_two_funs(self.ret.clone().into_inner());
        *self.ret.borrow_mut() = fun_fmt::process_fun_header_too_long(self.ret.clone().into_inner());
        self.remove_trailing_whitespaces();
        *self.ret.borrow_mut() = expr_fmt::split_if_else_in_let_block(self.ret.clone().into_inner());
        self.remove_trailing_whitespaces();

        if self.ret.clone().into_inner().contains("spec") {
            *self.ret.borrow_mut() = spec_fmt::add_blank_row_in_two_blocks(self.ret.clone().into_inner());
            *self.ret.borrow_mut() = spec_fmt::process_spec_fn_header_too_long(self.ret.clone().into_inner());
            self.remove_trailing_whitespaces();
        }
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
                TokenTree::SimpleToken {
                    content: _,
                    pos: _,
                    tok: _,
                    note: _,
                } => {}
                TokenTree::Nested {
                    elements: _,
                    kind,
                    note: _,
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
        self.ret.into_inner()
    }

    fn need_new_line(
        kind: NestKind_,
        delimiter: Option<Delimiter>,
        _has_colon: bool,
        current: &TokenTree,
        next: Option<&TokenTree>,
    ) -> bool {
        if next.map(|x| x.simple_str()).flatten() == delimiter.map(|x| x.to_static_str()) {
            return false;
        }

        let b_judge_next_token = if let Some((next_tok, next_content)) = next.map(|x| {  
            match x {  
                TokenTree::SimpleToken {  
                    content,
                    pos: _,  
                    tok,  
                    note: _,  
                } => (tok.clone(), content.clone()),  
                TokenTree::Nested {  
                    elements: _,  
                    kind,  
                    note: _,  
                } => (kind.kind.start_tok(), kind.kind.start_tok().to_string()),  
            }
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
                | Tok::Abort => true,
                Tok::Identifier => {
                    if next_content.as_str() == "entry" {
                        true
                    } else { false }
                }
                _ => false,
            }
        } else { true };

        // special case for `}}`
        if match current {
            TokenTree::SimpleToken {
                content: _,
                pos: _,
                tok: _,
                note: _,
            } => false,
            TokenTree::Nested {
                elements: _,
                kind,
                note: _,
            } => kind.kind == NestKind_::Brace,
        } && kind == NestKind_::Brace && b_judge_next_token {
            return true;
        }
        false
    }

    fn need_new_line_for_each_token_in_nested(
        kind: &NestKind,
        elements: &Vec<TokenTree>,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        index: usize,
        new_line_mode: bool
    ) -> bool {
        let t = elements.get(index).unwrap();
        let next_t = elements.get(index + 1);
        let d = delimiter.map(|x| x.to_static_str());
        let t_str = t.simple_str();

        let mut new_line = if new_line_mode {
            if (Self::need_new_line(kind.kind, delimiter, has_colon, t, next_t)
            || d == t_str && d.is_some())
            && index != elements.len() - 1
            {
                true
            } else {
                false
            }
        } else {
            false
        };

        // comma in fun resource access specifier not change new line
        if d == t_str && d.is_some() {
            if let Some(deli_str) = d {
                if deli_str.contains(',')  {
                    let mut idx = index;
                    while idx != 0 {
                        let ele = elements.get(idx).unwrap();
                        idx = idx - 1;
                        if let Some(key) = ele.simple_str() {
                            if key.contains("fun") {
                                break;
                            }
                        }
                        if None == ele.simple_str() {
                            continue;
                        }
                        if matches!(
                            ele.simple_str().unwrap(),
                            "acquires" | "reads" | "writes" | "pure" )
                        {
                            new_line = false;
                            break;
                        }
                    }
                }
            }
        }

        // ablility not change new line
        if let Some((next_tok, next_content)) = next_t.map(|x| match x {
            TokenTree::SimpleToken {
                content,
                pos: _,
                tok,
                note: _,
            } => (tok.clone(), content.clone()),
            TokenTree::Nested {
                elements: _,
                kind,
                note: _,
            } => (kind.kind.start_tok(), kind.kind.start_tok().to_string())
        }) {
            if let Some(_) = syntax::token_to_ability(next_tok, &next_content) {
                new_line = false;
            }
        }
        new_line
    }

    fn process_fn_header_before_before_fn_nested(&self) {
        let cur_ret = self.ret.clone().into_inner();
        // eprintln!("fun_header = {:?}", &self.format_context.content[(kind.start_pos as usize)..(kind.end_pos as usize)]);
        if let Some(last_fun_idx) = cur_ret.rfind("fun") {
            let fun_header: &str = &cur_ret[last_fun_idx..];
            if let Some(specifier_idx) = fun_header.rfind("fun") {
                let indent_str = " ".to_string()
                    .repeat((self.depth.get() + 1) * self.config.indent_size);
                let fun_specifier_fmted_str = fun_fmt::fun_header_specifier_fmt(
                    &fun_header[specifier_idx+1..], &indent_str);

                let ret_copy = &self.ret.clone().into_inner()[0..last_fun_idx+specifier_idx+1];
                let mut new_ret = ret_copy.to_string();
                new_ret.push_str(fun_specifier_fmted_str.as_str());
                *self.ret.borrow_mut() = new_ret.to_string();
            }
        }
        if self.ret.clone().into_inner().contains("writes") {
            eprintln!("self.last_line = {:?}", self.last_line());
        }
    }

    fn get_new_line_mode_begin_nested(
        &self,
        kind: &NestKind,
        elements: &Vec<TokenTree>,
        note: &Option<Note>,
        delimiter: Option<Delimiter>,
    ) -> bool {
        let stct_def = note.map(|x| x == Note::StructDefinition).unwrap_or_default();
        let fun_body = note.map(|x| x == Note::FunBody).unwrap_or_default();

        const MAX_LEN_WHEN_NO_ADD_LINE: usize = 35;
        let length = self.analyze_token_tree_length(elements, MAX_LEN_WHEN_NO_ADD_LINE);

        if fun_body {
            self.process_fn_header_before_before_fn_nested();
        }
        let mut new_line_mode = {
            length > MAX_LEN_WHEN_NO_ADD_LINE
                || delimiter
                    .map(|x| x == Delimiter::Semicolon)
                    .unwrap_or_default()
                || (stct_def && elements.len() > 0)
                || fun_body
        };
        if fun_body && elements.len()== 0 {
            new_line_mode = false;
        }

        match kind.kind {
            NestKind_::ParentTheses
            | NestKind_::Bracket
            | NestKind_::Type
            | NestKind_::Lambda => {
                if delimiter.is_none() {
                    new_line_mode = false;
                }
            }
            NestKind_::Brace => {
                // added by lzw: 20231213
                if self.last_line().contains("module") {
                    new_line_mode = true;
                }
            }
        }
        new_line_mode
    }
    
    fn add_new_line_after_nested_begin(
        &self,
        kind: &NestKind,
        elements: &Vec<TokenTree>,
        b_new_line_mode: bool
    )
    {
        if !b_new_line_mode {
            if let NestKind_::Brace = kind.kind {
                if elements.len() == 1 {
                    self.push_str(" ");
                }
            }
            return;
        }

        if elements.len() > 0 {
            let next_token = elements.get(0).unwrap();
            let mut next_token_start_pos: u32 = 0;
            self.analyzer_token_tree_start_pos_(&mut next_token_start_pos, next_token);
            if self.translate_line(next_token_start_pos) > self.translate_line(kind.start_pos) {
                // let source = self.format_context.content.clone();
                // let start_pos: usize = next_token_start_pos as usize;
                // let (_, next_str) = source.split_at(start_pos);                                                  
                // eprintln!("after format_token_trees_internal<TokenTree::Nested-start_token_tree> return, next_token = {:?}", 
                //     next_str);
                // process line tail comment
                self.process_same_line_comment(kind.start_pos, true);
            } else {
                self.new_line(Some(kind.start_pos));
            }
        } else {
            self.new_line(Some(kind.start_pos));
        }
    }

    fn format_each_token_in_nested_elements(
        &self,
        kind: &NestKind,
        elements: &Vec<TokenTree>,
        delimiter: Option<Delimiter>,
        has_colon: bool,
        b_new_line_mode: bool
    ) {
        let mut pound_sign = None;
        let len = elements.len();
        for index in 0..len {
            let t = elements.get(index).unwrap();
            if t.is_pound() {
                pound_sign = Some(index)
            }
            let next_t = elements.get(index + 1);
            let pound_sign_new_line =
                pound_sign.map(|x| (x + 1) == index).unwrap_or_default();

            let new_line = Self::need_new_line_for_each_token_in_nested(
                kind,
                elements,
                delimiter,
                has_colon,
                index,
                b_new_line_mode
            );

            self.format_token_trees_internal(
                t,
                elements.get(index + 1),
                pound_sign_new_line || new_line,
            );
            if pound_sign_new_line {
                eprintln!("in loop<TokenTree::Nested> pound_sign_new_line = true");
                self.new_line(Some(t.end_pos()));
                pound_sign = None;
                continue;
            }

            if new_line {
                let process_tail_comment_of_line = match next_t {
                    Some(next_token) => {
                        let mut next_token_start_pos: u32 = 0;
                        self.analyzer_token_tree_start_pos_(&mut next_token_start_pos, next_token);
                        if self.translate_line(next_token_start_pos) > self.translate_line(t.end_pos()) {
                            true
                        } else {
                            false
                        }
                    }
                    None => {
                        true
                    }
                };
                self.process_same_line_comment(t.end_pos(), process_tail_comment_of_line);
            } else {
                if let NestKind_::Brace = kind.kind {
                    if elements.len() == 1 {
                        self.push_str(" ");
                    }
                }
            }
        }

    }

    fn format_nested_token(
        &self,
        token: &TokenTree,
        next_token: Option<&TokenTree>
    ) {
        if let TokenTree::Nested {
            elements,
            kind,
            note,
        } = token {
            let (delimiter, has_colon) = Self::analyze_token_tree_delimiter(elements);
            let b_new_line_mode = self.get_new_line_mode_begin_nested(kind, elements, note, delimiter);
            let b_add_indent = !note.map(|x| x == Note::ModuleAddress).unwrap_or_default();

            // step1 -- format start_token
            self.format_token_trees_internal(&kind.start_token_tree(), None, b_new_line_mode);

            // step2 -- paired effect with step6
            if b_add_indent {
                self.inc_depth();
            }

            // step3
            self.add_new_line_after_nested_begin(kind, elements, b_new_line_mode);

            // step4 -- format elements
            self.format_each_token_in_nested_elements(kind, elements, delimiter, has_colon, b_new_line_mode);
            
            // step5 -- add_comments which before kind.end_pos
            self.add_comments(kind.end_pos, kind.end_token_tree().simple_str().unwrap_or_default().to_string());
            let ret_copy = self.ret.clone().into_inner();
            // may be already add_a_new_line in step5 by add_comments(doc_comment in tail of line)
            *self.ret.borrow_mut() = ret_copy.trim_end().to_string();
            if ret_copy.chars().last() == Some(' ') {
                self.push_str(" ");
            }
            let had_rm_added_new_line = self.ret.clone().into_inner().lines().count() < ret_copy.lines().count();

            // step6 -- paired effect with step2
            if b_add_indent {
                self.dec_depth();
            }

            // step7
            if b_new_line_mode || (!b_new_line_mode && had_rm_added_new_line){
                eprintln!("end_of_nested_block, b_new_line_mode = true");
                self.new_line(Some(kind.end_pos));
            }

            // step8 -- format end_token
            self.format_token_trees_internal(&kind.end_token_tree(), None, false);
            match kind.end_token_tree() {
                TokenTree::SimpleToken {
                    content: _,
                    pos: _t_pos,
                    tok: _t_tok,
                    note: _,
                } => {
                    if expr_fmt::need_space(token, next_token) {
                        self.push_str(" ");
                    }
                }
                _ => {}
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
            note
        } = token {
            // add comment(xxx) before current simple_token
            self.add_comments(*pos, content.clone());
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
            if (self.translate_line(*pos) - self.cur_line.get()) > 1 {
                // There are multiple blank lines between the cur_line and the current code simple_token
                eprintln!("self.translate_line(*pos) = {}, self.cur_line.get() = {}", self.translate_line(*pos), self.cur_line.get());
                eprintln!("SimpleToken[{:?}], add a new line", content);
                self.new_line(None);
            }

            self.push_str(&content.as_str());
            self.cur_line.set(self.translate_line(*pos));
            if new_line_after {
                return;
            }
            if self.last_line_length() > 90
                && Self::tok_suitable_for_new_line(tok.clone(), note.clone(), next_token)
            {
                eprintln!("last_line = {:?}", self.last_line());
                eprintln!("SimpleToken{:?}, add a new line because of split line", content);
                self.new_line(None);
                return;
            }
            if expr_fmt::need_space(token, next_token) {
                self.push_str(" ");
            }
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
            } => self.format_nested_token(token, next_token),
            TokenTree::SimpleToken {
                content: _,
                pos: _,
                tok: _,
                note: _,
            } => self.format_simple_token(token, next_token, new_line_after)
        }
    }

    fn add_comments(&self, pos: u32, content: String) {
        let mut comment_nums_before_cur_simple_token = 0;
        for c in &self.comments[self.comments_index.get()..] {
            if c.start_offset > pos {
                break;
            }

            if (self.translate_line(c.start_offset) - self.cur_line.get()) > 1 {
                self.new_line(None);
            }

            if (self.translate_line(c.start_offset) - self.cur_line.get()) == 1 {
                // if located after nestedToken start, maybe already chanedLine
                let ret_copy = self.ret.clone().into_inner();
                *self.ret.borrow_mut() = ret_copy.trim_end().to_string();
                self.new_line(None);
            }

            // eprintln!("-- add_comments: line(c.start_offset) - cur_line = {:?}", 
            //     self.translate_line(c.start_offset) - self.cur_line.get());
            // eprintln!("c.content.as_str() = {:?}\n", c.content.as_str());
            if self.no_space_or_new_line_for_comment() {
                self.push_str(" ");
            }

            self.push_str(c.format_comment(
                c.comment_kind(), self.depth.get() * self.config.indent_size, 0));

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
                }
                _ => {
                    let end = c.start_offset + (c.content.len() as u32);
                    let line_start = self.translate_line(c.start_offset);
                    let line_end = self.translate_line(end);

                    if !content.contains(")")
                    && !content.contains(",")
                    && !content.contains(";")
                    && !content.contains(".") {
                        self.push_str(" ");
                    }
                    if line_start != line_end {
                        self.new_line(None);
                    }
                }
            }
            self.comments_index.set(self.comments_index.get() + 1);
            self.cur_line.set(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));
            comment_nums_before_cur_simple_token = comment_nums_before_cur_simple_token + 1;
            // eprintln!("in add_comments for loop: self.cur_line = {:?}\n", self.cur_line);
        }
        if comment_nums_before_cur_simple_token > 0 {
            eprintln!("add_comments[{:?}] before pos[{:?}] = {:?} return <<<<<<<<<\n", 
                comment_nums_before_cur_simple_token, pos, content);
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
        self.depth.set(old - 1);
    }
    fn push_str(&self, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.ret.borrow_mut().push_str(s);
    }

    fn no_space_or_new_line_for_comment(&self) -> bool {
        if self.ret.borrow().chars().last().is_some() {
            self.ret.borrow().chars().last().unwrap() != '\n' &&
            self.ret.borrow().chars().last().unwrap() != ' ' && 
            self.ret.borrow().chars().last().unwrap() != '('
        } else {
            false
        }
    }

    /// 缩进
    fn indent(&self) {
        self.push_str(
            " ".to_string()
                .repeat(self.depth.get() * self.config.indent_size)
                .as_str(),
        );
    }

    fn translate_line(&self, pos: u32) -> u32 {
        self.line_mapping.translate(pos, pos).unwrap().start.line
    }

    /// analyzer a `Nested` token tree.
    fn analyze_token_tree_delimiter(
        token_tree: &Vec<TokenTree>,
    ) -> (
        Option<Delimiter>, // if this is a `Delimiter::Semicolon` we can know this is a function body or etc.
        bool,              // has a `:`
    ) {
        let mut d = None;
        let mut has_colon = false;
        for t in token_tree.iter() {
            match t {
                TokenTree::SimpleToken {
                    content,
                    pos: _,
                    tok: _,
                    note: _,
                } => match content.as_str() {
                    ";" => {
                        d = Some(Delimiter::Semicolon);
                    }
                    "," => {
                        if d.is_none() {
                            // Somehow `;` has high priority.
                            d = Some(Delimiter::Comma);
                        }
                    }
                    ":" => {
                        has_colon = true;
                    }
                    _ => {}
                },
                TokenTree::Nested { .. } => {}
            }
        }
        return (d, has_colon);
    }

    fn analyzer_token_tree_start_pos_(&self, ret: &mut u32, token_tree: &TokenTree) {
        match token_tree {
            TokenTree::SimpleToken { content: _, pos, .. } => {
                *ret = *pos;
            }
            TokenTree::Nested { elements: _, kind, .. } => {
                *ret = kind.start_pos;
            }
        }
    }

    fn analyzer_token_tree_length_(&self, ret: &mut usize, token_tree: &TokenTree, max: usize) {
        match token_tree {
            TokenTree::SimpleToken { content, .. } => {
                *ret = *ret + content.len();
            }
            TokenTree::Nested { elements, .. } => {
                for t in elements.iter() {
                    self.analyzer_token_tree_length_(ret, t, max);
                    if *ret > max {
                        return;
                    }
                }
                *ret = *ret + 2; // for delimiter.
            }
        }
    }

    /// analyzer How long is list of token_tree
    fn analyze_token_tree_length(&self, token_tree: &Vec<TokenTree>, max: usize) -> usize {
        let mut ret = usize::default();
        for t in token_tree.iter() {
            self.analyzer_token_tree_length_(&mut ret, t, max);
            if ret > max {
                return ret;
            }
        }
        ret
    }

    fn process_same_line_comment(&self, add_line_comment_pos: u32, process_tail_comment_of_line: bool) {
        let cur_line = self.cur_line.get();
        let mut call_new_line = false;
        for c in &self.comments[self.comments_index.get()..] {
            if !process_tail_comment_of_line && c.start_offset > add_line_comment_pos {
                break;
            }

            if self.translate_line(add_line_comment_pos) != self.translate_line(c.start_offset) {
                break;
            }
            // eprintln!("self.translate_line(c.start_offset) = {}, self.cur_line.get() = {}", self.translate_line(c.start_offset), self.cur_line.get());
            // eprintln!("add a new line[{:?}], meet comment", c.content);
            // if (self.translate_line(c.start_offset) - self.cur_line.get()) > 1 {
            //     eprintln!("add a black line");
            //     self.new_line(None);
            // }
            // self.push_str(c.content.as_str());
            let kind = c.comment_kind();
            let fmted_cmt_str = c.format_comment(
                kind, self.depth.get() * self.config.indent_size, 0);
            // eprintln!("fmted_cmt_str in same_line = \n{}", fmted_cmt_str);
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
            match kind {
                CommentKind::BlockComment => {
                    let end = c.start_offset + (c.content.len() as u32);
                    let line_start = self.translate_line(c.start_offset);
                    let line_end = self.translate_line(end);
                    if line_start != line_end {
                        eprintln!("in new_line, add CommentKind::BlockComment");
                        self.new_line(None);
                        call_new_line = true;
                    }
                }
                _ => {
                    // eprintln!("-- process_same_line_comment, add CommentKind::_({})", c.content);
                    self.new_line(None);
                    call_new_line = true;
                }
            }
            self.comments_index.set(self.comments_index.get() + 1);
            self.cur_line.set(self.translate_line(c.start_offset + (c.content.len() as u32) - 1));
        }
        if cur_line != self.cur_line.get() || call_new_line {
            eprintln!("success new line, return <<<<<<<<<<<<<<<<< \n");
            return;
        }
        self.push_str("\n");
        self.indent();
    }

    fn new_line(&self, add_line_comment_option: Option<u32>) {
        let (add_line_comment, b_add_comment) = match add_line_comment_option {
            Some(add_line_comment) => (add_line_comment, true),
            _  => (0, false),
        };
        if !b_add_comment {
            self.push_str("\n");
            self.indent();
            return;
        }
        self.process_same_line_comment(add_line_comment, false);
    }

}

impl Format {
    fn last_line_length(&self) -> usize {
        self.ret
            .borrow()
            .lines()
            .last()
            .map(|x| x.len())
            .unwrap_or_default()
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
        // special case
        if next
            .map(|x| match x {
                TokenTree::SimpleToken {
                    content: _,
                    pos: _,
                    tok: _,
                    note: _,
                } => None,
                TokenTree::Nested {
                    elements: _,
                    kind,
                    note: _,
                } => Some(kind.kind == NestKind_::Type),
            })
            .flatten()
            .unwrap_or_default()
        {
            // eprintln!("tok_suitable_for_new_line ret false");
            return false;
        }
        let is_bin = note.map(|x| x == Note::BinaryOP).unwrap_or_default();
        let ret = match tok {
            Tok::Less | Tok::Amp | Tok::Star | Tok::Greater if is_bin => true,
            Tok::ExclaimEqual
            | Tok::Percent
            | Tok::AmpAmp
            | Tok::ColonColon
            | Tok::Plus
            | Tok::Minus
            | Tok::Period
            | Tok::PeriodPeriod
            | Tok::Slash
            | Tok::LessEqual
            | Tok::LessLess
            | Tok::Equal
            | Tok::EqualEqual
            | Tok::EqualEqualGreater
            | Tok::LessEqualEqualGreater
            | Tok::GreaterEqual
            | Tok::GreaterGreater
            | Tok::NumValue => true,
            _ => false,
        };
        // eprintln!("tok_suitable_for_new_line ret = {}", ret);
        ret
    }

    fn remove_trailing_whitespaces(&mut self) {
        let ret_copy = self.ret.clone().into_inner();
        let lines = ret_copy.lines();
        let result: String = lines.collect::<Vec<_>>()
            .iter()  
            .map(|line| line.trim_end_matches(|c| c == ' '))  
            .collect::<Vec<_>>()
            .join("\n");
        *self.ret.borrow_mut() = result;
    }
}


pub fn format(content: impl AsRef<str>, config: FormatConfig) -> Result<String, Diagnostics> {
    let content = content.as_ref();
    let attrs: BTreeSet<String> = BTreeSet::new();
    let mut env = CompilationEnv::new(Flags::testing(), attrs);
    let filehash = FileHash::empty();
    let (defs, _) = parse_file_string(&mut env, filehash, &content)?;
    let lexer = Lexer::new(&content, filehash);
    let parse = crate::core::token_tree::Parser::new(lexer, &defs);
    let parse_result = parse.parse_tokens();
    let ce = CommentExtrator::new(content).unwrap();
    let mut t = FileLineMappingOneFile::default();
    t.update(&content);

    let f = Format::new(config, ce, t, parse_result, 
        FormatContext::new(content.to_string(), FormatEnv::FormatDefault));
    Ok(f.format_token_trees())
}
