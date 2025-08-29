// ---------- 1. Tok def ----------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    LParen,
    RParen,
    Less,
    Greater,
    Comma,
    Dot,
    EOF,
}

// ---------- 2. AST ----------
#[derive(Debug, PartialEq)]
pub enum ChainMember {
    Field(String),
    Call(String, Vec<Vec<ChainMember>>), // function name + argument list (each arg is itself a chain)
    Raw(String), 
}

// ---------- 1) 打印“链式成员”(a.b.c) ----------
fn fmt_chain(chain: &[ChainMember]) -> String {
    chain
        .iter()
        .map(|m| match m {
            ChainMember::Field(name) => name.clone(),
            ChainMember::Call(name, args) => {
                let args_str: Vec<String> =
                    args.iter().map(|arg| fmt_arg(arg)).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            ChainMember::Raw(_) => todo!()
        })
        .collect::<Vec<_>>()
        .join(".")
}
// fn fmt_chain(chain: &[ChainMember]) -> String {
//     chain
//         .iter()
//         .map(|m| match m {
//             ChainMember::Field(name) => name.clone(),
//             ChainMember::Call(name, args) => {
//                 format!("{}({})", name, args.join(", "))
//             }
//             ChainMember::Raw(text) => text.clone(),
//         })
//         .collect::<Vec<_>>()
//         .join(".")
// }

// ---------- 2) 打印“实参”(普通表达式) ----------
fn fmt_arg(tokens: &[ChainMember]) -> String {
    tokens
        .iter()
        .map(|m| match m {
            ChainMember::Field(name) => name.clone(),
            ChainMember::Call(name, args) => {
                let args_str: Vec<String> =
                    args.iter().map(|arg| fmt_arg(arg)).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            _ => "".to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------- 3) 测试宏 ----------
macro_rules! pp {
    ($e:expr) => {{
        println!("{}", fmt_chain($e));
    }};
}


// /// Pretty print a single ChainMember
// fn fmt_chain_member(m: &ChainMember) -> String {
//     match m {
//         ChainMember::Field(name) => name.clone(),
//         ChainMember::Call(name, args) => {
//             let mut s = format!("{}(", name);
//             for (i, arg) in args.iter().enumerate() {
//                 if i > 0 {
//                     s.push_str(", ");
//                 }
//                 s.push_str(&fmt_chain_list(arg));  // 这里不应该调用 fmt_chain_list， 否则就会出现 x.+.y.*.z. 这种
//                 // 要有一个专门合理打印参数的函数
//             }
//             s.push(')');
//             s
//         }
//     }
// }

// /// Pretty print a list of ChainMember (one argument or whole chain)
// fn fmt_chain_list(list: &[ChainMember]) -> String {
//     list.iter()
//         .map(fmt_chain_member)
//         .collect::<Vec<_>>()
//         .join(".")
// }

// /// Convenience macro for tests: println!("{}", pretty_print(&args[1]));
// macro_rules! pp {
//     ($e:expr) => {
//         println!("{}", fmt_chain_list($e))
//     };
// }

// ---------- 3. Parser ----------
pub fn parse_dot_chain(tokens: Vec<Tok>) -> Option<Vec<ChainMember>> {
    let mut p = Parser::new(tokens);
    p.parse_chain().and_then(|_| {
        if matches!(p.current(), Tok::EOF) {
            Some(p.result)
        } else {
            None
        }
    })
}

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
    result: Vec<ChainMember>,
}

impl Parser {
    fn new(mut t: Vec<Tok>) -> Self {
        t.push(Tok::EOF);
        Self {
            tokens: t,
            pos: 0,
            result: Vec::new(),
        }
    }
    fn current(&self) -> &Tok {
        &self.tokens[self.pos]
    }
    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn parse_chain(&mut self) -> Option<()> {
        let first = match self.current() {
            Tok::Ident(name) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => return None,
        };
        self.result.push(ChainMember::Field(first));

        while matches!(self.current(), Tok::Dot) {
            self.advance();
            self.parse_postfix()?;
        }
        Some(())
    }

    fn parse_postfix(&mut self) -> Option<()> {
        let name = match self.current() {
            Tok::Ident(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return None,
        };

        // Treat as Call if followed by '<' or '('
        let is_call = matches!(self.current(), Tok::Less | Tok::LParen);
        let args = self.parse_call_args()?; // consumes <>() or (); returns empty vec if none
        if is_call || !args.is_empty() {
            self.result.push(ChainMember::Call(name, args));
        } else {
            self.result.push(ChainMember::Field(name));
        }
        Some(())
    }
    fn parse_call_args(&mut self) -> Option<Vec<Vec<ChainMember>>> {
        // 1. 跳过可选的 <...>
        if matches!(self.current(), Tok::Less) {
            self.advance();
            let mut depth = 1;
            while depth > 0 && !matches!(self.current(), Tok::EOF) {
                match self.current() {
                    Tok::Less => depth += 1,
                    Tok::Greater => depth -= 1,
                    _ => {}
                }
                self.advance();
            }
            if depth != 0 {
                return None;
            }
        }

        if !matches!(self.current(), Tok::LParen) {
            return Some(Vec::new());
        }
        self.advance();

        let mut all = Vec::new();
        let mut arg_start = self.pos;

        loop {
            match self.current() {
                Tok::RParen => {
                    // 收集最后一个实参（如果有）
                    if arg_start < self.pos {
                        let slice = &self.tokens[arg_start..self.pos];
                        all.push(slice_to_chain(slice));
                    }
                    self.advance();
                    return Some(all);
                }
                Tok::Comma => {
                    // 收集当前实参
                    let slice = &self.tokens[arg_start..self.pos];
                    all.push(slice_to_chain(slice));
                    self.advance();
                    arg_start = self.pos;
                }
                Tok::EOF => return None,
                _ => {
                    // 2. 遇到嵌套括号，需要计数
                    let mut depth = 0;
                    loop {
                        match self.current() {
                            Tok::LParen | Tok::Less => depth += 1,
                            Tok::RParen | Tok::Greater => {
                                depth -= 1;
                                if depth < 0 {
                                    break; // 外层右括号
                                }
                            }
                            Tok::Comma if depth == 0 => break,
                            Tok::EOF => return None,
                            _ => {}
                        }
                        self.advance();
                    }
                }
            }
        }
    }
}

/// 把一段 token slice 简单映射成 ChainMember::Field 列表
fn slice_to_chain(slice: &[Tok]) -> Vec<ChainMember> {
    slice
        .iter()
        .filter_map(|t| match t {
            Tok::Ident(s) => Some(ChainMember::Field(s.clone())),
            Tok::LParen => Some(ChainMember::Field("(".to_string())),
            Tok::RParen => Some(ChainMember::Field(")".to_string())),
            Tok::Less   => Some(ChainMember::Field("<".to_string())),
            Tok::Greater=> Some(ChainMember::Field(">".to_string())),
            Tok::Comma  => Some(ChainMember::Field(".".to_string())),
            _ => None,
        })
        .collect()
}

/// 把一段 token slice 直接拼成字符串
fn slice_to_raw(slice: &[Tok]) -> String {
    slice
        .iter()
        .filter_map(|t| match t {
            Tok::Ident(s) => Some(s.as_str()),
            Tok::LParen => Some("("),
            Tok::RParen => Some(")"),
            Tok::Less   => Some("<"),
            Tok::Greater=> Some(">"),
            Tok::Comma  => Some(","),
            _           => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" ( ", "(")
        .replace(" ) ", ")")
        .replace(" , ", ",")
        .replace(" < ", "<")
        .replace(" > ", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(s: &str) -> Tok {
        Tok::Ident(s.to_string())
    }

    #[test]
    fn simple_field_chain() {
        // a.b.c
        let toks = vec![ident("a"), Tok::Dot, ident("b"), Tok::Dot, ident("c")];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(
            res,
            vec![
                ChainMember::Field("a".into()),
                ChainMember::Field("b".into()),
                ChainMember::Field("c".into())
            ]
        );
    }

    #[test]
    fn function_call_chain() {
        // a.f1().f2(x)
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("f1"),
            Tok::LParen,
            Tok::RParen,
            Tok::Dot,
            ident("f2"),
            Tok::LParen,
            ident("x"),
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 3);
        match &res[2] {
            ChainMember::Call(name, args) => {
                assert_eq!(name, "f2");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0][0], ChainMember::Field("x".into()));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn generic_call() {
        // a.m<>()
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("m"),
            Tok::Less,
            Tok::Greater,
            Tok::LParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        match &res[1] {
            ChainMember::Call(name, args) => {
                assert_eq!(name, "m");
                assert!(args.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn invalid_missing_identifier() {
        // a.(
        let toks = vec![ident("a"), Tok::Dot, Tok::LParen];
        assert!(parse_dot_chain(toks).is_none());
    }

    #[test]
    fn invalid_leading_dot() {
        // .a
        let toks = vec![Tok::Dot, ident("a")];
        assert!(parse_dot_chain(toks).is_none());
    }

    #[test]
    fn long_pure_field() {
        // a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z
        let toks = "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"
            .split('.')
            .flat_map(|s| [ident(s), Tok::Dot])
            .collect::<Vec<_>>();
        let toks = &toks[..toks.len() - 1]; // remove trailing Dot
        let res = parse_dot_chain(toks.to_vec()).unwrap();
        assert_eq!(res.len(), 26);
        assert!(matches!(res.last().unwrap(), ChainMember::Field(n) if n == "z"));
    }

    #[test]
    fn long_mixed_chain() {
        // a.b().c.d().e<f>().g()
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("b"),
            Tok::LParen,
            Tok::RParen,
            Tok::Dot,
            ident("c"),
            Tok::Dot,
            ident("d"),
            Tok::LParen,
            Tok::RParen,
            Tok::Dot,
            ident("e"),
            Tok::Less,
            Tok::Greater,
            Tok::LParen,
            Tok::RParen,
            Tok::Dot,
            ident("g"),
            Tok::LParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 6);
        let calls: Vec<_> = res
            .iter()
            .filter(|m| matches!(m, ChainMember::Call(_, _)))
            .collect();
        assert_eq!(calls.len(), 4);
    }

    #[test]
    fn nested_call_args() {
        // a.f(b.c, d.e.g())
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("f"),
            Tok::LParen,
            ident("b"),
            Tok::Dot,
            ident("c"),
            Tok::Comma,
            ident("d"),
            Tok::Dot,
            ident("e"),
            Tok::Dot,
            ident("g"),
            Tok::LParen,
            Tok::RParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2); // a, f(...)
        match &res[1] {
            ChainMember::Call(_, args) => {
                assert_eq!(args.len(), 2);
                assert_eq!(args[0][0], ChainMember::Field("b".into()));
                assert_eq!(args[0][1], ChainMember::Field("c".into()));
                assert_eq!(args[1][2], ChainMember::Field("g".into()));
                // assert_eq!(args[1][2], ChainMember::Field("g".into(), vec![]));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn huge_single_line() {
        // a.f1().f2().f3()...f100()
        let mut toks = vec![ident("a")];
        for i in 1..=100 {
            toks.extend([Tok::Dot, ident(&format!("f{i}")), Tok::LParen, Tok::RParen]);
        }
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 101);
        assert!(matches!(res.last().unwrap(), ChainMember::Call(n, _) if n == "f100"));
    }

    #[test]
    fn call_with_simple_binary_args() {
        // a.f(b + 1, c - 1)
        // “+” 与 “-” 在 parser 里被当成普通 Ident，所以仍符合语法
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("f"),
            Tok::LParen,
            ident("b"),
            ident("+"),
            ident("1"),
            Tok::Comma,
            ident("c"),
            ident("-"),
            ident("1"),
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2);
        match &res[1] {
            ChainMember::Call(name, args) => {
                assert_eq!(name, "f");
                assert_eq!(args.len(), 2);
                // arg0: b + 1
                assert_eq!(
                    args[0],
                    vec![
                        ChainMember::Field("b".into()),
                        ChainMember::Field("+".into()),
                        ChainMember::Field("1".into()),
                    ]
                );
                // arg1: c - 1
                assert_eq!(
                    args[1],
                    vec![
                        ChainMember::Field("c".into()),
                        ChainMember::Field("-".into()),
                        ChainMember::Field("1".into()),
                    ]
                );
            }
            _ => panic!(),
        }
    }

    #[test]
    fn nested_call_with_mixed_expressions() {
        // a.g(x + y * z, h(1 + 2))
        let toks = vec![
            ident("a"),
            Tok::Dot,
            ident("g"),
            Tok::LParen,
            ident("x"),
            ident("+"),
            ident("y"),
            ident("*"),
            ident("z"),
            Tok::Comma,
            ident("h"),
            Tok::LParen,
            ident("1"),
            ident("+"),
            ident("2"),
            Tok::RParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2);
        match &res[1] {
            ChainMember::Call(name, args) => {
                assert_eq!(name, "g");
                assert_eq!(args.len(), 2);
                // arg0: x + y * z
                assert_eq!(
                    args[0],
                    vec![
                        ChainMember::Field("x".into()),
                        ChainMember::Field("+".into()),
                        ChainMember::Field("y".into()),
                        ChainMember::Field("*".into()),
                        ChainMember::Field("z".into()),
                    ]
                );
                // arg1: h(1 + 2)
                pp!(&res);                       // 整链
                pp!(&args[1]);                  // 单个实参
                // assert_eq!(args[1].len(), 1);
                // 我的问题：这里我想 pretty print 出 args[1]，请给我加一个 漂亮打印的函数
                match &args[1][0] {
                    ChainMember::Call(inner_name, inner_args) => {
                        assert_eq!(inner_name, "h");
                        assert_eq!(
                            inner_args[0],
                            vec![
                                ChainMember::Field("1".into()),
                                ChainMember::Field("+".into()),
                                ChainMember::Field("2".into()),
                            ]
                        );
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn single_arg_binary_expression() {
        // obj.method(a * b / c)
        let toks = vec![
            ident("obj"),
            Tok::Dot,
            ident("method"),
            Tok::LParen,
            ident("a"),
            ident("*"),
            ident("b"),
            ident("/"),
            ident("c"),
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2);
        if let ChainMember::Call(_, args) = &res[1] {
            assert_eq!(args.len(), 1);
            assert_eq!(
                args[0],
                vec![
                    ChainMember::Field("a".into()),
                    ChainMember::Field("*".into()),
                    ChainMember::Field("b".into()),
                    ChainMember::Field("/".into()),
                    ChainMember::Field("c".into()),
                ]
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn empty_call_with_binary_expression_in_generic() {
        // foo.bar<1 + 2>()
        // generic part is skipped; 1 + 2 is treated as binary expression
        let toks = vec![
            ident("foo"),
            Tok::Dot,
            ident("bar"),
            Tok::Less,
            ident("1"),
            ident("+"),
            ident("2"),
            Tok::Greater,
            Tok::LParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2);
        match &res[1] {
            ChainMember::Call(name, args) => {
                assert_eq!(name, "bar");
                assert!(args.is_empty()); // no actual call arguments
            }
            _ => panic!(),
        }
    }
}
