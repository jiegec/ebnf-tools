use crate::ast::*;
use std::cell::Cell;
use typed_arena::Arena;

#[derive(Default)]
pub struct FlattenAlloc<'a> {
    pub string: Arena<String>,
    pub prod: Arena<Prod<'a>>,
    pub flat_prod: Arena<FlatProd<'a>>,
    counter: Cell<usize>,
}

// returns new_name, new_prod, new_rules
fn flatten_one<'a>(
    name: &'a str,
    prod: &'a Prod<'a>,
    alloc: &'a FlattenAlloc<'a>,
) -> (&'a str, &'a Prod<'a>, Vec<(&'a str, &'a Prod<'a>)>) {
    match prod {
        Prod::Eps | Prod::Terminal(_) | Prod::NonTerminal(_) => (name, prod, vec![]),
        Prod::Concat(l, r) => {
            if let Prod::Eps = l {
                flatten_one(name, r, alloc)
            } else if let Prod::Eps = r {
                flatten_one(name, l, alloc)
            } else {
                let (new_name, ll, l_res) = flatten_one(name, l, alloc);
                if l_res.len() > 0 {
                    (new_name, alloc.prod.alloc(Prod::Concat(ll, r)), l_res)
                } else {
                    let (new_name, rr, r_res) = flatten_one(name, r, alloc);
                    (new_name, alloc.prod.alloc(Prod::Concat(l, rr)), r_res)
                }
            }
        }
        Prod::Optional(o) => {
            let orig_name = format!("{}", name);
            let orig_name = alloc.string.alloc(orig_name);
            let opt_name = format!("{}_opt{}", name, alloc.counter.get());
            alloc.counter.set(alloc.counter.get() + 1);
            let opt_name = alloc.string.alloc(opt_name);
            let opt = alloc.prod.alloc(Prod::NonTerminal(opt_name));
            (orig_name, opt, vec![(opt_name, o), (opt_name, &Prod::Eps)])
        }
        Prod::Star(o) => {
            let orig_name = format!("{}", name);
            let orig_name = alloc.string.alloc(orig_name);
            let star_name = format!("{}_star{}", name, alloc.counter.get());
            alloc.counter.set(alloc.counter.get() + 1);
            let star_name = alloc.string.alloc(star_name);
            let star = alloc.prod.alloc(Prod::NonTerminal(star_name));
            let concat = alloc.prod.alloc(Prod::Concat(o, star));
            (
                orig_name,
                star,
                vec![(star_name, concat), (star_name, &Prod::Eps)],
            )
        }
    }
}

fn flatten_prod<'a>(prod: &'a Prod<'a>, alloc: &'a FlattenAlloc<'a>) -> Vec<&'a FlatProd<'a>> {
    match prod {
        Prod::Concat(l, r) => {
            let mut l_res = flatten_prod(l, alloc);
            let mut r_res = flatten_prod(r, alloc);
            l_res.append(&mut r_res);
            l_res
        }
        Prod::Terminal(t) => {
            let p = alloc.flat_prod.alloc(FlatProd::Terminal(t));
            vec![p]
        }
        Prod::NonTerminal(t) => {
            let p = alloc.flat_prod.alloc(FlatProd::NonTerminal(t));
            vec![p]
        }
        Prod::Eps => {
            let p = alloc.flat_prod.alloc(FlatProd::Eps);
            vec![p]
        }
        _ => unreachable!(),
    }
}

fn flatten_rule<'a>(rule: RuleDef<'a>, alloc: &'a FlattenAlloc<'a>) -> FlatRuleDef<'a> {
    FlatRuleDef {
        name: rule.name,
        prod: flatten_prod(&rule.prod[0], alloc),
    }
}

pub fn flatten<'a>(file: &'a File<'a>, alloc: &'a FlattenAlloc<'a>) -> Vec<FlatRuleDef<'a>> {
    let mut rules = file.rules.clone();
    loop {
        let mut new_rules = vec![];
        for rule in rules.iter() {
            for prod in rule.prod.iter() {
                let (new_name, res_prod, new) = flatten_one(rule.name, prod, alloc);
                new_rules.push(RuleDef {
                    name: new_name,
                    prod: vec![res_prod],
                });
                for (name, new_rule) in new {
                    new_rules.push(RuleDef {
                        name,
                        prod: vec![new_rule],
                    });
                }
            }
        }
        if rules.len() == new_rules.len() {
            break;
        }
        rules = new_rules;
    }

    rules
        .into_iter()
        .map(|rule| flatten_rule(rule, alloc))
        .collect()
}
