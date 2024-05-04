use std::{borrow::Borrow, fmt::Display, rc::Rc};

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

impl PartialEq for Proposition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Term(l0), Self::Term(r0)) => l0 == r0,
            (Self::And(l0, l1), Self::And(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Or(l0, l1), Self::Or(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Not(l0), Self::Not(r0)) => l0 == r0,
            (Self::Implies(l0, l1), Self::Implies(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Iff(l0, l1), Self::Iff(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
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

enum FitchComponent {
    Assumption(Rc<Proposition>),
    Deduction(Rc<Proposition>),
}

impl FitchComponent {
    fn unwrap(&self) -> &Rc<Proposition> {
        match self {
            FitchComponent::Assumption(t) => t,
            FitchComponent::Deduction(t) => t,
        }
    }
}

type Level = u32;

struct Fitch {
    statements: Vec<(Level, FitchComponent)>,
    start_of_deductions: usize,
    current_level: u32,
}

impl Display for Fitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        self.statements
            .iter()
            .enumerate()
            .for_each(|(i, (level, expression))| {
                if i == self.start_of_deductions {
                    res.push_str("------------------\n");
                }
                for _ in 0..*level {
                    res.push_str("    ");
                }
                res.push_str(expression.unwrap().to_string().as_str());
                res.push_str("\n");
            });
        write!(f, "{}", res)
    }
}

impl Fitch {
    fn new() -> Fitch {
        Fitch {
            statements: Vec::new(),
            start_of_deductions: 0,
            current_level: 0,
        }
    }

    fn add_assumption(&mut self, prop: &Rc<Proposition>) {
        self.statements.insert(
            self.start_of_deductions,
            (0, FitchComponent::Assumption(prop.clone())),
        );
        self.start_of_deductions += 1;
    }

    fn delete_last_row(&mut self) {
        self.statements.pop();
        if self.statements.len() < self.start_of_deductions {
            self.start_of_deductions = self.statements.len();
        }
    }

    fn introduce_and(&mut self, left: usize, right: usize) -> bool {
        let left = match self.statements.get(left) {
            Some(v) => v,
            None => return false,
        };
        let right = match self.statements.get(right) {
            Some(v) => v,
            None => return false,
        };

        let ris = Proposition::new_and(left.1.unwrap(), right.1.unwrap());
        self.statements
            .push((self.current_level, FitchComponent::Deduction(ris)));
        true
    }

    fn eliminate_and(&mut self, assum: usize, prop: &Rc<Proposition>) -> bool {
        let assum = match self.statements.get(assum) {
            Some(v) => v.1.unwrap(),
            None => return false,
        };

        match assum.borrow() {
            Proposition::And(left, _) if left == prop => {
                self.statements
                    .push((self.current_level, FitchComponent::Deduction(left.clone())));
                true
            }
            Proposition::And(_, right) if right == prop => {
                self.statements
                    .push((self.current_level, FitchComponent::Deduction(right.clone())));
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fitch, Proposition};

    #[test]
    fn eliminate_and() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        let prop = Proposition::new_and(&t0, &t1);
        fitch.add_assumption(&prop);
        let ris = fitch.eliminate_and(0, &Proposition::new_term("A"));
        assert!(ris);
        let ris = fitch.eliminate_and(0, &Proposition::new_term("C"));
        assert!(!ris);
    }
}
