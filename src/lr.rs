use crate::*;
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug)]
pub struct LrGraph<'a> {
    rules: &'a Vec<FlatRuleDef<'a>>,
    states: Vec<LrState<'a>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProdState {
    position: usize,
    rule_index: usize,
}

#[derive(Debug, Clone)]
pub struct LrState<'a> {
    index: usize,
    prods: Vec<ProdState>,
    edges: Vec<(FlatProd<'a>, &'a LrState<'a>)>,
}

pub fn closure<'a>(rules: &'a Vec<FlatRuleDef<'a>>, orig: Vec<ProdState>) -> Vec<ProdState> {
    let mut res = orig.clone();
    loop {
        let mut current = res.clone();
        for ProdState {
            position,
            rule_index,
        } in res.iter()
        {
            let rule = &rules[*rule_index];
            if *position < rule.prod.len() {
                if let FlatProd::NonTerminal(name) = rule.prod[*position] {
                    // add all prod of name
                    for (idx, rule) in rules.iter().enumerate() {
                        if rule.name == *name {
                            current.push(ProdState {
                                position: 0,
                                rule_index: idx,
                            });
                        }
                    }
                }
            }
        }
        current.sort();
        current.dedup();
        if res.len() == current.len() {
            break;
        }
        res = current;
    }
    res
}

pub fn lr_graph<'a>(rules: &'a Vec<FlatRuleDef<'a>>) -> LrGraph<'a> {
    let mut graph = LrGraph {
        rules,
        states: vec![],
    };
    let mut nodes = 1;
    let mut init_state = LrState {
        index: 0,
        prods: vec![],
        edges: vec![],
    };
    init_state.prods = closure(
        rules,
        vec![ProdState {
            position: 0,
            rule_index: 0,
        }],
    );
    graph.states.insert(0, init_state);

    let mut pending = VecDeque::new();
    pending.push_back(0);
    loop {
        if let Some(current) = pending.pop_front() {
            let state = (*graph
                .states
                .iter()
                .filter(|state| state.index == current)
                .next()
                .unwrap())
            .clone();
            let mut possible_prods: Vec<&'a FlatProd<'a>> = state
                .prods
                .iter()
                .filter_map(|prod_state| {
                    let rule = &rules[prod_state.rule_index];
                    if prod_state.position < rule.prod.len() {
                        Some(rule.prod[prod_state.position])
                    } else {
                        None
                    }
                })
                .collect();
            possible_prods.sort();
            possible_prods.dedup();
            for prod in possible_prods {
                let mut new_state = LrState {
                    index: nodes,
                    prods: vec![],
                    edges: vec![],
                };
                for ProdState {
                    position,
                    rule_index,
                } in state.prods.iter()
                {
                    let rule = &rules[*rule_index];
                    if *position < rule.prod.len() && rule.prod[*position] == prod {
                        new_state.prods.push(ProdState {
                            position: position + 1,
                            rule_index: *rule_index,
                        });
                    }
                }
                new_state.prods = closure(rules, new_state.prods);
                let exists = graph
                    .states
                    .iter()
                    .any(|state| state.prods == new_state.prods);
                if !exists {
                    nodes += 1;
                    pending.push_back(new_state.index);
                    graph.states.push(new_state);
                }
            }
        } else {
            break;
        }
    }

    graph
}

impl<'a> fmt::Display for LrGraph<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for state in self.states.iter() {
            write!(f, "{}: ", state.index)?;
            for ProdState {
                position,
                rule_index,
            } in state.prods.iter()
            {
                let rule = &self.rules[*rule_index];
                write!(f, "{} ::=", rule.name)?;
                for (idx, prod) in rule.prod.iter().enumerate() {
                    if idx == *position {
                        write!(f, " .")?;
                    }
                    match prod {
                        FlatProd::Terminal(name) | FlatProd::NonTerminal(name) => {
                            write!(f, " {}", name)?
                        }
                        FlatProd::Eps => write!(f, " _")?,
                    }
                }
                write!(f, ", ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
