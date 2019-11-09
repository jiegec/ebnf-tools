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
        .arg(
            Arg::with_name("plain")
                .short("p")
                .help("Print plain lr fsm"),
        )
        .arg(Arg::with_name("lr0").short("l").help("Print lr(0) table"))
        .arg(Arg::with_name("slr1").short("s").help("Print slr(1) table"))
        .arg(
            Arg::with_name("flatten")
                .short("f")
                .help("Print flattened rules"),
        )
        .get_matches();
    let opts = matches.value_of("file").unwrap();
    let dot = matches.is_present("dot");
    let plain = matches.is_present("plain");
    let lr0 = matches.is_present("lr0");
    let slr1 = matches.is_present("slr1");
    let flattened = matches.is_present("flatten");
    let code = fs::read_to_string(opts).unwrap();
    let ast_alloc = ASTAlloc::default();
    let flatten_alloc = FlattenAlloc::default();
    let ebnf = work(&code, &ast_alloc);
    if let Ok(ebnf) = ebnf {
        let res = flatten(&ebnf, &flatten_alloc);
        let lr = lr_graph(&res);
        if flattened {
            println!("{:?}", res);
        }
        if dot {
            println!("{}", lr.print_dot().unwrap());
        }
        if plain {
            println!("{}", lr);
        }
        if lr0 {
            println!("LR(0) Table:");
            print!("{}", lr0_table(&lr));
        }
        if slr1 {
            println!("SLR(1) Table:");
            print!("{}", slr1_table(&lr));
        }
    } else {
        println!("{:?}", ebnf.unwrap_err());
    }
}
