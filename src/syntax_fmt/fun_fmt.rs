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


pub fn fun_header_specifier_fmt(specifier: &str, indent_str: &String) -> String {
    eprintln!("fun_specifier_str = {:?}", specifier);

    let mut fun_specifiers_code = vec![];
    let mut lexer = Lexer::new(specifier, FileHash::empty());
    lexer.advance().unwrap();
    while lexer.peek() != Tok::EOF {
        fun_specifiers_code.push((lexer.start_loc() as u32, 
            (lexer.start_loc() + lexer.content().len()) as u32, lexer.content().clone().to_string()));
        lexer.advance().unwrap();
    }

    let mut tokens = specifier.split_whitespace();
   
    let mut fun_specifiers = vec![];
    while let Some(token) = tokens.next() {  
        fun_specifiers.push(token);
    }

    let mut fun_specifier_fmted_str = "".to_string();
    let mut found_specifier = false;
    let mut first_specifier_idx = 0;
    
    let mut current_specifier_idx = 0;
    let mut last_substr_len = 0;
    for i in 0..fun_specifiers.len() {
        if i < current_specifier_idx {
            continue;
        }
        let specifier_set = fun_specifiers[i];

        let mut parse_access_specifier_list = |
            last_substr_len: &mut usize, fun_specifiers_code: &mut Vec<(u32, u32, String)>| {
            let mut chain: Vec<_> = vec![];
            if i + 1 == fun_specifiers.len() {
                return chain;
            }
            let mut old_last_substr_len = last_substr_len.clone();
            for j in (i + 1)..fun_specifiers.len() {
                let mut this_token_is_comment = true;
                let iter_specifier = &specifier[last_substr_len.clone()..];
                if let Some(idx) = iter_specifier.find(fun_specifiers[j]) {
                    // if this token's pos not comment
                    for token_idx in 0..fun_specifiers_code.len() {
                        let token = &fun_specifiers_code[token_idx];
                        if token.0 == (idx + last_substr_len.clone()) as u32 {
                            this_token_is_comment = false;
                            break;
                        }
                    }
                    old_last_substr_len = last_substr_len.clone();
                    *last_substr_len = last_substr_len.clone() + idx + fun_specifiers[j].len();
                }
        
                if this_token_is_comment {
                    eprintln!("intern> this token is comment -- {}",  fun_specifiers[j]);
                    chain.push(fun_specifiers[j].to_string());
                    continue;
                }

                if matches!(
                    fun_specifiers[j],
                    "acquires" | "reads" | "writes" | "pure" |
                    "!acquires" | "!reads" | "!writes"
                ) {
                    current_specifier_idx = j;
                    *last_substr_len =  old_last_substr_len;
                    break;
                } else {
                    chain.push(fun_specifiers[j].to_string());
                }
            }
            eprintln!("intern> chain[{:?}] -- {:?}", i, chain);
            chain
        };

        let mut this_token_is_comment = true;
        let iter_specifier = &specifier[last_substr_len..];
        if let Some(idx) = iter_specifier.find(specifier_set) {
            // if this token's pos not comment
            for token_idx in 0..fun_specifiers_code.len() {
                let token = &fun_specifiers_code[token_idx];
                // eprintln!("iter_specifier = {}, specifier_set = {}", iter_specifier, specifier_set);
                eprintln!("token.0 = {}, idx = {}, last_substr_len = {}", token.0, idx, last_substr_len);
                if token.0 == (idx + last_substr_len) as u32 {
                    this_token_is_comment = false;
                    eprintln!("token.0 = {} === idx + last_substr_len = {}", token.0, idx + last_substr_len);
                    fun_specifiers_code.remove(token_idx);
                    break;
                }
            }
            last_substr_len =  last_substr_len + idx + specifier_set.len();
        }

        if this_token_is_comment {
            eprintln!("extern> this token is comment -- {}",  specifier_set);
            continue;
        }

        if matches!(
            specifier_set,
            "acquires" | "reads" | "writes" | "pure" |
            "!acquires" | "!reads" | "!writes"
        ) {
            if !found_specifier {
                first_specifier_idx = last_substr_len - specifier_set.len();
                found_specifier = true;
            }

            fun_specifier_fmted_str.push_str(&"\n".to_string());
            fun_specifier_fmted_str.push_str(&indent_str);
            fun_specifier_fmted_str.push_str(&specifier_set.to_string());
            if specifier_set != "pure" {
                fun_specifier_fmted_str.push_str(&" ".to_string());
                fun_specifier_fmted_str.push_str(&(parse_access_specifier_list(
                    &mut last_substr_len, &mut fun_specifiers_code).join(" ")));
            }
        }

        eprintln!("<< for loop end, last_substr_len = {}, specifier.len = {}", last_substr_len, specifier.len());
        if last_substr_len == specifier.len() {
            break;
        }
    }

    let mut ret_str = specifier[0..first_specifier_idx].to_string();
    if found_specifier {
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

pub fn add_blank_row_in_two_funs(fmt_buffer: String) -> String {
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let fun_extractor = FunExtractor::new(fmt_buffer.clone());
    let mut insert_char_nums = 0;
    let fun_nums = fun_extractor.loc_line_vec.len();
    for pre_fun_idx in 0..fun_nums {
        if pre_fun_idx == fun_nums - 1 {
            break;
        }
        let next_fun_idx = pre_fun_idx + 1;
        let fun1_end_line = fun_extractor.loc_line_vec[pre_fun_idx].1;
        let fun2_start_line = fun_extractor.loc_line_vec[next_fun_idx].0;

        let is_need_blank_row = {
            if fun1_end_line + 1 == fun2_start_line {
                true
            } else {
                let the_row_after_fun1_end = get_nth_line(buf.as_str(), (fun1_end_line + 1) as usize).unwrap_or_default();
                let trimed_prefix = the_row_after_fun1_end.trim_start();
                if trimed_prefix.len() > 0 {
                    eprintln!("trimed_prefix = {:?}", trimed_prefix);
                    true 
                } else {
                    false
                }
            }
        };
        if is_need_blank_row {
            result.insert(fun_extractor.loc_vec[pre_fun_idx].end() as usize + insert_char_nums + 1, 
                '\n');
            insert_char_nums = insert_char_nums + 1;
        }
    }

    eprintln!("result = {}", result);
    result
}

fn get_nth_line(s: &str, n: usize) -> Option<&str> {  
    s.lines().nth(n)  
}

pub fn process_block_comment_before_fun_header(fmt_buffer: String) -> String {
    let buf = fmt_buffer.clone();
    let mut result = fmt_buffer.clone();
    let fun_extractor = FunExtractor::new(fmt_buffer.clone());
    let mut insert_char_nums = 0;
    let mut fun_idx = 0;
    for (fun_start_line, _) in fun_extractor.loc_line_vec.iter() {  
        // eprintln!("fun header is {:?}", );
        let fun_header_str = get_nth_line(buf.as_str(), *fun_start_line as usize).unwrap_or_default();
        let filehash = FileHash::empty();
        let mut lexer = Lexer::new(fun_header_str, filehash);
        lexer.advance().unwrap();
        while lexer.peek() != Tok::EOF {
            // eprintln!("fun_extractor.loc_vec[fun_idx].start() = {:?}", fun_extractor.loc_vec[fun_idx].start());
            let header_prefix = &fun_header_str[0..lexer.start_loc()];
            let trimed_header_prefix = header_prefix.trim_start();
            if trimed_header_prefix.len() > 0 {
                eprintln!("header_prefix = {:?}", header_prefix);
                // result.insert(lexer.start_loc() + fun_extractor.loc_vec[fun_idx].start() as usize, '\n');

                let mut insert_str = "\n".to_string();
                if let Some(indent) = header_prefix.find(trimed_header_prefix) {
                    insert_str.push_str(" ".to_string().repeat(indent).as_str());
                }
                result.insert_str(fun_extractor.loc_vec[fun_idx].start() as usize + insert_char_nums, 
                    &insert_str);
                insert_char_nums = insert_char_nums + insert_str.len();
            }
            eprintln!("token[{:?}] = {:?}", lexer.start_loc(), lexer.content());
            break;
        }
        fun_idx = fun_idx + 1;
    }

    result
}


#[test]
fn test_rewrite_fun_header_1() {
    fun_header_specifier_fmt("acquires *(make_up_address(x))", &"    ".to_string());
    fun_header_specifier_fmt("!reads *(0x42), *(0x43)", &"    ".to_string());
    fun_header_specifier_fmt(": u32 !reads *(0x42), *(0x43)", &"    ".to_string());
    fun_header_specifier_fmt(": /*(bool, bool)*/ (bool, bool) ", &"    ".to_string());
}

#[test]
fn test_rewrite_fun_header_2() {
    fun_header_specifier_fmt(": u64 /* acquires comment1 */ acquires SomeStruct ", &"    ".to_string());
    fun_header_specifier_fmt(": u64 acquires SomeStruct/* acquires comment2 */ ", &"    ".to_string());
    fun_header_specifier_fmt(": u64 /* acquires comment3 */ acquires /* acquires comment4 */ SomeStruct /* acquires comment5 */", 
        &"    ".to_string());
    fun_header_specifier_fmt("acquires R reads R writes T, S reads G<u64> ", &"    ".to_string());
    fun_header_specifier_fmt("fun f11() !reads *(0x42) ", &"    ".to_string());
}

#[test]
fn test_add_space_line_in_two_funs_1() {
    add_blank_row_in_two_funs(
    "
    module TestFunFormat {
        
        struct SomeOtherStruct has drop {
            some_field: u64,
        } 

        /* BlockComment1 */ public fun multi_arg(p1: u64, p2: u64): u64 {
            p1 + p2
        }
        // test two fun Close together without any blank lines, and here is a InlineComment
        /* BlockComment2 */ public fun multi_arg22(p1: u64, p2: u64): u64 {
            p1 + p2
        } 
        /* BlockComment3 */ /* BlockComment4 */ fun multi_arg22(p1: u64, p2: u64): u64 {
            p1 + p2
        }
    }
    ".to_string()
    );
}

#[test]
fn test_process_block_comment_before_fun_header_1() {
    process_block_comment_before_fun_header(
        "
        module TestFunFormat {
        
            struct SomeOtherStruct has drop {
                some_field: u64,
            } 
            /* BlockComment1 */ public fun multi_arg(p1: u64, p2: u64): u64 {
                p1 + p2
            }
            // test two fun Close together without any blank lines, and here is a InlineComment
            /* BlockComment2 */ public fun multi_arg22(p1: u64, p2: u64): u64 {
                p1 + p2
            } 
            /* BlockComment3 */ /* BlockComment4 */ fun multi_arg22(p1: u64, p2: u64): u64 {
                p1 + p2
            }
        }
        ".to_string()
    );
}
