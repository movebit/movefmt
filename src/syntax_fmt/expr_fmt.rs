// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core::token_tree::*;
use move_compiler::parser::lexer::{ Lexer, Tok };
use once_cell::sync::Lazy;

const NO_BREAK_TOKENS: &[Tok] = &[
    Tok::Script,
    Tok::Module,
    Tok::Struct,
    Tok::Fun,
    Tok::Spec,
    Tok::Const,
    Tok::Friend,
    Tok::If,
    Tok::Inline,
    Tok::Public,
    Tok::Use,
    Tok::While,
    Tok::Native,
    Tok::NumSign,
    Tok::Exclaim,
    Tok::ExclaimEqual,
    Tok::Percent,
    Tok::Star,
    Tok::Plus,
    Tok::Minus,
    Tok::Period,
    Tok::PeriodPeriod,
    Tok::Slash,
    Tok::Comma,
    Tok::Colon,
    Tok::ColonColon,
    Tok::Less,
    Tok::LessEqual,
    Tok::LessLess,
    Tok::Equal,
    Tok::Greater,
    Tok::GreaterEqual,
    Tok::GreaterGreater,
    Tok::Acquires,
    Tok::As,
    Tok::Invariant,
    Tok::EqualEqual,
];

const NO_BREAK_PAIRS: &[(Tok, Tok)] = &[
    (Tok::Amp, Tok::Identifier),
    (Tok::Amp, Tok::LParen),
    (Tok::Amp, Tok::NumValue),
    (Tok::AtSign, Tok::Identifier),
    (Tok::AtSign, Tok::LParen),
    (Tok::AtSign, Tok::NumValue),
    (Tok::Identifier, Tok::Identifier),
    (Tok::LBrace, Tok::Identifier),
    (Tok::LBrace, Tok::Module),
    (Tok::LBrace, Tok::NumTypedValue),
    (Tok::LBrace, Tok::NumValue),
    (Tok::LParen, Tok::Identifier),
    (Tok::LParen, Tok::NumTypedValue),
    (Tok::LParen, Tok::NumValue),
    (Tok::RBrace, Tok::RBrace),
];

static NO_BREAK_TOKENS_VEC: Lazy<Vec<Tok>> = Lazy::new(|| NO_BREAK_TOKENS.to_vec());

static NO_BREAK_PAIRS_VEC: Lazy<Vec<(Tok, Tok)>> = Lazy::new(|| NO_BREAK_PAIRS.to_vec());

pub enum TokType {
    /// abc like token, e.g., 'if', 'let', 'my_var'
    Alphabet,
    /// + - * / % ! == >= <= ...
    MathSign,
    ///
    Sign,
    // specials no need space at all.
    NoNeedSpace,
    /// numbers 0x1 ...
    Number,
    /// b"hello world"
    String,
    /// &
    Amp,
    /// *
    Star,
    ///
    Semicolon,
    ///:
    Colon,
    /// @
    AtSign,
    /// <
    Less,
}

impl From<Tok> for TokType {
    fn from(value: Tok) -> Self {
        match value {
            Tok::EOF => unreachable!(), // EOF not in `TokenTree`.
            Tok::NumValue => TokType::Number,
            Tok::NumTypedValue => TokType::Number,
            Tok::ByteStringValue => TokType::String,
            Tok::Exclaim => TokType::Sign,
            Tok::ExclaimEqual => TokType::MathSign,
            Tok::Percent => TokType::MathSign,
            Tok::Amp => TokType::Amp,
            Tok::AmpAmp => TokType::MathSign,
            Tok::LParen => TokType::Sign,
            Tok::RParen => TokType::Sign,
            Tok::LBracket => TokType::Sign,
            Tok::RBracket => TokType::Sign,
            Tok::Star => TokType::Star,
            Tok::Plus => TokType::MathSign,
            Tok::Comma => TokType::Sign,
            Tok::Minus => TokType::MathSign,
            Tok::Period => TokType::NoNeedSpace,
            Tok::PeriodPeriod => TokType::NoNeedSpace,
            Tok::Slash => TokType::Sign,
            Tok::Colon => TokType::Colon,
            Tok::ColonColon => TokType::NoNeedSpace,
            Tok::Semicolon => TokType::Semicolon,
            Tok::Less => TokType::Less,
            Tok::LessEqual => TokType::MathSign,
            Tok::LessLess => TokType::MathSign,
            Tok::Equal => TokType::MathSign,
            Tok::EqualEqual => TokType::MathSign,
            Tok::EqualEqualGreater => TokType::MathSign,
            Tok::LessEqualEqualGreater => TokType::MathSign,
            Tok::Greater => TokType::MathSign,
            Tok::GreaterEqual => TokType::MathSign,
            Tok::GreaterGreater => TokType::MathSign,
            Tok::LBrace => TokType::Sign,
            Tok::Pipe => TokType::Sign,
            Tok::PipePipe => TokType::MathSign,
            Tok::RBrace => TokType::Sign,
            Tok::NumSign => TokType::Sign,
            Tok::AtSign => TokType::AtSign,
            Tok::AmpMut => TokType::Amp,
            _ => TokType::Alphabet,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ChainMember {
    Field(String),
    Call(String, Vec<Vec<ChainMember>>), // function name + argument list (each arg is itself a chain)
}

struct DotChainParser<'a> {
    lexer: Lexer<'a>,
    result: Vec<ChainMember>,
}
// let mut lexer = Lexer::new(specifier, FileHash::empty());
// lexer.advance().unwrap();
// while lexer.peek() != Tok::EOF {
//     fun_specifiers_code.push((
//         lexer.start_loc() as u32,
//         (lexer.start_loc() + lexer.content().len()) as u32,
//         lexer.content().to_string(),
//     ));
//     if lexer.advance().is_err() {
//         break;
//     }
// }
// impl DotChainParser {
//     fn new(codespan_str: &str) -> Self {
//         Self {
//             lexer: Lexer::new(codespan_str, FileHash::empty()),
//             result: Vec::new(),
//         }
//     }

//     fn current(&self) -> &Tok {
//         &self.lexer.peek()
//     }

//     fn current_word(&self) -> &str {
//         &self.lexer.content()
//     }

//     fn advance(&mut self) {
//         self.lexer.advance().unwrap();
//     }

//     fn parse_chain(&mut self) -> Option<()> {
//         let first = match self.current() {
//             Tok::Identifier => {
//                 let n = self.current_word().to_string();
//                 self.advance();
//                 n
//             }
//             _ => return None,
//         };
//         self.result.push(ChainMember::Field(first));

//         while matches!(self.current(), Tok::Dot) {
//             self.advance();
//             self.parse_postfix()?;
//         }
//         Some(())
//     }

//     fn parse_postfix(&mut self) -> Option<()> {
//         let name = match self.current() {
//             Tok::Identifier => {
//                 let n = self.current_word().to_string();
//                 self.advance();
//                 n
//             }
//             _ => return None,
//         };
    
//         // Treat as Call if followed by '<' or '('
//         let is_call = matches!(self.current(), Tok::Less | Tok::LParen);
//         let args = self.parse_call_args()?; // consumes <>() or (); returns empty vec if none
//         if is_call || !args.is_empty() {
//             self.result.push(ChainMember::Call(name, args));
//         } else {
//             self.result.push(ChainMember::Field(name));
//         }
//         Some(())
//     }

//     fn parse_call_args(&mut self) -> Option<Vec<Vec<ChainMember>>> {
//         // Optional generic arguments <...>
//         if matches!(self.current(), Tok::Less) {
//             self.advance();
//             while !matches!(self.current(), Tok::Greater) && !matches!(self.current(), Tok::EOF) {
//                 self.advance();
//             }
//             if matches!(self.current(), Tok::Greater) {
//                 self.advance();
//             } else {
//                 return None;
//             }
//         }

//         if !matches!(self.current(), Tok::LParen) {
//             return Some(Vec::new());
//         }
//         self.advance();

//         let mut all = Vec::new();
//         loop {
//             if matches!(self.current(), Tok::RParen) {
//                 break;
//             }
//             let mut sub = Vec::new();
//             {
//                 let mut p2 = DotChainParser {
//                     tokens: self.tokens.clone(),
//                     result: sub,
//                 };
//                 p2.parse_chain()?;
//                 sub = p2.result;
//             }
//             all.push(sub);

//             if matches!(self.current(), Tok::Comma) {
//                 self.advance();
//             } else {
//                 break;
//             }
//         }

//         if matches!(self.current(), Tok::RParen) {
//             self.advance();
//             Some(all)
//         } else {
//             None
//         }
//     }
// }

// pub fn parse_dot_chain(codespan_str: &str) -> Option<Vec<ChainMember>> {
//     let mut p = DotChainParser::new(codespan_str);
//     p.parse_chain().and_then(|_| {
//         if matches!(p.current(), Tok::EOF) {
//             Some(p.result)
//         } else {
//             None
//         }
//     })
// }

fn is_to_or_except(token: &Option<&TokenTree>) -> bool {
    match token {
        None => false,
        Some(TokenTree::SimpleToken { content: con, .. }) => {
            con.as_str() == "to" || con.as_str() == "except"
        }
        _ => false,
    }
}

// Counts nested structures and commas within a slice of TokenTrees
pub(crate) fn get_nested_and_comma_num(elements: &[TokenTree]) -> (usize, usize) {
    let mut result = (0, 0);
    for ele in elements {
        if let TokenTree::Nested {
            elements: recursive_elements,
            ..
        } = ele
        {
            let recursive_result = get_nested_and_comma_num(recursive_elements);
            result.0 += recursive_result.0 + 1;
            result.1 += recursive_result.1;
        }
        if let TokenTree::SimpleToken {
            content: _,
            pos: _,
            tok,
            note: _,
        } = ele
        {
            if Tok::Comma == *tok {
                result.1 += 1;
            }
        }
    }

    result
}

// Determines if a space is needed between the current and next token for formatting.
pub(crate) fn need_space(current: &TokenTree, next: Option<&TokenTree>) -> bool {
    if next.is_none() {
        return false;
    }
    let next_token_tree = next.unwrap();

    let is_bin_current = current
        .get_note()
        .map(|x| x == Note::BinaryOP)
        .unwrap_or_default();
    let is_bin_next = next_token_tree
        .get_note()
        .map(|x| x == Note::BinaryOP)
        .unwrap_or_default();

    let is_apply_current = current
        .get_note()
        .map(|x| x == Note::ApplyName)
        .unwrap_or_default();
    let is_apply_next = next_token_tree
        .get_note()
        .map(|x| x == Note::ApplyName)
        .unwrap_or_default();

    let is_to_execpt = is_to_or_except(&Some(current)) || is_to_or_except(&next);

    let curr_start_tok = current.get_start_tok();
    let curr_end_tok = current.get_end_tok();
    let next_start_tok = next_token_tree.get_start_tok();

    if Tok::Greater == curr_end_tok {
        if let TokType::Alphabet = TokType::from(next_start_tok) {
            return true;
        }
        if Tok::LBrace == next_start_tok {
            return true;
        }
    }

    let mut is_next_tok_nested = false;
    let mut next_tok_nested_kind = NestKind_::Brace;
    let mut next_tok_simple_content = "".to_string();
    match next_token_tree {
        TokenTree::Nested { kind, .. } => {
            is_next_tok_nested = true;
            next_tok_nested_kind = kind.kind;
        }
        TokenTree::SimpleToken { content, .. } => {
            next_tok_simple_content = content.to_string();
        }
    }

    match (TokType::from(curr_start_tok), TokType::from(next_start_tok)) {
        (
            TokType::Alphabet,
            TokType::Alphabet | TokType::String | TokType::Number | TokType::AtSign,
        ) => true,
        (_, TokType::Amp) => Tok::Star != curr_start_tok,
        (TokType::MathSign, _) => true,
        (TokType::Sign, TokType::Alphabet) => Tok::Exclaim != curr_end_tok,
        (TokType::Sign, TokType::Number) => true,
        (TokType::Sign, TokType::String | TokType::AtSign) => {
            if !is_next_tok_nested && Tok::ByteStringValue == next_start_tok {
                return true;
            }
            if Tok::Comma == curr_start_tok && next_start_tok == Tok::AtSign {
                return true;
            }
            false
        }
        (TokType::Number, TokType::Alphabet | TokType::Star) => true,
        (TokType::Colon, _) => true,
        (_, TokType::Less) => is_bin_next,
        (TokType::Alphabet, TokType::MathSign) => {
            if next_tok_simple_content.contains('>') {
                return is_bin_next;
            }
            true
        }
        (_, TokType::MathSign) => true,
        (TokType::Less, _) => is_bin_current,
        (TokType::Amp, _) => is_bin_current,
        (_, TokType::Star) => {
            if is_bin_next {
                return true;
            }
            if is_apply_next {
                return is_to_execpt;
            }
            matches!(
                curr_start_tok,
                Tok::NumValue | Tok::NumTypedValue | Tok::Acquires | Tok::Identifier | Tok::Star
            ) || matches!(
                curr_end_tok,
                Tok::RParen
                    | Tok::Comma
                    | Tok::Slash
                    | Tok::Pipe
                    | Tok::PipePipe
                    | Tok::Return
                    | Tok::If
                    | Tok::Else
                    | Tok::EqualGreater
            )
        }

        (TokType::Star, _) => {
            if is_bin_current || is_bin_next {
                return true;
            }

            if is_apply_current {
                is_to_execpt
            } else {
                if is_next_tok_nested && next_tok_nested_kind == NestKind_::Brace {
                    return true;
                }
                if !is_next_tok_nested {
                    if matches!(
                        next_start_tok,
                        Tok::NumValue | Tok::NumTypedValue | Tok::LParen
                    ) {
                        return true;
                    }
                    if next_tok_simple_content == "vector" {
                        return false;
                    }
                }
                return false;
            }
        }

        (TokType::AtSign, TokType::Alphabet) => false,
        (TokType::Alphabet | TokType::Number | TokType::Sign, TokType::Sign) => {
            let mut result = false;
            if is_next_tok_nested && Tok::LBrace == next_start_tok {
                return true;
            }
            if !is_next_tok_nested && Tok::Slash == next_start_tok {
                return true;
            }

            if matches!(
                curr_start_tok,
                Tok::Let | Tok::Slash | Tok::If | Tok::Else | Tok::While | Tok::Comma | Tok::Return
            ) {
                return true;
            }

            if matches!(curr_start_tok, Tok::Pipe | Tok::Caret)
                && matches!(next_start_tok, Tok::LParen | Tok::LBrace)
            {
                return true;
            }

            if matches!(
                curr_end_tok,
                Tok::RParen
                    | Tok::EqualGreater
                    | Tok::EqualEqualGreater
                    | Tok::LessEqualEqualGreater
            ) && next_start_tok == Tok::LParen
            {
                return true;
            }

            if Tok::Pipe == next_start_tok {
                return true;
            }

            if next_start_tok == Tok::Exclaim {
                result = matches!(TokType::from(curr_start_tok), TokType::Alphabet)
                    || Tok::RParen == curr_end_tok;
            }

            if let Some(content) = current.simple_str() {
                if matches!(
                    content,
                    "aborts_if"
                        | "ensures"
                        | "include"
                        | "pragma"
                        | "invariant"
                        | "succeeds_if"
                        | "aborts_with"
                        | "modifies"
                        | "emits"
                        | "requires"
                        | "global"
                ) {
                    return true;
                }
                if content == "assert" && next_start_tok == Tok::Exclaim {
                    result = false;
                }

                // added in 20240430: support for loop
                // optimize in 20240510: and in(special identifier)
                if matches!(content, "for" | "in") && next_start_tok == Tok::LParen {
                    return true;
                }
            }

            result
        }
        _ => false,
    }
}

pub(crate) fn process_link_access(elements: &[TokenTree], idx: usize) -> (usize, usize) {
    tracing::trace!("process_link_access >>");
    if idx >= elements.len() - 1 {
        return (0, 0);
    }
    let mut continue_dot_cnt = 0;
    let mut index = idx;
    while index <= elements.len() - 2 {
        let t = elements.get(index).unwrap();
        if !t.simple_str().unwrap_or_default().contains('.') {
            break;
        }
        continue_dot_cnt += 1;
        index += 2;
    }
    tracing::trace!(
        "process_link_access << (continue_dot_cnt, index) = ({}, {})",
        continue_dot_cnt,
        index
    );
    (continue_dot_cnt, index - 2)
}

// Determines whether a newline is needed for the current line when trimming blank lines.
pub(crate) fn need_newline_when_trim_blank_line(current: &Tok, next: &Tok) -> bool {
    !(NO_BREAK_TOKENS_VEC.contains(current) || NO_BREAK_PAIRS_VEC.contains(&(*current, *next)))
}
