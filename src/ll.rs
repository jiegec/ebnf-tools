use crate::*;
use std::collections::{BTreeSet, HashMap};

pub type TermSet<'a> = HashMap<&'a str, BTreeSet<&'a FlatProd<'a>>>;

pub fn first_set<'a>(rules: &'a Vec<FlatRuleDef<'a>>) -> TermSet {
    let mut res = HashMap::new();
    loop {
        let mut cur = res.clone();
        for rule in rules.iter() {
            if let FlatProd::Eps = rule.prod[0] {
                cur.entry(rule.name)
                    .or_insert(BTreeSet::new())
                    .insert(&FlatProd::Eps);
            } else {
                for prod in rule.prod.iter() {
                    match prod {
                        FlatProd::Terminal(_) => {
                            cur.entry(rule.name).or_insert(BTreeSet::new()).insert(prod);
                            break;
                        }
                        FlatProd::NonTerminal(name) => {
                            let first = cur.entry(name).or_insert(BTreeSet::new()).clone();
                            let cur_first = cur.entry(rule.name).or_insert(BTreeSet::new());
                            *cur_first = cur_first.union(&first).cloned().collect();
                            if !first.contains(&FlatProd::Eps) {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if cur == res {
            break;
        }
        res = cur;
    }
    res
}

pub fn follow_set<'a>(rules: &'a Vec<FlatRuleDef<'a>>, first: &TermSet<'a>) -> TermSet<'a> {
    let mut res = HashMap::new();
    res.entry(rules[0].name)
        .or_insert(BTreeSet::new())
        .insert(&FlatProd::NonTerminal("#"));
    loop {
        let mut cur = res.clone();
        for rule in rules.iter() {
            for i in 0..rule.prod.len() {
                if let FlatProd::NonTerminal(name) = &rule.prod[i] {
                    let mut stop = false;
                    for j in (i + 1)..rule.prod.len() {
                        match &rule.prod[j] {
                            FlatProd::NonTerminal(follow_name) => {
                                let follow = first
                                    .get(follow_name)
                                    .expect(&format!("can't find {}", follow_name))
                                    .clone();
                                let cur_follow = cur.entry(name).or_insert(BTreeSet::new());
                                *cur_follow = cur_follow.union(&follow).cloned().collect();
                                cur_follow.remove(&FlatProd::Eps);
                                if !first.get(follow_name).unwrap().contains(&FlatProd::Eps) {
                                    stop = true;
                                    break;
                                }
                            }
                            FlatProd::Terminal(_) => {
                                cur.entry(name)
                                    .or_insert(BTreeSet::new())
                                    .insert(rule.prod[j]);
                                stop = true;
                                break;
                            }
                            _ => {
                                stop = true;
                                break;
                            }
                        }
                    }
                    if !stop {
                        let follow = cur.entry(rule.name).or_insert(BTreeSet::new()).clone();
                        let cur_follow = cur.entry(name).or_insert(BTreeSet::new());
                        *cur_follow = cur_follow.union(&follow).cloned().collect();
                    }
                }
            }
        }
        if cur == res {
            break;
        }
        res = cur;
    }
    res
}
