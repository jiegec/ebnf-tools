use clap::{App, Arg};
use ebnf_tools;
use ebnf_tools::Generate;
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
    let alloc = ebnf_gen::ASTAlloc::default();
    let ebnf = ebnf_gen::work(&code, &alloc);
    if let Ok(ebnf) = ebnf {
        println!("{}", ebnf.generate(&ebnf, 30));
    } else {
        println!("{:?}", ebnf.unwrap_err());
    }
}
