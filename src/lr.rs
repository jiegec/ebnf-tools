use crate::*;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::io::{self, Write};

// lr graph
#[derive(Debug)]
pub struct LrGraph<'a> {
    rules: &'a Vec<FlatRuleDef<'a>>,
    states: Vec<LrState<'a>>,
    terminals: BTreeSet<&'a str>,
    non_terminals: BTreeSet<&'a str>,
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
    edges: Vec<(&'a FlatProd<'a>, usize)>,
}

// Lr tables
#[derive(Debug)]
pub struct LrTable<'a> {
    rows: Vec<LrTableEntry<'a>>,
    graph: &'a LrGraph<'a>,
}

#[derive(Debug, Clone)]
pub enum LrAction {
    Shift(usize),
    Reduce(usize),
    Accept
}

#[derive(Debug, Clone)]
pub struct LrTableEntry<'a> {
    // terminals -> action
    actions: HashMap<&'a str, Vec<LrAction>>,
    // non terminals -> goto
    goto: HashMap<&'a str, usize>,
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
        terminals: BTreeSet::new(),
        non_terminals: BTreeSet::new(),
    };
    graph.terminals.insert("#");

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
                match prod {
                    FlatProd::Terminal(name) => graph.terminals.insert(name),
                    FlatProd::NonTerminal(name) => graph.non_terminals.insert(name),
                    _ => false
                };

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
                    .find(|state| state.prods == new_state.prods)
                    .map(|state| (*state).clone());
                if let Some(old) = exists {
                    graph.states[current].edges.push((prod, old.index));
                } else {
                    nodes += 1;
                    graph.states[current].edges.push((prod, new_state.index));
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
                if rule.prod.len() == *position {
                    write!(f, " .")?;
                }
                write!(f, ", ")?;
            }
            writeln!(f)?;
            write!(f, "Edges: ")?;
            for (prod, next_state) in state.edges.iter() {
                match prod {
                    FlatProd::Terminal(name) | FlatProd::NonTerminal(name) => {
                        write!(f, " {} -> {}", name, next_state)?
                    }
                    FlatProd::Eps => write!(f, " _ -> {}", next_state)?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<'a> LrGraph<'a> {
    pub fn print_dot(&self) -> io::Result<String> {
        let res: Vec<u8> = Vec::new();
        let mut f = io::Cursor::new(res);
        writeln!(f, "digraph {{")?;
        for state in self.states.iter() {
            write!(f, "{}[shape=box, label=\"I{}:", state.index, state.index)?;
            for ProdState {
                position,
                rule_index,
            } in state.prods.iter()
            {
                let rule = &self.rules[*rule_index];
                write!(f, "{}->", rule.name)?;
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
                if rule.prod.len() == *position {
                    write!(f, " .")?;
                }
                write!(f, "\\n")?;
            }
            writeln!(f, "\"]")?;
            for (prod, next_state) in state.edges.iter() {
                match prod {
                    FlatProd::Terminal(name) | FlatProd::NonTerminal(name) => {
                        writeln!(f, "{} -> {} [label=\"{}\"]", state.index, next_state, name)?
                    }
                    FlatProd::Eps => {
                        writeln!(f, "{} -> {} [label=\"_\"]", state.index, next_state)?
                    }
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(String::from_utf8(f.into_inner()).unwrap())
    }
}

pub fn lr0_table<'a>(graph: &'a LrGraph<'a>) -> LrTable<'a> {
    let mut rows = vec![
        LrTableEntry {
            actions: HashMap::new(),
            goto: HashMap::new()
        };
        graph.states.len()
    ];
    for state in graph.states.iter() {
        for (prod, index) in state.edges.iter() {
            match prod {
                FlatProd::NonTerminal(name) => {
                    // goto
                    rows[state.index].goto.insert(name, *index);
                }
                FlatProd::Terminal(name) => {
                    // shift
                    rows[state.index]
                        .actions
                        .entry(name)
                        .or_insert(vec![])
                        .push(LrAction::Shift(*index));
                }
                _ => unimplemented!(),
            }
        }
        for ProdState {
            position,
            rule_index,
        } in state.prods.iter()
        {
            let rule = &graph.rules[*rule_index];
            if *position == rule.prod.len() {
                // prod
                if *rule_index == 0 {
                    // accept
                    rows[state.index]
                        .actions
                        .entry("#")
                        .or_insert(vec![])
                        .push(LrAction::Accept);
                } else {
                    // reduce for all terminals
                    for prod in graph.terminals.iter() {
                        rows[state.index]
                            .actions
                            .entry(prod)
                            .or_insert(vec![])
                            .push(LrAction::Reduce(*rule_index));
                    }
                }
            }
        }
    }
    LrTable { rows, graph }
}

impl<'a> fmt::Display for LrTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State\t")?;
        for prod in self.graph.terminals.iter() {
            write!(f, "{}\t", prod)?;
        }
        for prod in self.graph.non_terminals.iter() {
            write!(f, "{}\t", prod)?;
        }
        writeln!(f)?;
        for (index, row) in self.rows.iter().enumerate() {
            write!(f, "{}\t", index)?;
            for prod in self.graph.terminals.iter() {
                if let Some(vec) = row.actions.get(prod) {
                    for (idx, action) in vec.iter().enumerate() {
                        if idx > 0 {
                            write!(f, "/")?;
                        }
                        write!(f, "{}", action)?;
                    }
                }
                write!(f, "\t")?;
            }
            for prod in self.graph.non_terminals.iter() {
                if let Some(target) = row.goto.get(prod) {
                    write!(f, "{}", target)?;
                }
                write!(f, "\t")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for LrAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LrAction::Shift(s) => write!(f, "s{}", s),
            LrAction::Reduce(r) => write!(f, "r{}", r),
            LrAction::Accept => write!(f, "acc"),
        }
    }
}