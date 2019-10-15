use typed_arena::Arena;

#[derive(Default)]
pub struct ASTAlloc<'a> {
    pub file: Arena<File<'a>>,
    pub rule: Arena<RuleDef<'a>>,
    pub prod: Arena<Prod<'a>>,
}

#[derive(Debug)]
pub struct File<'a> {
    pub rules: Vec<RuleDef<'a>>,
}

#[derive(Debug)]
pub struct RuleDef<'a> {
    pub name: &'a str,
    pub prod: Vec<&'a Prod<'a>>,
}

#[derive(Debug)]
pub enum Prod<'a> {
    Concat(&'a Prod<'a>, &'a Prod<'a>),
    Terminal(&'a str),
    NonTerminal(&'a str),
    Optional(&'a Prod<'a>),
    Star(&'a Prod<'a>),
    Plus(&'a Prod<'a>),
    Eps,
}
