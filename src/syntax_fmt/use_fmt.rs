// Copyright © Aptos Foundation
// Copyright (c) The BitsLab.MoveBit Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tools::utils::FileLineMappingOneFile;
use commentfmt::Config;
use move_command_line_common::files::FileHash;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
use move_compiler::parser::lexer::{Lexer, Tok};
use move_compiler::parser::syntax::parse_file_string;
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use move_ir_types::location::*;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct UseExtractor {
    pub use_module_loc_vec: Vec<Loc>,
    pub use_items_loc_vec: Vec<Loc>,
    pub line_mapping: FileLineMappingOneFile,
    pub use_with_member: Vec<Use>,
}

impl UseExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut use_extractor = Self {
            use_module_loc_vec: vec![],
            use_items_loc_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
            use_with_member: vec![],
        };

        use_extractor.line_mapping.update(&fmt_buffer);
        let attrs: BTreeSet<String> = BTreeSet::new();
        let mut env = CompilationEnv::new(Flags::testing(), attrs);
        let (defs, _) = parse_file_string(&mut env, FileHash::empty(), &fmt_buffer).unwrap();

        for d in defs.iter() {
            use_extractor.collect_definition(d);
        }
        use_extractor
    }

    fn collect_module(&mut self, d: &ModuleDefinition) {
        for m in d.members.iter() {
            if let ModuleMember::Use(use_decl) = m {
                match &use_decl.use_ {
                    Use::Module(m, alias_opt) => {
                        self.use_module_loc_vec.push(m.loc);
                        self.use_items_loc_vec.push(m.loc);
                        eprintln!("use00 {}", m);
                        if let Some(alias) = alias_opt {
                            eprintln!(" as {}", alias);
                        }
                    }
                    Use::Members(m, sub_uses) => {
                        self.use_with_member.push(use_decl.use_.clone());
                        self.use_module_loc_vec.push(m.loc);
                        if sub_uses.is_empty() {
                            self.use_items_loc_vec.push(m.loc);
                        } else {
                            self.use_items_loc_vec.push(Loc::new(
                                FileHash::empty(),
                                sub_uses[0].0.loc.start(),
                                sub_uses.last().unwrap().0.loc.end(),
                            ));
                        }

                        eprintln!("use11 {}::", m);
                        for (sub_use, sub_alias) in sub_uses.iter() {
                            eprintln!("  ::{}", sub_use);
                            if let Some(sub_alias) = sub_alias {
                                eprintln!("    as {}", sub_alias);
                            }
                        }
                    }
                }
            }
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
            _ => {}
        }
    }
}

/*
// process case:
    use aptos_framework::coin::{Self,
        /* use_item before */ Coin};
// after formatted:
    use aptos_framework::coin::{
        Self,
        /* use_item before */ Coin
    };
*/
pub fn optimize_brace_of_use(fmt_buffer: String, config: Config) -> String {
    tracing::debug!("optimize_brace_of_use >>");
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let mut insert_char_nums = 0;

    let use_extractor = UseExtractor::new(fmt_buffer.clone());
    for use_with_member in use_extractor.use_with_member {
        if let Use::Members(m, sub_uses) = use_with_member {
            if let Some(item) = sub_uses.first() {
                let text = &buf[m.loc.end() as usize..item.0.loc.start() as usize];
                let mut lexer = Lexer::new(text, FileHash::empty());
                lexer.advance().unwrap();
                while lexer.peek() != Tok::EOF {
                    if lexer.peek() == Tok::LBrace {
                        let first_item_start_line = use_extractor
                            .line_mapping
                            .translate(item.0.loc.start(), item.0.loc.start())
                            .unwrap()
                            .start
                            .line;
                        let first_lbrace_start_line = use_extractor
                            .line_mapping
                            .translate(
                                m.loc.end() + lexer.start_loc() as u32,
                                m.loc.end() + lexer.start_loc() as u32,
                            )
                            .unwrap()
                            .start
                            .line;
                        let last_item_start_line = use_extractor
                            .line_mapping
                            .translate(
                                sub_uses.last().unwrap().0.loc.start(),
                                sub_uses.last().unwrap().0.loc.start(),
                            )
                            .unwrap()
                            .start
                            .line;

                        let insert_pos =
                            m.loc.end() as usize + lexer.start_loc() + insert_char_nums + 1;
                        if last_item_start_line > first_item_start_line
                            && first_item_start_line == first_lbrace_start_line
                        {
                            let mut insert_str = "\n".to_string();
                            let start_line = use_extractor
                                .line_mapping
                                .translate(m.loc.start(), m.loc.start())
                                .unwrap()
                                .start
                                .line;
                            let use_module_str =
                                buf.lines().nth(start_line as usize).unwrap_or_default();
                            let trimed_header_prefix = use_module_str.trim_start();
                            if !trimed_header_prefix.is_empty() {
                                if let Some(indent) = use_module_str.find(trimed_header_prefix) {
                                    insert_str.push_str(
                                        " ".to_string()
                                            .repeat(indent + config.indent_size())
                                            .as_str(),
                                    );
                                }
                            }

                            result.insert_str(insert_pos, &insert_str);
                            insert_char_nums += insert_str.len();
                        }
                        break;
                    }
                    lexer.advance().unwrap();
                }
            }

            if let Some(item) = sub_uses.last() {
                let item_str = &buf[item.0.loc.start() as usize..item.0.loc.end() as usize];
                tracing::debug!("item_str = {}", item_str);

                let text = &buf[item.0.loc.start() as usize..];
                let mut lexer = Lexer::new(text, FileHash::empty());
                lexer.advance().unwrap();
                while lexer.peek() != Tok::EOF {
                    if lexer.peek() == Tok::RBrace {
                        let last_item_start_line = use_extractor
                            .line_mapping
                            .translate(item.0.loc.start(), item.0.loc.start())
                            .unwrap()
                            .start
                            .line;
                        let last_rbrace_start_line = use_extractor
                            .line_mapping
                            .translate(
                                item.0.loc.start() + lexer.start_loc() as u32,
                                item.0.loc.start() + lexer.start_loc() as u32,
                            )
                            .unwrap()
                            .start
                            .line;
                        let insert_pos =
                            item.0.loc.start() as usize + lexer.start_loc() + insert_char_nums;
                        if item.0.loc.start() as usize + lexer.start_loc() > m.loc.start() as usize
                            && last_item_start_line == last_rbrace_start_line
                        {
                            let mut insert_str = "\n".to_string();
                            let start_line = use_extractor
                                .line_mapping
                                .translate(m.loc.start(), m.loc.start())
                                .unwrap()
                                .start
                                .line;
                            let use_module_str =
                                buf.lines().nth(start_line as usize).unwrap_or_default();
                            let trimed_header_prefix = use_module_str.trim_start();
                            if !trimed_header_prefix.is_empty() {
                                if let Some(indent) = use_module_str.find(trimed_header_prefix) {
                                    insert_str.push_str(" ".to_string().repeat(indent).as_str());
                                }
                            }

                            result.insert_str(insert_pos, &insert_str);
                            insert_char_nums += insert_str.len();
                        }
                        break;
                    }
                    lexer.advance().unwrap();
                }
            }
        }
    }

    result
}

#[test]
fn test_optimize_brace_of_use_1() {
    use tracing_subscriber::EnvFilter;
    std::env::set_var("MOVEFMT_LOG", "movefmt=DEBUG");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("MOVEFMT_LOG"))
        .init();
    let result = optimize_brace_of_use(
        "
        /// test_point: Multiple blank lines between use statements

        module BlockLine {
            // Multiple blank lines between statements
                use aptos_std::type_info::{
                    /* use_item before */Self, 
        
                    TypeInfo/*cmt1*/ 
                };
        
        
        
            use aptos_framework::coin::{Self, 
        
                /* use_item before */Coin/*cmt2*/ };

                      
        
        
            use aptos_framework::coin::{Self, 
        
            /* use_item before */Coin/*cmt2*/ };
         
        
            use aptos_framework::coin::{Self, 
        
            /* use_item before */Coin/*cmt2*/ };        
        
            use aptos_framework::coin::{Self, 
        
            /* use_item before */Coin/*cmt2*/ };           
        }
    "
        .to_string(),
        Config::default(),
    );

    tracing::debug!("result = {}", result);
}
