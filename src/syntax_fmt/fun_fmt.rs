// use std::cell::RefCell;
// use core::panic;
// use std::cmp::Ordering;
// use std::collections::HashSet;
// use std::result::Result::*;
use move_command_line_common::files::FileHash;
// use move_compiler::diagnostics::Diagnostics;
// use move_compiler::parser::lexer::{Lexer, Tok};
use move_ir_types::location::*;
use move_compiler::shared::CompilationEnv;
use move_compiler::Flags;
use move_compiler::parser::ast::Definition;
use move_compiler::parser::ast::*;
// use move_compiler::parser::lexer::{Lexer, Tok};
// use move_compiler::shared::Identifier;
// use std::cell::Cell;
use std::collections::BTreeSet;
// use crate::core::token_tree::{
//     Comment, CommentExtrator, CommentKind, Delimiter, NestKind_, Note, TokenTree,
// };
use crate::utils::FileLineMappingOneFile;
use crate::syntax::parse_file_string;

// struct CodeBlock<T> {  
//     content: &'static str,  
//     kind: &'static str,  
//     acquire: &'static str,  
//     read: &'static str,  
//     write: &'static str,  
//     other: T,  
// }  

// Parse an access specifier list:
//      AccessSpecifierList = <AccessSpecifier> ( "," <AccessSpecifier> )* ","?
// fn parse_access_specifier_list(
//     lexer: &mut Lexer<'_>,
// ) -> Vec<String> {
//     let mut chain = vec![];
//     loop {
//         chain.push(lexer.content().to_string());
//         lexer.advance().unwrap();
//         match lexer.peek() {
//             Tok::Acquires | Tok::EOF => break,
//             Tok::Identifier if lexer.content() == "reads" => break,
//             Tok::Identifier if lexer.content() == "writes" => break,
//             Tok::Identifier if lexer.content() == "pure" => break,
//             _ => continue,
//         }
//     }
//     chain
// }

// fn parse_function_decl(
//     lexer: &mut Lexer<'_>
// ) -> Vec<Vec<String>> {
//     let mut fun_specifiers = vec![];
//     let mut acquires_specifiers = vec![];
//     let mut reads_specifiers = vec![];
//     let mut writes_specifiers = vec![];
//     let mut pure_specifiers = vec![];
//     loop {
//         let negated = if lexer.peek() == Tok::Exclaim {
//             lexer.advance().unwrap();
//             true
//         } else {
//             false
//         };
//         match lexer.peek() {
//             Tok::Acquires => {
//                 let key_str: String =  if negated { "!acquires".to_string() } else { "acquires".to_string() };
//                 acquires_specifiers.push(key_str);
//                 lexer.advance().unwrap();
//                 acquires_specifiers.extend(parse_access_specifier_list(lexer))
//             },
//             Tok::Identifier if lexer.content() == "reads" => {
//                 let key_str: String =  if negated { "!reads".to_string() } else { "reads".to_string() };
//                 reads_specifiers.push(key_str);
//                 lexer.advance().unwrap();
//                 reads_specifiers.extend(parse_access_specifier_list(lexer))
//             },
//             Tok::Identifier if lexer.content() == "writes" => {
//                 let key_str: String =  if negated { "!writes".to_string() } else { "writes".to_string() };
//                 writes_specifiers.push(key_str);
//                 lexer.advance().unwrap();
//                 writes_specifiers.extend(parse_access_specifier_list(lexer))
//             },
//             Tok::Identifier if lexer.content() == "pure" => {
//                 pure_specifiers.push(lexer.content().to_string());
//                 lexer.advance().unwrap();
//             },
//             Tok::EOF => break,
//             _ => lexer.advance().unwrap()
//         }
//     }
//     fun_specifiers.push(acquires_specifiers);
//     fun_specifiers.push(reads_specifiers);
//     fun_specifiers.push(writes_specifiers);
//     fun_specifiers.push(pure_specifiers);
//     fun_specifiers
// }

pub fn fun_header_specifier_fmt(specifier: &str, indent_str: &String) -> String {
    eprintln!("fun_specifier_str = {:?}", specifier);
    let mut tokens = specifier.split_whitespace();
   
    let mut fun_specifiers = vec![];
    while let Some(token) = tokens.next() {  
        fun_specifiers.push(token);
    }

    let mut fun_specifier_fmted_str = "".to_string();
    let mut found_specifier = false;
    let mut first_specifier_idx = 0;
    for mut i in 0..fun_specifiers.len() {
        let specifier_set = fun_specifiers[i];
        let mut parse_access_specifier_list = || {
            let mut chain: Vec<_> = vec![];
            if i + 1 == fun_specifiers.len() {
                return chain;
            }
            for j in (i + 1)..fun_specifiers.len() {
                if matches!(
                    fun_specifiers[j],
                    "acquires" | "reads" | "writes" | "pure" |
                    "!acquires" | "!reads" | "!writes"
                ) {
                    i = j - 1;
                    break;
                } else {
                    chain.push(fun_specifiers[j].to_string());
                }
            }
            chain
        };

        if matches!(
            specifier_set,
            "acquires" | "reads" | "writes" | "pure" |
            "!acquires" | "!reads" | "!writes"
        ) {
            if !found_specifier {
                if let Some(str_idx) = specifier.find(specifier_set) {
                    first_specifier_idx = str_idx;
                    found_specifier = true;
                }
            }

            fun_specifier_fmted_str.push_str(&"\n".to_string());
            fun_specifier_fmted_str.push_str(&indent_str);
            fun_specifier_fmted_str.push_str(&specifier_set.to_string());
            if specifier_set != "pure" {
                fun_specifier_fmted_str.push_str(&" ".to_string());
                fun_specifier_fmted_str.push_str(&(parse_access_specifier_list().join(" ")));
            }
        }
    }

    let mut ret_str = specifier[0..first_specifier_idx].to_string();
    if first_specifier_idx > 0 {
        ret_str.push_str(fun_specifier_fmted_str.as_str());
        ret_str.push_str(&" ".to_string());
        eprintln!("fun_specifier_fmted_str = --------------{}", ret_str);
    } else {
        ret_str = specifier.to_string();
    }
    ret_str
}

pub fn add_space_line_in_two_fun(fmt_buffer: String) -> String {
    use regex::Regex;
    let re = Regex::new(r"}\s*fun").unwrap();
    let mut ret_fmt_buffer = fmt_buffer.clone();
    let text = fmt_buffer.clone();
    for cap in re.clone().captures_iter(text.as_str()) {  
        let cap = cap[0].to_string();
        if cap.chars().filter(|c| *c == '\n').count() == 1 {
            eprintln!("cap = {:?}", cap);
            match fmt_buffer.find(&cap) {
                Some(idx) => {
                    ret_fmt_buffer.insert(idx + 2, '\n');
                    eprintln!("after insert, cap = {:?}", &ret_fmt_buffer[idx..idx+cap.len()]);
                },
                _ => {},
            }
        } else {
            eprintln!("cap = {:?}", cap);
        }
    }
    ret_fmt_buffer
}

#[derive(Debug, Default)]
pub struct FunExtractor {
    pub loc_vec: Vec<Loc>,
    pub loc_line_vec: Vec<(u32, u32)>,
    pub line_mapping: FileLineMappingOneFile,
}

impl FunExtractor {
    pub fn new(fmt_buffer: String) -> Self {
        let mut this_fun_extractor = Self {      
            loc_vec: vec![],
            loc_line_vec: vec![],
            line_mapping: FileLineMappingOneFile::default(),
        };

        this_fun_extractor.line_mapping.update(&fmt_buffer);
        let attrs: BTreeSet<String> = BTreeSet::new();    
        let mut env = CompilationEnv::new(Flags::testing(), attrs);
        let filehash = FileHash::empty();
        let (defs, _) = parse_file_string(&mut env, filehash, &fmt_buffer).unwrap();
    
        
        for d in defs.iter() {
            this_fun_extractor.collect_definition(d);
        }

        this_fun_extractor
    }

    fn collect_function(&mut self, d: &Function) {
        match &d.body.value {
            FunctionBody_::Defined(..) => {
                let start_line = self.line_mapping.translate(d.loc.start(), d.loc.start()).unwrap().start.line;
                let end_line = self.line_mapping.translate(d.loc.end(), d.loc.end()).unwrap().start.line;
                self.loc_vec.push(d.loc);
                self.loc_line_vec.push((start_line, end_line));
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

pub fn add_space_line_in_two_funs(fmt_buffer: String) {
    let fun_extractor = FunExtractor::new(fmt_buffer);
    eprintln!("loc_line_vec = {:?}", fun_extractor.loc_line_vec);
}

// #[test]
// fn test_rewrite_fun_header_1() {
//     let specifier = "acquires R reads R writes T,\n    S reads G<u64>";
//     let mut lexer = Lexer::new(specifier, FileHash::empty());
//     lexer.advance().unwrap();
//     let specififers = parse_function_decl(&mut lexer);
//     if specififers.len() == 4 {
//         let mut acquires_specifiers = "".to_string();
//         for acquires in &specififers[0] {
//             acquires_specifiers = acquires_specifiers.to_owned() + &acquires + &" ".to_string();
//         }
//         eprintln!("acquires_specifiers = {}", acquires_specifiers);

//         let mut reads_specifiers = "".to_string();
//         for reads in &specififers[1] {
//             reads_specifiers = reads_specifiers.to_owned() + &reads + &" ".to_string();
//         }
//         eprintln!("reads_specifiers = {}", reads_specifiers);
//         eprintln!("writes_specifiers = {:?}", specififers[2]);
//         eprintln!("pure_specifiers = {:?}", specififers[3]);
//     }
//     // let code = CodeBlock {  
//     //     content: "f_multiple",  
//     //     kind: "",  
//     //     acquire: "R",  
//     //     read: "R",  
//     //     write: "T, S",  
//     //     other: "reads G<u64>",  
//     // };  
//     // println!("{}", code);  
// }


#[test]
fn test_rewrite_fun_header_2() {
    fun_header_specifier_fmt("acquires *(make_up_address(x))", &"    ".to_string());
    fun_header_specifier_fmt("!reads *(0x42), *(0x43)", &"    ".to_string());
    fun_header_specifier_fmt(": u32 !reads *(0x42), *(0x43)", &"    ".to_string());
    fun_header_specifier_fmt(": /*(bool, bool)*/ (bool, bool) ", &"    ".to_string());
}


#[test]
fn test_add_space_line_in_two_funs_1() {
    // add_space_line_in_two_funs(
    //     "
    //   }
    // // test two fun Close together without any blank lines, and here is a InlineComment
    // public fun 
    // ".to_string()
    // );

    add_space_line_in_two_funs(
    "
    module TestFunFormat {
    
        struct SomeOtherStruct has drop {
            some_field: u64,
        } 
        
        // test two fun Close together without any blank lines
        public fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(v: u64): SomeOtherStruct {
            SomeOtherStruct {some_field: v}
        } 
        public fun multi_arg(p1: u64, p2: u64): u64 {
            p1 + p2
        }
        // test two fun Close together without any blank lines, and here is a InlineComment
        public fun multi_arg22(p1: u64, p2: u64): u64 {
            p1 + p2
        } 
        /* test two fun Close together without any blank lines, and here is a BlockComment */ fun multi_arg22(p1: u64, p2: u64): u64 {
            p1 + p2
        }
    }
    ".to_string()
    );
}