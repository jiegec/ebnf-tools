use ebnf_gen;

fn main() {
    let code = include_str!("../decaf.ebnf");
    let alloc = ebnf_gen::ASTAlloc::default();
    println!("{:#?}", ebnf_gen::work(code, &alloc));
}
