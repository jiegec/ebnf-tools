use ebnf_gen;
use ebnf_gen::Generate;

fn main() {
    let code = include_str!("../decaf-2019.ebnf");
    let alloc = ebnf_gen::ASTAlloc::default();
    let ebnf = ebnf_gen::work(code, &alloc);
    if let Ok(ebnf) = ebnf {
        println!("{}", ebnf.generate(&ebnf));
    }
}
