use std::{fmt::Display, rc::Rc};

enum Proposition {
    Absurdum,
    Term(String),
    And(Rc<Proposition>, Rc<Proposition>),
    Or(Rc<Proposition>, Rc<Proposition>),
    Not(Rc<Proposition>),
    Implies(Rc<Proposition>, Rc<Proposition>),
    Iff(Rc<Proposition>, Rc<Proposition>),
}

impl Display for Proposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absurdum => write!(f, "⊥"),
            Self::Term(name) => write!(f, "{}", name),
            Self::And(left, right) => write!(f, "({} ^ {})", left, right),
            Self::Or(left, right) => write!(f, "({} v {})", left, right),
            Self::Not(t) => write!(f, "(!{})", t),
            Self::Implies(left, right) => write!(f, "({} => {})", left, right),
            Self::Iff(left, right) => write!(f, "({} <=> {})", left, right),
        }
    }
}

impl Proposition {
    fn new_absurdum() -> Rc<Proposition> {
        Rc::new(Proposition::Absurdum)
    }

    fn new_term(name: &str) -> Rc<Proposition> {
        Rc::new(Proposition::Term(name.to_string()))
    }

    fn new_and(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::And(left.clone(), right.clone()))
    }

    fn new_or(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Or(left.clone(), right.clone()))
    }

    fn new_not(prop: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Not(prop.clone()))
    }

    fn new_implies(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Implies(left.clone(), right.clone()))
    }

    fn new_iff(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Iff(left.clone(), right.clone()))
    }
}

type Level = u32;

struct Fitch {
    statements: Vec<(Level, Proposition)>,
    start_of_deductions: u32,
}

impl Display for Fitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        self.statements
            .iter()
            .enumerate()
            .for_each(|(i, (level, expression))| {
                if i as u32 == self.start_of_deductions {
                    res.push_str("------------------\n");
                }
                for _ in 0..*level {
                    res.push_str("    ");
                }
                res.push_str(expression.to_string().as_str());
                res.push_str("\n");
            });
        write!(f, "{}", res)
    }
}
