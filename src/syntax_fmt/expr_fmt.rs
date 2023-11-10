#![allow(dead_code)]
use std::cell::RefCell;

use std::result::Result::*;

use move_command_line_common::files::FileHash;
use move_compiler::diagnostics::Diagnostics;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use std::cell::Cell;
use std::collections::BTreeSet;
use crate::core::fmt::*;
use crate::core::token_tree::{
    Comment, CommentExtrator, CommentKind, Delimiter, NestKind_, Note, TokenTree,
};
use crate::utils::FileLineMappingOneFile;
pub fn format_use(fmter: &mut Format) {
    
}

pub fn process_expr_in_token_trees(fmter: &mut Format) {
    let length = fmter.token_tree.len();
    let mut index = 0;
    while index < length {
        let t = fmter.token_tree.get(index).unwrap();
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

            }
        }
        index += 1;
    }

}