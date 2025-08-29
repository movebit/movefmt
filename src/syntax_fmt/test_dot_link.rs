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
}

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
        // Optional generic arguments <...>
        if matches!(self.current(), Tok::Less) {
            self.advance();
            while !matches!(self.current(), Tok::Greater) && !matches!(self.current(), Tok::EOF) {
                self.advance();
            }
            if matches!(self.current(), Tok::Greater) {
                self.advance();
            } else {
                return None;
            }
        }

        if !matches!(self.current(), Tok::LParen) {
            return Some(Vec::new());
        }
        self.advance();

        let mut all = Vec::new();
        loop {
            if matches!(self.current(), Tok::RParen) {
                break;
            }
            let mut sub = Vec::new();
            {
                let mut p2 = Parser {
                    tokens: self.tokens.clone(),
                    pos: self.pos,
                    result: sub,
                };
                p2.parse_chain()?;
                sub = p2.result;
                self.pos = p2.pos;
            }
            all.push(sub);

            if matches!(self.current(), Tok::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        if matches!(self.current(), Tok::RParen) {
            self.advance();
            Some(all)
        } else {
            None
        }
    }
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
            ident("a"), Tok::Dot, ident("b"), Tok::LParen, Tok::RParen,
            Tok::Dot, ident("c"),
            Tok::Dot, ident("d"), Tok::LParen, Tok::RParen,
            Tok::Dot, ident("e"), Tok::Less, Tok::Greater, Tok::LParen, Tok::RParen,
            Tok::Dot, ident("g"), Tok::LParen, Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 6);
        let calls: Vec<_> = res.iter().filter(|m| matches!(m, ChainMember::Call(_, _))).collect();
        assert_eq!(calls.len(), 4);
    }

    #[test]
    fn nested_call_args() {
        // a.f(b.c, d.e.g())
        let toks = vec![
            ident("a"), Tok::Dot, ident("f"), Tok::LParen,
            ident("b"), Tok::Dot, ident("c"),
            Tok::Comma,
            ident("d"), Tok::Dot, ident("e"), Tok::Dot, ident("g"), Tok::LParen, Tok::RParen,
            Tok::RParen,
        ];
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 2); // a, f(...)
        match &res[1] {
            ChainMember::Call(_, args) => {
                assert_eq!(args.len(), 2);
                assert_eq!(args[0][0], ChainMember::Field("b".into()));
                assert_eq!(args[0][1], ChainMember::Field("c".into()));
                assert_eq!(args[1][2], ChainMember::Call("g".into(), vec![]));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn huge_single_line() {
        // a.f1().f2().f3()...f100()
        let mut toks = vec![ident("a")];
        for i in 1..=100 {
            toks.extend([
                Tok::Dot,
                ident(&format!("f{i}")),
                Tok::LParen,
                Tok::RParen,
            ]);
        }
        let res = parse_dot_chain(toks).unwrap();
        assert_eq!(res.len(), 101);
        assert!(matches!(res.last().unwrap(), ChainMember::Call(n, _) if n == "f100"));
    }
}