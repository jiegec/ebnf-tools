#![feature(proc_macro_hygiene)] // allow proc macro output macro definition

mod ast;
mod errors;
mod gen;
mod loc;

pub use ast::*;
pub use errors::*;
pub use gen::*;
pub use loc::*;

use parser_macros::lalr1;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub fn work<'p>(code: &'p str, alloc: &'p ASTAlloc<'p>) -> Result<&'p File<'p>, Errors> {
    let mut parser = Parser {
        alloc,
        error: Errors::default(),
    };
    let mut lexer = Lexer::new(code.as_bytes()); // Lexer can be used independently from Parser, you can use it to debug
    match parser.parse(&mut lexer) {
        Ok(program) if parser.error.0.is_empty() => {
            let mut mapping = program.mapping.borrow_mut();
            for rule in program.rules.iter() {
                mapping.insert(rule.name, rule);
            }
            Ok(program)
        }
        Err(token) => {
            let mut error = parser.error;
            let loc = Loc(token.line, token.col);
            match token.ty {
                TokenKind::_Err => {
                    error.issue(loc, ErrorKind::UnrecognizedChar(token.piece[0] as char))
                }
                _ => error.issue(loc, ErrorKind::SyntaxError),
            }
            Err(error)
        }
        _ => Err(parser.error),
    }
}

pub struct Parser<'p> {
    pub alloc: &'p ASTAlloc<'p>,
    pub error: Errors,
}

impl<'p> Token<'p> {
    pub fn str(&self) -> &'p str {
        std::str::from_utf8(self.piece).unwrap()
    }
    pub fn loc(&self) -> Loc {
        Loc(self.line, self.col)
    }
}

impl Lexer<'_> {
    pub fn loc(&self) -> Loc {
        Loc(self.line, self.col)
    }
}

#[lalr1(File)]
#[lex(
    r##"
priority = [
]

[lexical]
'\*' = 'Star'
'\+' = 'Plus'
'\?' = 'Opt'
'\(' = 'LPar' # short for parenthesis
'\)' = 'RPar'
'\|' = 'Or'
';' = 'Comma'
'_' = 'Eps'
'::=' = 'Def'
"'[^']+'" = 'StringLit'
'//[^\n]*' = '_Eps'
'\s+' = '_Eps'
'[A-Za-z]\w*' = 'Id'
'.' = '_Err'
"##
)]
impl<'p> Parser<'p> {
    #[rule(File -> RuleList)]
    fn file(&self, mut l: Vec<RuleDef<'p>>) -> &'p File<'p> {
        l.reverse();
        self.alloc.file.alloc(File {
            rules: l,
            mapping: RefCell::new(BTreeMap::new()),
        })
    }

    #[rule(RuleList -> Rule RuleList)]
    fn rule_list(&self, r: RuleDef<'p>, mut l: Vec<RuleDef<'p>>) -> Vec<RuleDef<'p>> {
        l.push(r);
        l
    }
    #[rule(RuleList -> Rule)]
    fn rule_list1(&self, r: RuleDef<'p>) -> Vec<RuleDef<'p>> {
        vec![r]
    }

    #[rule(Rule -> Id Def ProdList Comma)]
    fn rule(
        &self,
        id: Token<'p>,
        _d: Token<'p>,
        prod: Vec<&'p Prod<'p>>,
        _c: Token,
    ) -> RuleDef<'p> {
        RuleDef {
            name: id.str(),
            prod,
        }
    }

    #[rule(ProdList -> Prod ProdListRem)]
    fn rule_list_more(&self, p: &'p Prod<'p>, mut r: Vec<&'p Prod<'p>>) -> Vec<&'p Prod<'p>> {
        r.push(p);
        r
    }

    #[rule(ProdListRem -> Or Prod ProdListRem)]
    fn rule_list_rem(
        &self,
        _r: Token<'p>,
        p: &'p Prod<'p>,
        mut r: Vec<&'p Prod<'p>>,
    ) -> Vec<&'p Prod<'p>> {
        r.push(p);
        r
    }
    #[rule(ProdListRem -> )]
    fn rule_list_rem_0(&self) -> Vec<&'p Prod<'p>> {
        vec![]
    }

    #[rule(Prod -> LPar Prod RPar)]
    fn rule_paren(&self, _l: Token<'p>, p: &'p Prod<'p>, _r: Token<'p>) -> &'p Prod<'p> {
        p
    }

    #[rule(Prod -> Prod Plus)]
    fn rule_plus(&self, p: &'p Prod<'p>, _p: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Plus(p))
    }

    #[rule(Prod -> Prod Opt)]
    fn rule_opt(&self, p: &'p Prod<'p>, _o: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Optional(p))
    }

    #[rule(Prod -> Prod Star)]
    fn rule_star(&self, p: &'p Prod<'p>, _s: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Star(p))
    }

    #[rule(Prod -> Prod Prod)]
    fn rule_concat(&self, l: &'p Prod<'p>, r: &'p Prod<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Concat(l, r))
    }

    #[rule(Prod -> Eps)]
    fn prod_eps(&self, _e: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Eps)
    }

    #[rule(Prod -> Id)]
    fn prod_id(&self, id: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::NonTerminal(id.str()))
    }

    #[rule(Prod -> StringLit)]
    fn prod_string(&self, lit: Token<'p>) -> &'p Prod<'p> {
        self.alloc.prod.alloc(Prod::Terminal(lit.str()))
    }
}
