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

pub type FlatProds<'a> = Vec<&'a FlatProd<'a>>;

#[derive(Debug, PartialEq, Eq)]
pub struct FlatRuleDef<'a> {
    pub name: &'a str,
    pub prod: FlatProds<'a>,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone)]
pub enum FlatProd<'a> {
    Terminal(&'a str),
    NonTerminal(&'a str),
    Eps,
}

impl<'a> FlatProd<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            FlatProd::Terminal(t) => t,
            FlatProd::NonTerminal(t) => t,
            FlatProd::Eps => "_",
        }
    }

    pub fn is_eps(&self) -> bool {
        match self {
            FlatProd::Eps => true,
            _ => false,
        }
    }
}
