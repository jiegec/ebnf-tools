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
        .arg(Arg::with_name("dot").short("d").help("Print graphviz"))
        .get_matches();
    let opts = matches.value_of("file").unwrap();
    let dot = matches.is_present("dot");
    let code = fs::read_to_string(opts).unwrap();
    let ast_alloc = ASTAlloc::default();
    let flatten_alloc = FlattenAlloc::default();
    let ebnf = work(&code, &ast_alloc);
    if let Ok(ebnf) = ebnf {
        let res = flatten(&ebnf, &flatten_alloc);
        let lr = lr_graph(&res);
        if dot {
            println!("{}", lr.print_dot().unwrap());
        } else {
            println!("{}", lr);
        }
    } else {
        println!("{:?}", ebnf.unwrap_err());
    }
}
