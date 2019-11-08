use super::ast::*;
use rand::random;

pub trait Generate {
    fn generate<'a>(&self, file: &File) -> String;
}

impl<'a> Generate for File<'a> {
    fn generate(&self, file: &File) -> String {
        let top_level = &self.rules[0];
        top_level.generate(file)
    }
}

impl<'a> Generate for RuleDef<'a> {
    fn generate(&self, file: &File) -> String {
        let prod = self.prod[(self.prod.len() as f64 * random::<f64>()) as usize];
        prod.generate(file)
    }
}

impl<'a> Generate for Prod<'a> {
    fn generate(&self, file: &File) -> String {
        use Prod::*;
        match self {
            Concat(l, r) => {
                let mut ll = l.generate(file);
                let rr = r.generate(file);
                ll.push_str(" ");
                ll.push_str(&rr);
                ll
            }
            Terminal(s) => String::from(&s[1..s.len() - 1]),
            NonTerminal(s) => {
                if let Some(term) = file.mapping.borrow().get(s) {
                    term.generate(file)
                } else {
                    panic!("{:?} not found", s)
                }
            }
            Optional(p) => {
                if random::<f64>() < 0.6 {
                    p.generate(file)
                } else {
                    String::new()
                }
            }
            Star(p) => {
                let mut result = String::new();
                while random::<f64>() < 0.6 {
                    result.push_str(&p.generate(file));
                }
                result
            }
            Eps => String::new(),
        }
    }
}
