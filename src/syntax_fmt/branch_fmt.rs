use move_command_line_common::files::FileHash;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_ir_types::location::*;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use crate::tools::utils::FileLineMappingOneFile;
use commentfmt::Config;
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(Debug)]
pub struct LetIfElseBlock {
    pub let_if_else_block_loc_vec: Vec<Loc>,
    pub then_in_let_loc_vec: Vec<Loc>,
    pub else_in_let_loc_vec: Vec<Loc>,

    pub let_if_else_block: Vec<lsp_types::Range>,
    pub if_cond_in_let: Vec<lsp_types::Range>,
    pub then_in_let: Vec<lsp_types::Range>,
    pub else_in_let: Vec<lsp_types::Range>,
}

#[derive(Debug)]
pub struct ComIfElseBlock {
    pub if_else_blk_loc_vec: Vec<Loc>,
    pub then_loc_vec: Vec<Loc>,
    pub else_loc_vec: Vec<Loc>,
}


#[derive(Debug)]
pub enum BranchKind {
    LetIfElse,
    ComIfElse,
}

#[derive(Debug)]
pub struct BranchExtractor {
    pub let_if_else: LetIfElseBlock,
    pub com_if_else: ComIfElseBlock,
    pub cur_kind: BranchKind,
    pub source: String,
    pub line_mapping: FileLineMappingOneFile,
    pub added_new_line_branch: RefCell<HashMap<ByteIndex, bool>>,
}

#[inline(always)]
fn get_nth_line(s: &str, n: usize) -> Option<&str> {  
    s.lines().nth(n)
}

impl BranchExtractor {
    pub fn new(fmt_buffer: String, cur_kind: BranchKind) -> Self {

        let let_if_else = LetIfElseBlock {
            let_if_else_block_loc_vec: vec![],
            then_in_let_loc_vec: vec![],
            else_in_let_loc_vec: vec![],

            let_if_else_block: vec![],
            if_cond_in_let: vec![],
            then_in_let: vec![],
            else_in_let: vec![],
        };
        let com_if_else = ComIfElseBlock {
            if_else_blk_loc_vec: vec![],
            then_loc_vec: vec![],
            else_loc_vec: vec![],
        };
        let mut this_exp_extractor = Self {
            let_if_else,
            com_if_else,
            source: fmt_buffer.clone(),
            line_mapping: FileLineMappingOneFile::default(),
            cur_kind,
            added_new_line_branch: HashMap::default().into(),
        };

        this_exp_extractor.line_mapping.update(&fmt_buffer.clone());
        this_exp_extractor
    }

    fn get_loc_range(&self, loc: Loc) -> lsp_types::Range {
        self.line_mapping.translate(loc.start(), loc.end()).unwrap()
    }

    fn collect_expr(&mut self, e: &Exp) {
        if let Exp_::IfElse(c, then_, Some(eles)) = &e.value {
            if let BranchKind::LetIfElse = self.cur_kind {
                self.let_if_else.let_if_else_block_loc_vec.push(e.loc);
                self.let_if_else.then_in_let_loc_vec.push(then_.loc);
                self.let_if_else.else_in_let_loc_vec.push(eles.loc);

                self.let_if_else.let_if_else_block.push(self.get_loc_range(e.loc));
                self.let_if_else.if_cond_in_let.push(self.get_loc_range(c.loc));
                self.let_if_else.then_in_let.push(self.get_loc_range(then_.loc));
                self.let_if_else.else_in_let.push(self.get_loc_range(eles.loc));
            }
        }
        if let Exp_::IfElse(_, then_, eles_opt) = &e.value {
            if let BranchKind::ComIfElse = self.cur_kind {
                self.com_if_else.if_else_blk_loc_vec.push(e.loc);
                self.com_if_else.then_loc_vec.push(then_.loc);
                if let Some(el) = eles_opt {
                    self.com_if_else.else_loc_vec.push(el.loc);
                } else {
                    self.com_if_else.else_loc_vec.push(then_.loc);
                }
            }
        }
    }

    fn collect_seq_item(&mut self, s: &SequenceItem) {
        if let BranchKind::LetIfElse = self.cur_kind {
            if let SequenceItem_::Bind(_, _, e) = &s.value {
                self.collect_expr(&e);
            }
        }
        if let BranchKind::ComIfElse = self.cur_kind {
            if let SequenceItem_::Seq(e) = &s.value {
                self.collect_expr(&e);
            }
        }
    }

    fn collect_seq(&mut self, s: &Sequence) {
        for s in s.1.iter() {
            self.collect_seq_item(s);
        }
        if let BranchKind::ComIfElse = self.cur_kind {
            if let Some(t) = s.3.as_ref() {
                self.collect_expr(t);
            }
        }
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

impl BranchExtractor {
    pub fn preprocess(&mut self, module_defs: Vec<Definition>) {
        for d in module_defs.iter() {
            self.collect_definition(d);
        }
    }

    pub fn split_if_else_in_let_block(&self) -> String {
        let mut result = "".to_string();
        let process_branch = |range: lsp_types::Range| {
            let mut branch_content = "".to_string();
            let mut indent_str = "".to_string();

            let first_line = get_nth_line(&self.source, range.start.line as usize).unwrap_or_default();
            let header_prefix = &first_line[0..range.start.character as usize];
            let trimed_header_prefix = header_prefix.trim_start();
            if trimed_header_prefix.len() > 0 {
                if let Some(indent) = header_prefix.find(trimed_header_prefix) {
                    indent_str.push_str(" ".to_string().repeat(indent).as_str());
                }
                indent_str.push_str(" ".to_string().repeat(4).as_str());  // increase indent
            }

            for line_idx in range.start.line..range.end.line {
                let this_line = get_nth_line(&self.source, line_idx as usize).unwrap_or_default();
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
            let end_str = get_nth_line(&self.source, range.end.line as usize).unwrap_or_default();
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

            // tracing::debug!("branch_content = {}", branch_content);
            branch_content
        };

        let get_else_pos = |let_if_else_loc: Loc, else_branch_in_let_loc: Loc| {
            let branch_str = &self.source[0..let_if_else_loc.end() as usize];
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
        for let_if_else_idx in 0..self.let_if_else.let_if_else_block.len() {
            let start = self.let_if_else.let_if_else_block[let_if_else_idx].start;
            let end = self.let_if_else.let_if_else_block[let_if_else_idx].end;
            if end.line == start.line && end.character - start.character < 70 {
                continue;
            }
            let then_str = process_branch(self.let_if_else.then_in_let[let_if_else_idx]);
            if then_str.contains("{") && then_str.contains("}") {
                // note: maybe comment has "{" or "}"
                continue;
            }
            need_split_idx.push(let_if_else_idx);
        }

        let mut last_pos = (0, 0);
        for idx in need_split_idx {
            let then_str = process_branch(self.let_if_else.then_in_let[idx]);
            let else_str = process_branch(self.let_if_else.else_in_let[idx]);
            let if_cond_range = self.let_if_else.if_cond_in_let[idx];
            let cond_end_line = get_nth_line(&self.source, if_cond_range.end.line as usize).unwrap_or_default();

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
                result.push_str(&get_nth_line(&self.source, idx).unwrap_or_default()[last_pos.1..]);
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
            if if_cond_range.end.line == self.let_if_else.then_in_let[idx].start.line {
                result.push_str(&cond_end_line[if_cond_range.end.character as usize..self.let_if_else.then_in_let[idx].start.character as usize]);
            }
            result.push_str(&then_str);

            // there maybe comment before else
            let else_pos = get_else_pos(self.let_if_else.let_if_else_block_loc_vec[idx],
                self.let_if_else.else_in_let_loc_vec[idx]);
            result.push_str(&self.source[self.let_if_else.then_in_let_loc_vec[idx].end() as usize..else_pos.0]);
            
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
            result.push_str(&self.source[else_pos.1..self.let_if_else.else_in_let_loc_vec[idx].start() as usize]);
            // append else branch content
            result.push_str(&else_str);

            last_pos = (self.let_if_else.else_in_let[idx].end.line as usize, self.let_if_else.else_in_let[idx].end.character as usize);
        }
        // tracing::debug!("last_pos = \n{:?}", last_pos);
        for idx in last_pos.0..self.source.lines().count() as usize {
            result.push_str(&get_nth_line(&self.source, idx).unwrap_or_default()[last_pos.1..]);
            if idx != self.source.lines().count() - 1 {
                result.push_str(&"\n".to_string());
            }
            last_pos = (idx + 1, 0);
        }
        result
    }

    pub fn need_new_line_in_then_without_brace(&self, cur_line: String, then_start_pos: ByteIndex, config: Config) -> bool {
        for then_loc in &self.com_if_else.then_loc_vec {
            if then_loc.start() == then_start_pos {
                let has_added = cur_line.len() as u32 + then_loc.end() - then_loc.start() > config.max_width() as u32;
                self.added_new_line_branch.borrow_mut().insert(then_start_pos, has_added);
                return has_added;
            }
        }
        false
    }

    pub fn added_new_line_in_then_without_brace(&self, then_end_pos: ByteIndex) -> bool {
        for then_loc in &self.com_if_else.then_loc_vec {
            if then_loc.end() == then_end_pos && self.added_new_line_branch.borrow().contains_key(&then_loc.start()){
                return self.added_new_line_branch.borrow()[&then_loc.start()];
            }
        }
        false
    }
}
