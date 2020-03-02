use clap::{App, Arg};
use ebnf_tools::*;
use std::fs;

fn main() {
    let matches = App::new("generate")
        .arg(
            Arg::with_name("file")
                .value_name("file")
                .takes_value(true)
                .required(true),
        )
        .arg(Arg::with_name("first").short("f").help("Print FIRST set"))
        .arg(Arg::with_name("follow").short("F").help("Print FOLLOW set"))
        .get_matches();
    let opts = matches.value_of("file").unwrap();
    let first = matches.is_present("first");
    let follow = matches.is_present("follow");
    let code = fs::read_to_string(opts).unwrap();
    let ast_alloc = ASTAlloc::default();
    let flatten_alloc = FlattenAlloc::default();
    let ebnf = work(&code, &ast_alloc);
    if let Ok(ebnf) = ebnf {
        let res = flatten(&ebnf, &flatten_alloc);
        let first_s = first_set(&res);
        let follow_s = follow_set(&res, &first_s);
        if first {
            let mut first: Vec<_> = first_s.iter().collect();
            first.sort();
            println!("FIRST:");
            for item in first {
                println!("{}:  {:?}", item.0, item.1);
            }
        }
        if follow {
            let mut follow: Vec<_> = follow_s.iter().collect();
            follow.sort();
            println!("FOLLOW:");
            for item in follow {
                println!("{}:  {:?}", item.0, item.1);
            }
        }
    } else {
        println!("{:?}", ebnf.unwrap_err());
    }
}
