use clap::{App, Arg};
use ebnf_tools;
use ebnf_tools::flatten;
use ebnf_tools::FlatProd;
use std::fs;

fn main() {
    let matches = App::new("generate")
        .arg(
            Arg::with_name("file")
                .value_name("file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let opts = matches.value_of("file").unwrap();
    let code = fs::read_to_string(opts).unwrap();
    let ast_alloc = ebnf_gen::ASTAlloc::default();
    let flatten_alloc = ebnf_gen::FlattenAlloc::default();
    let ebnf = ebnf_gen::work(&code, &ast_alloc);
    if let Ok(ebnf) = ebnf {
        let res = flatten(&ebnf, &flatten_alloc);
        for rule in res {
            print!("{} ::=", rule.name);
            for prod in rule.prod {
                match prod {
                    FlatProd::Terminal(name) | FlatProd::NonTerminal(name) => print!(" {}", name),
                    FlatProd::Eps => print!(" _"),
                }
            }
            println!();
        }
    } else {
        println!("{:?}", ebnf.unwrap_err());
    }
}
