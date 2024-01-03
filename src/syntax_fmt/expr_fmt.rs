use crate::core::token_tree::{NestKind, NestKind_, Note, TokenTree};
use move_command_line_common::files::FileHash;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_ir_types::location::*;
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use std::collections::BTreeSet;
use crate::utils::FileLineMappingOneFile;
use crate::syntax::parse_file_string;

pub enum TokType {
    /// abc like token,
    Alphabet,
    /// + - ...
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
    /// &mut
    AmpMut,
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

fn get_start_tok(t: &TokenTree) -> Tok {
    match t {
        TokenTree::SimpleToken {
            content: _,
            pos: _,
            tok,
            note: _,
        } => tok.clone(),
        TokenTree::Nested {
            elements: _,
            kind,
            note: _,
        } => kind.kind.start_tok(),
    }
}

fn get_end_tok(t: &TokenTree) -> Tok {
    match t {
        TokenTree::SimpleToken {
            content: _,
            pos: _,
            tok,
            note: _,
        } => tok.clone(),
        TokenTree::Nested {
            elements: _,
            kind,
            note: _,
        } => kind.kind.end_tok(),
    }
}

fn is_to_or_except(token: &Option<&TokenTree>) -> bool {  
    match token {  
        None => false,  
        Some(TokenTree::SimpleToken { content: con, .. }) => con.as_str() == "to" || con.as_str() == "except",  
        _ => false,  
    }  
}

fn get_nth_line(s: &str, n: usize) -> Option<&str> {  
    s.lines().nth(n)
}

fn get_paren_comma_num_in_statement(elements: &Vec<TokenTree>) -> (usize, usize) {
    let mut result = (0, 0);
    for ele in elements {
        if let TokenTree::Nested {
            elements: recursive_elements,
            kind,
            note: _,
        } = ele {
            if NestKind_::ParentTheses == kind.kind {
                let recursive_result = get_paren_comma_num_in_statement(recursive_elements);
                result.0 = result.0 + recursive_result.0 + 1;
                result.1 = result.1 + recursive_result.1 + 1;
            }
        }
        if let TokenTree::SimpleToken {
            content: _,
            pos: _,
            tok,
            note: _,
        } = ele {
            if Tok::Comma == *tok {
                result.1 = result.1 + 1;
            }
        }
    }

    result
}

pub(crate) fn need_space(current: &TokenTree, next: Option<&TokenTree>) -> bool {
    if next.is_none() {
        return false;
    }

    let _is_bin_current = current
        .get_note()
        .map(|x| x == Note::BinaryOP)
        .unwrap_or_default();

    let is_bin_next = match next {
        None => false,
        Some(next_) => next_
            .get_note()
            .map(|x| x == Note::BinaryOP)
            .unwrap_or_default(),
    };
    let is_apply_current = current
        .get_note()
        .map(|x| x == Note::ApplyName)
        .unwrap_or_default();

    let is_apply_next = match next {
        None => false,
        Some(next_) => next_
            .get_note()
            .map(|x| x == Note::ApplyName)
            .unwrap_or_default(),
    };

    let is_to_execpt = is_to_or_except(&Some(current)) || is_to_or_except(&next);

    return match (
        TokType::from(get_start_tok(current)),
        TokType::from(next.map(|x| get_start_tok(x)).unwrap()),
    ) {
        (TokType::Alphabet, TokType::Alphabet) => true,
        (TokType::MathSign, _) => true,
        (TokType::Sign, TokType::Alphabet) => {
            !(Tok::Exclaim == get_end_tok(current))
        },
        (TokType::Sign, TokType::Number) => true,
        (TokType::Sign, TokType::String | TokType::AtSign) => {
            let mut result = false;
            let mut next_tok = Tok::EOF;
            next.map(|x| {
                match x {
                    TokenTree::Nested {
                        elements: _,
                        kind,
                        note: _,
                    } => {
                        next_tok = kind.kind.start_tok();
                        // if kind.kind.start_tok() == Tok::LBrace {
                        //     result = true;
                        // }
                    },
                    TokenTree::SimpleToken {
                        content: _,
                        pos: _,
                        tok,
                        note: _,
                    } => {
                        next_tok = *tok;
                        // println!("content = {:?}", content);                    
                        if Tok::ByteStringValue == *tok {
                            result = true;
                        }
                    }
                }
            });

            if Tok::Comma == get_start_tok(current) {
                if Tok::AtSign == next_tok {
                    result = true;
                }
                // println!("after Comma, result = {}, next_tok = {:?}", result, next_tok);
            }
            result
        },
        (_, TokType::MathSign) => true,
        (TokType::Alphabet, TokType::String) => true,
        (TokType::Number, TokType::Alphabet) => true,
        (_, TokType::AmpMut) => true,
        (TokType::Colon, _) => true,
        (TokType::Alphabet, TokType::Number) => true,

        (_, TokType::Less) => {
            if is_bin_next {
                true
            } else {
                false
            }
        }
        (TokType::Less, TokType::Alphabet) => true,
        (TokType::Less, _) => false,

        (_, TokType::Amp) => {
            if is_bin_next {
                true
            } else {
                false
            }
        }

        (_, TokType::Star) => {
            let result = if is_bin_next || is_apply_next {
                if is_to_execpt {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            result || Tok::NumValue == get_start_tok(current) 
                   || Tok::NumTypedValue == get_start_tok(current)
                   || Tok::Acquires == get_start_tok(current)
                   || Tok::Identifier == get_start_tok(current)
                   || Tok::RParen == get_end_tok(current)
                   || Tok::Comma == get_end_tok(current)
        }

        (TokType::Star, _) => {
            if is_bin_next {
                return true;
            }
            match next {
                None => {},
                Some(next_) => {
                    if let TokenTree::Nested{elements, ..} = next_ {
                        // if elements.len() > 0 {
                        //     elements[0]
                        //     .get_note()
                        //     .map(|x| x == Note::BinaryOP)
                        //     .unwrap_or_default()
                        // } else {
                        //     false
                        // }
                        if Tok::LParen == get_start_tok(next_) {
                            return elements.len() > 2    
                        }
                    }
                }
            }

            if is_apply_current {
                if is_to_execpt {
                    true
                } else {
                    false
                }
            } else {
                let mut result = false;
                let mut next_tok = Tok::EOF;
                next.map(|x| {
                    match x {
                        TokenTree::Nested {
                            elements: _,
                            kind,
                            note: _,
                        } => {
                            next_tok = kind.kind.start_tok();
                            if kind.kind == NestKind_::Brace {
                                result = true;
                            }
                        },
                        TokenTree::SimpleToken {
                            content,
                            pos: _,
                            tok,
                            note: _,
                        } => {
                            next_tok = *tok;
                            // println!("content = {:?}", content);
                            if Tok::NumValue == *tok 
                            || Tok::NumTypedValue == *tok
                            || Tok::LParen == *tok {
                                result = true;
                            }
                            if Tok::Identifier == *tok {
                                if content.contains("vector") {
                                    result = false;
                                } else if _is_bin_current {
                                    result = true;
                                }
                            }
                        }
                    }
                });
                result
            }
        }

        (TokType::AtSign, TokType::Alphabet) => false,
        (TokType::Alphabet | TokType::Number | TokType::Sign, TokType::Sign) => {
            let mut result = false;
            let mut next_tok = Tok::EOF;
            next.map(|x| {
                match x {
                    TokenTree::Nested {
                        elements: _,
                        kind,
                        note: _,
                    } => {
                        next_tok = kind.kind.start_tok();
                        if kind.kind.start_tok() == Tok::LBrace {
                            result = true;
                        }
                    },
                    TokenTree::SimpleToken {
                        content: _,
                        pos: _,
                        tok,
                        note: _,
                    } => {
                        next_tok = *tok;
                        // println!("content = {:?}", content);
                        if Tok::Slash == *tok || Tok::LBrace == *tok {
                            result = true;
                        }
                    }
                }
            });
            if Tok::Let == get_start_tok(current) || 
               Tok::Slash == get_start_tok(current) || 
               Tok::If == get_start_tok(current) || 
               Tok::Else == get_start_tok(current) ||
               Tok::While == get_start_tok(current) {
                result = true;
            }

            if next_tok == Tok::Exclaim {
                result = if let TokType::Alphabet = TokType::from(get_start_tok(current)) {
                    true
                } else { false } || Tok::RParen == get_end_tok(current);
            }

            if Tok::RParen == get_end_tok(current) && next_tok == Tok::LParen {
                result = true;
            }

            // println!("result = {}, next_tok = {:?}", result, next_tok);
            result
        },
        _ => false,
    };
}

pub(crate) fn judge_simple_statement(kind: &NestKind, elements: &Vec<TokenTree>) -> bool {
    if NestKind_::ParentTheses == kind.kind {
        let paren_num = get_paren_comma_num_in_statement(elements);
        eprintln!("paren_num = {:?}", paren_num);
        if paren_num.0 > 2 || paren_num.1 > 4 {
            eprintln!("elements[0] = {:?}", elements[0].simple_str());
            return false;
        }
        if paren_num.0 >= 1 && paren_num.1 >= 2 {
            eprintln!("elements[0] = {:?}", elements[0].simple_str());
            return false;
        }
    }
    true
}

#[derive(Debug, Default)]
pub struct ExpExtractor {
    pub let_if_else_block_loc_vec: Vec<Loc>,
    pub then_in_let_loc_vec: Vec<Loc>,
    pub else_in_let_loc_vec: Vec<Loc>,

    pub let_if_else_block: Vec<lsp_types::Range>,
    pub if_cond_in_let: Vec<lsp_types::Range>,
    pub then_in_let: Vec<lsp_types::Range>,
    pub else_in_let: Vec<lsp_types::Range>,
    pub line_mapping: FileLineMappingOneFile,
}

impl ExpExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut this_exp_extractor = Self {      
            let_if_else_block_loc_vec: vec![],
            then_in_let_loc_vec: vec![],
            else_in_let_loc_vec: vec![],

            let_if_else_block: vec![],
            if_cond_in_let: vec![],
            then_in_let: vec![],
            else_in_let: vec![],
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_exp_extractor.line_mapping.update(&fmt_buffer);
        let attrs: BTreeSet<String> = BTreeSet::new();    
        let mut env = CompilationEnv::new(Flags::testing(), attrs);
        let filehash = FileHash::empty();
        let (defs, _) = parse_file_string(&mut env, filehash, &fmt_buffer).unwrap();

        for d in defs.iter() {
            this_exp_extractor.collect_definition(d);
        }
        // eprintln!("this_exp_extractor = {:?}\n{:?}\n{:?}\n{:?}", 
        //     this_exp_extractor.let_if_else_block, 
        //     this_exp_extractor.if_cond_in_let, 
        //     this_exp_extractor.then_in_let, 
        //     this_exp_extractor.else_in_let
        // );

        this_exp_extractor
    }

    fn get_loc_range(&self, loc: Loc) -> lsp_types::Range {
        self.line_mapping.translate(loc.start(), loc.end()).unwrap()
    }

    fn collect_expr(&mut self, e: &Exp) {
        match &e.value {
            Exp_::IfElse(c, then_, Some(eles)) => {
                self.let_if_else_block_loc_vec.push(e.loc);
                self.then_in_let_loc_vec.push(then_.loc);
                self.else_in_let_loc_vec.push(eles.loc);

                self.let_if_else_block.push(self.get_loc_range(e.loc));
                self.if_cond_in_let.push(self.get_loc_range(c.loc));
                self.then_in_let.push(self.get_loc_range(then_.loc));
                self.else_in_let.push(self.get_loc_range(eles.loc));
            }
            _ => {}
        }
    }

    fn collect_seq_item(&mut self, s: &SequenceItem) {
        match &s.value {
            SequenceItem_::Bind(_, _, e) => {
                self.collect_expr(&e);
            }
            _ => {}
        }
    }

    fn collect_seq(&mut self, s: &Sequence) {
        for s in s.1.iter() {
            self.collect_seq_item(s);
        }
        // if let Some(t) = s.3.as_ref() {
        //     self.collect_expr(t);
        // }
    }

    fn collect_function(&mut self, d: &Function) {
        match &d.body.value {
            FunctionBody_::Defined(s) => {
                self.collect_seq(s);
            }
            FunctionBody_::Native => {}
        }
    }

    fn collect_module(&mut self, d: &ModuleDefinition) {
        for m in d.members.iter() {
            match &m {
                ModuleMember::Function(x) => self.collect_function(x),
                _ => {},
            }
        }
    }

    fn collect_script(&mut self, d: &Script) {
        self.collect_function(&d.function);
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

pub fn split_if_else_in_let_block(fmt_buffer: String) -> String {
    let mut result = "".to_string();
    let exp_extractor = ExpExtractor::new(fmt_buffer.clone());

    let process_branch = |range: lsp_types::Range| {
        let mut branch_content = "".to_string();
        let mut indent_str = "".to_string();

        let first_line = get_nth_line(&fmt_buffer, range.start.line as usize).unwrap_or_default();
        let header_prefix = &first_line[0..range.start.character as usize];
        let trimed_header_prefix = header_prefix.trim_start();
        if trimed_header_prefix.len() > 0 {
            if let Some(indent) = header_prefix.find(trimed_header_prefix) {
                indent_str.push_str(" ".to_string().repeat(indent).as_str());
            }
            indent_str.push_str(" ".to_string().repeat(4).as_str());  // increase indent
        }

        for line_idx in range.start.line..range.end.line {
            let this_line = get_nth_line(&fmt_buffer, line_idx as usize).unwrap_or_default();
            if line_idx == range.start.line {
                branch_content.push_str(&"\n".to_string());
                branch_content.push_str(&indent_str);
                branch_content.push_str(&this_line[range.start.character as usize..].trim_start());
            } else {
                if branch_content.lines().last()
                    .map(|x| x.len()).unwrap_or_default() > 50 ||
                    branch_content.lines().last().unwrap().contains("//") {
                    branch_content.push_str(&"\n".to_string());
                    branch_content.push_str(&indent_str);
                } else {
                    branch_content.push_str(&" ".to_string());
                }
                branch_content.push_str(this_line.trim_start());
            }
        }
        let end_str = get_nth_line(&fmt_buffer, range.end.line as usize).unwrap_or_default();
        if range.start.line == range.end.line {
            branch_content.push_str(&"\n".to_string());
            branch_content.push_str(&indent_str);
            branch_content.push_str(&end_str[range.start.character as usize .. range.end.character as usize].trim_start());
        } else {
            if branch_content.lines().last()
                .map(|x| x.len()).unwrap_or_default() > 50 ||
                branch_content.lines().last().unwrap().contains("//") {
                branch_content.push_str(&"\n".to_string());
                branch_content.push_str(&indent_str);
            } else {
                branch_content.push_str(&" ".to_string());
            }   
            branch_content.push_str(&end_str[0..range.end.character as usize].trim_start());
        }

        // eprintln!("branch_content = {}", branch_content);
        branch_content
    };

    let get_else_pos = |let_if_else_loc: Loc, else_branch_in_let_loc: Loc| {
        let branch_str = &fmt_buffer[0..let_if_else_loc.end() as usize];
        let mut lexer = Lexer::new(&branch_str, FileHash::empty());
        lexer.advance().unwrap();
        let mut else_in_let_vec = vec![];
        while lexer.peek() != Tok::EOF {
            if lexer.start_loc() >= else_branch_in_let_loc.start() as usize {
                break;
            }
            if let Tok::Else = lexer.peek() {
                else_in_let_vec.push((lexer.start_loc(), lexer.start_loc() + lexer.content().len()));
            }
            lexer.advance().unwrap();
        }

        let ret_pos = else_in_let_vec.last().unwrap();
        (ret_pos.0, ret_pos.1)
    };

    let mut need_split_idx = vec![];
    for let_if_else_idx in 0..exp_extractor.let_if_else_block.len() {
        let start = exp_extractor.let_if_else_block[let_if_else_idx].start;
        let end = exp_extractor.let_if_else_block[let_if_else_idx].end;
        if end.line == start.line && end.character - start.character < 70 {
            continue;
        }
        let then_str = process_branch(exp_extractor.then_in_let[let_if_else_idx]);
        if then_str.contains("{") && then_str.contains("}") {
            // note: maybe comment has "{" or "}"
            continue;
        }
        need_split_idx.push(let_if_else_idx);
    }

    let mut last_pos = (0, 0);
    for idx in need_split_idx {
        let then_str = process_branch(exp_extractor.then_in_let[idx]);
        let else_str = process_branch(exp_extractor.else_in_let[idx]);
        let if_cond_range = exp_extractor.if_cond_in_let[idx];
        let cond_end_line = get_nth_line(&fmt_buffer, if_cond_range.end.line as usize).unwrap_or_default();

        // append line[last_line, if ()]
        // eg:
        /*
        // line_x -- last_line
        // ...
        // line_x_plus_n
        if (...)
            ...
        else
            ...
        */
        for idx in last_pos.0..if_cond_range.end.line as usize {
            result.push_str(&get_nth_line(&fmt_buffer, idx).unwrap_or_default()[last_pos.1..]);
            result.push_str(&"\n".to_string());
            last_pos = (idx + 1, 0);
        }
        result.push_str(&cond_end_line[0..(if_cond_range.end.character) as usize]);

        // append line[if (), then)
        // eg:
        /*
        if (...) /* maybe there has comment1 */ ...
        /* maybe there has 
        comment2 */ else /* maybe there has 
        comment3 */ 
            ...
        */
        if if_cond_range.end.line == exp_extractor.then_in_let[idx].start.line {
            result.push_str(&cond_end_line[if_cond_range.end.character as usize..exp_extractor.then_in_let[idx].start.character as usize]);
        }
        result.push_str(&then_str);

        // there maybe comment before else
        let else_pos = get_else_pos(exp_extractor.let_if_else_block_loc_vec[idx],
            exp_extractor.else_in_let_loc_vec[idx]);
        result.push_str(&fmt_buffer[exp_extractor.then_in_let_loc_vec[idx].end() as usize..else_pos.0]);
        
        // append "\n$indent_str$else"
        let mut indent_str = "".to_string();
        let header_prefix = &cond_end_line[0..if_cond_range.end.character as usize];
        let trimed_header_prefix = header_prefix.trim_start();
        if trimed_header_prefix.len() > 0 {
            if let Some(indent) = header_prefix.find(trimed_header_prefix) {
                indent_str.push_str(" ".to_string().repeat(indent).as_str());
            }
        }
        result.push_str(&"\n".to_string());
        result.push_str(&indent_str);
        result.push_str(&"else".to_string());

        // there maybe comment after else
        result.push_str(&fmt_buffer[else_pos.1..exp_extractor.else_in_let_loc_vec[idx].start() as usize]);
        // append else branch content
        result.push_str(&else_str);

        last_pos = (exp_extractor.else_in_let[idx].end.line as usize, exp_extractor.else_in_let[idx].end.character as usize);
    }
    // eprintln!("last_pos = \n{:?}", last_pos);
    for idx in last_pos.0..fmt_buffer.lines().count() as usize {
        result.push_str(&get_nth_line(&fmt_buffer, idx).unwrap_or_default()[last_pos.1..]);
        if idx != fmt_buffer.lines().count() - 1 {
            result.push_str(&"\n".to_string());
        }
        last_pos = (idx + 1, 0);
    }
    result
}

#[test]
fn test_split_if_else_in_let_block_1() {
    split_if_else_in_let_block("
    script {fun main() {  
        // Initialize variable y with value 100  
        let y: u64 = 100;  
        // If y is less than or equal to 10, increment y by 1, otherwise set y to 10  
        let z = if (y /*condition check*/ <= /*less than or equal to*/ 10) y = /*assignment*/ y + /*increment*/ 1 else y = /*assignment*/ 10;  
    }}
    ".to_string());
}

#[test]
fn test_split_if_else_in_let_block_2() {
    split_if_else_in_let_block(
"
script {
    fun main() {
        // Initialize variable y with value 100
        let y: u64 = 100;
        // If y is less than or equal to 10, increment y by 1, otherwise set y to 10
        let z = if (y /*condition check*/ <= /*less than or equal to*/ 10) y = /*assignment*/ y +
        /*increment*/ 1 else y = /*assignment*/ 10;

        // ----------------------------------
        // Initialize variable y with value 100
        let y: u64 = 100;
        // If y is less than or equal to 10, increment y by 1, otherwise set y to 10
        let z = if (y /*condition check*/ <= /*less than or equal to*/ 10) y = /*assignment*/ y + 2 +
        /*increment*/ 1 else y = /*assignment*/ 10;
    }
}
    ".to_string());
}

#[test]
fn test_split_if_else_in_let_block_3() {
    split_if_else_in_let_block(
"
script {
    fun main() {
        // Initialize variable y with value 100
        let y: u64 = 100;
        // If y is less than or equal to 10, increment y by 1, otherwise set y to 10
        let z = if (y /*condition check*/ <= /*less than or equal to*/ 10) y = /*assignment*/ y +
        /*incre
        xxxxxxxxxxxx
        ment*/ 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15 + 16 +
        17 + 18 + 19 + 20 + 21 + 22 + 23 + 24 + 25 + 26 + 27 + 28 + 29 + 30 + 31 + 32 + 33 +
        34 + 35 /*before else comment*/ else /*after else comment*/ y = /*assignment*/ 10;
    }
}
".to_string());
}

#[test]
fn test_split_if_else_in_let_block_4() {
    split_if_else_in_let_block(
"
script {
    fun main() {  
        let y: u64 = 100; // Define an unsigned 64-bit integer variable y and assign it a value of 100  
        let /*comment*/z/*comment*/ = if/*comment*/ (/*comment*/y <= /*comment*/10/*comment*/) { // If y is less than or equal to 10  
            y = y + 1; // Increment y by 1  
        }/*comment*/ else /*comment*/{  
            y = 10; // Otherwise, set y to 10  
        };  
    }
    }
".to_string());
}

#[test]
fn test_split_if_else_in_let_block_5() {
    split_if_else_in_let_block(
"
script {
    fun main() {  
        let y: u64 = 100; // Define an unsigned 64-bit integer variable y and assign it a value of 100  
        let /*comment*/z/*comment*/ = if/*comment*/ (/*comment*/y <= /*comment*/10/*comment*/) { // If y is less than or equal to 10  
            y = y + 1; // Increment y by 1  
        }/*comment*/ else /*comment*/{  
            y = 10; // Otherwise, set y to 10  
        };  

        // ----------
        let y: u64 = 100; // Define an unsigned 64-bit integer variable y and assign it a value of 100  
        let /*comment*/z/*comment*/ = if/*comment*/ (/*comment*/y <= /*comment*/10/*comment*/) { // If y is less than or equal to 10  
            y = y + 1 + 2; // Increment y by 1  
        }/*comment*/ else /*comment*/{  
            y = 10; // Otherwise, set y to 10  
        };  
    }
    }
".to_string());
}
