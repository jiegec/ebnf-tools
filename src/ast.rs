use std::cell::RefCell;
use std::collections::BTreeMap;
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
    pub mapping: RefCell<BTreeMap<&'a str, &'a RuleDef<'a>>>,
}

#[derive(Debug, Clone)]
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
    Eps,
}

#[derive(Debug)]
pub struct FlatRuleDef<'a> {
    pub name: &'a str,
    pub prod: Vec<&'a FlatProd<'a>>,
}

#[derive(Debug)]
pub enum FlatProd<'a> {
    Terminal(&'a str),
    NonTerminal(&'a str),
    Eps,
}
