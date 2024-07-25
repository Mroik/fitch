use std::{borrow::Borrow, fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum Proposition {
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
            Self::And(left, right) => write!(f, "({} & {})", left, right),
            Self::Or(left, right) => write!(f, "({} | {})", left, right),
            Self::Not(t) => write!(f, "(~{})", t),
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
    pub fn new_absurdum() -> Rc<Proposition> {
        Rc::new(Proposition::Absurdum)
    }

    pub fn new_term(name: &str) -> Rc<Proposition> {
        Rc::new(Proposition::Term(name.to_string()))
    }

    pub fn new_and(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::And(left.clone(), right.clone()))
    }

    pub fn new_or(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Or(left.clone(), right.clone()))
    }

    pub fn new_not(prop: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Not(prop.clone()))
    }

    pub fn new_implies(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Implies(left.clone(), right.clone()))
    }

    pub fn new_iff(left: &Rc<Proposition>, right: &Rc<Proposition>) -> Rc<Proposition> {
        Rc::new(Proposition::Iff(left.clone(), right.clone()))
    }
}

enum Rule {
    Reiter,
    IntroAbs,
    ElimAbs,
    IntroAnd,
    ElimAnd,
    IntroOr,
    ElimOr,
    IntroNot,
    ElimNot,
    IntroImpl,
    ElimImpl,
    IntroIff,
    ElimIff,
}

enum FitchComponent {
    Assumption(Rc<Proposition>),
    Deduction(Rc<Proposition>, Rule, Vec<usize>),
}

impl FitchComponent {
    fn unwrap(&self) -> &Rc<Proposition> {
        match self {
            FitchComponent::Assumption(t) => t,
            FitchComponent::Deduction(t, _, _) => t,
        }
    }
}

type Level = usize;

pub struct Fitch {
    statements: Vec<(Level, FitchComponent)>,
    start_of_deductions: usize,
    current_level: usize,
}

impl Display for Fitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        let mut temp = self.statements.len() as i32 - 1;
        if temp == 0 {
            temp = 1;
        }

        let mut max = 0;
        while temp > 0 {
            temp /= 10;
            max += 1;
        }

        let longest_row = self
            .statements
            .iter()
            .map(|(level, comp)| comp.unwrap().to_string().chars().count() + level * 4)
            .max()
            .unwrap_or(0);

        self.statements
            .iter()
            .enumerate()
            .for_each(|(i, (level, expression))| {
                if i == self.start_of_deductions {
                    res.push_str("------------------\n");
                }

                res.push(' ');
                let mut temp = if i == 0 { 1 } else { i as i32 };
                let mut white = 0;
                while temp > 0 {
                    temp /= 10;
                    white += 1;
                }

                let white = max - white;
                for _ in 0..white {
                    res.push(' ');
                }
                res.push_str(format!("{} | ", i).as_str());

                let is_sub = matches!(expression, FitchComponent::Assumption(_));

                if is_sub {
                    res.push('-');
                }
                for _ in 0..*level {
                    res.push_str("    ");
                }
                if is_sub {
                    res.pop();
                }

                let exp_str = expression.unwrap().to_string();
                res.push_str(exp_str.as_str());

                let to_add = longest_row - (exp_str.chars().count() + level * 4);
                for _ in 0..to_add {
                    res.push(' ');
                }

                match expression {
                    FitchComponent::Assumption(_) => (),
                    FitchComponent::Deduction(_, r, ass) => {
                        let sym = match r {
                            Rule::Reiter => "    Re ",
                            Rule::IntroAbs => "    I ⊥ ",
                            Rule::ElimAbs => "    E ⊥ ",
                            Rule::IntroAnd => "    I & ",
                            Rule::ElimAnd => "    E & ",
                            Rule::IntroOr => "    I | ",
                            Rule::ElimOr => "    E | ",
                            Rule::IntroNot => "    I ~ ",
                            Rule::ElimNot => "    E ~ ",
                            Rule::IntroImpl => "    I => ",
                            Rule::ElimImpl => "    E => ",
                            Rule::IntroIff => "    I <=> ",
                            Rule::ElimIff => "    E <=> ",
                        };
                        res.push_str(sym);
                        let mut assums = ass
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");
                        assums.insert(0, '[');
                        assums.push(']');
                        res.push_str(&assums);
                    }
                }

                res.push('\n');
            });

        for _ in 0..max + 4 {
            res.push(' ');
        }

        for _ in 0..self.current_level {
            res.push_str("    ");
        }

        res.push_str("^\n");
        write!(f, "{}", res)
    }
}

impl Fitch {
    pub fn new() -> Fitch {
        Fitch {
            statements: Vec::new(),
            start_of_deductions: 0,
            current_level: 0,
        }
    }

    pub fn add_assumption(&mut self, prop: &Rc<Proposition>) -> bool {
        if self.statements.len() > self.start_of_deductions {
            return false;
        }

        self.statements
            .push((0, FitchComponent::Assumption(prop.clone())));
        self.start_of_deductions += 1;
        true
    }

    pub fn add_subproof(&mut self, prop: &Rc<Proposition>) {
        self.current_level += 1;
        self.statements
            .push((self.current_level, FitchComponent::Assumption(prop.clone())));
    }

    pub fn end_subproof(&mut self) {
        if self.current_level == 0 {
            return;
        }
        self.current_level -= 1;
    }

    pub fn delete_last_row(&mut self) {
        self.statements.pop();
        if self.statements.len() < self.start_of_deductions {
            self.start_of_deductions = self.statements.len();
        }
        match self.statements.last() {
            None => self.current_level = 0,
            Some((l, _)) => self.current_level = *l,
        }
    }

    pub fn introduce_and(&mut self, left: usize, right: usize) -> bool {
        let left_x = match self.statements.get(left) {
            Some(v) => v,
            None => return false,
        };
        let right_x = match self.statements.get(right) {
            Some(v) => v,
            None => return false,
        };

        let ris = Proposition::new_and(left_x.1.unwrap(), right_x.1.unwrap());
        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(ris, Rule::IntroAnd, vec![left, right]),
        ));
        true
    }

    pub fn eliminate_and(&mut self, assum: usize, new_exp: &Rc<Proposition>) -> bool {
        let assum_x = match self.statements.get(assum) {
            Some(v) => v.1.unwrap(),
            None => return false,
        };

        match assum_x.borrow() {
            Proposition::And(left, _) if left == new_exp => {
                self.statements.push((
                    self.current_level,
                    FitchComponent::Deduction(left.clone(), Rule::ElimAnd, vec![assum]),
                ));
                true
            }
            Proposition::And(_, right) if right == new_exp => {
                self.statements.push((
                    self.current_level,
                    FitchComponent::Deduction(right.clone(), Rule::ElimAnd, vec![assum]),
                ));
                true
            }
            _ => false,
        }
    }

    pub fn reiterate(&mut self, row: usize) -> bool {
        if row >= self.statements.len() {
            return false;
        }

        let a = self.statements.get(row).unwrap();
        if a.0 > self.current_level {
            return false;
        }

        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(a.1.unwrap().clone(), Rule::Reiter, vec![row]),
        ));
        true
    }

    pub fn introduce_or(&mut self, assum: usize, prop: &Rc<Proposition>) -> bool {
        let assum_x = match self.statements.get(assum) {
            Some(v) => v.1.unwrap(),
            None => return false,
        };

        match prop.borrow() {
            Proposition::Or(left, right) => {
                if assum_x == left || assum_x == right {
                    self.statements.push((
                        self.current_level,
                        FitchComponent::Deduction(prop.clone(), Rule::IntroOr, vec![assum]),
                    ));
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    // This prolly has some bugs
    fn get_subproof_result(&self, n: usize) -> Option<&Rc<Proposition>> {
        let start = match self.statements.get(n) {
            Some((level, FitchComponent::Assumption(_))) => level,
            _ => return None,
        };

        for x in n + 1..self.statements.len() {
            if x + 1 == self.statements.len() {
                return Some(self.statements.get(x).unwrap().1.unwrap());
            }

            match self.statements.get(x + 1).unwrap() {
                (level, FitchComponent::Assumption(_)) if level <= start => (),
                (level, _) if level < start => (),
                _ => continue,
            }
            return Some(self.statements.get(x).unwrap().1.unwrap());
        }
        None
    }

    pub fn eliminate_or(&mut self, assum: usize, left: usize, right: usize) -> bool {
        if assum >= left || assum >= right {
            return false;
        }

        let assum_x = match self.statements.get(assum) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        let left_a = match self.statements.get(left) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        let right_a = match self.statements.get(right) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        match assum_x.borrow() {
            Proposition::Or(ll, rr) => {
                if !(ll == left_a && rr == right_a) {
                    return false;
                }
            }
            _ => return false,
        }

        let left_sub = self.get_subproof_result(left);
        let right_sub = self.get_subproof_result(right);
        if left_sub.is_none() || left_sub != right_sub {
            return false;
        }
        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(
                left_sub.unwrap().clone(),
                Rule::ElimOr,
                vec![assum, left, right],
            ),
        ));
        true
    }

    pub fn introduce_absurdum(&mut self, ass1: usize, ass2: usize) -> bool {
        let a1 = match self.statements.get(ass1) {
            None => return false,
            Some(v) => v.1.unwrap(),
        };
        let a2 = match self.statements.get(ass2) {
            None => return false,
            Some(v) => v.1.unwrap(),
        };

        let n1 = Proposition::new_not(a1);
        let n2 = Proposition::new_not(a2);
        if !(&n1 == a2 || &n2 == a1) {
            return false;
        }

        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(
                Proposition::new_absurdum(),
                Rule::IntroAbs,
                vec![ass1, ass2],
            ),
        ));
        true
    }

    pub fn eliminate_absurdum(&mut self, absurdum: usize, introduce: &Rc<Proposition>) -> bool {
        match self.statements.get(absurdum) {
            Some((level, m))
                if m.unwrap() == &Proposition::new_absurdum() && *level <= self.current_level =>
            {
                self.statements.push((
                    self.current_level,
                    FitchComponent::Deduction(introduce.clone(), Rule::ElimAbs, vec![absurdum]),
                ));
                true
            }
            _ => false,
        }
    }

    pub fn introduce_not(&mut self, sub_proof: usize) -> bool {
        match self.get_subproof_result(sub_proof) {
            None => return false,
            Some(v) => match v.borrow() {
                Proposition::Absurdum => (),
                _ => return false,
            },
        }

        if self.current_level != self.statements.get(sub_proof).unwrap().0 - 1 {
            return false;
        }

        let cur = self.statements.get(sub_proof).unwrap().1.unwrap();
        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(Proposition::new_not(cur), Rule::IntroNot, vec![sub_proof]),
        ));
        true
    }

    pub fn eliminate_not(&mut self, row: usize) -> bool {
        let cur = match self.statements.get(row) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        let cur = match cur.borrow() {
            Proposition::Not(v) => match v.borrow() {
                Proposition::Not(b) => b,
                _ => return false,
            },
            _ => return false,
        };

        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(cur.clone(), Rule::ElimNot, vec![row]),
        ));
        true
    }

    pub fn introduce_implies(&mut self, sub_proof: usize) -> bool {
        let start = match self.statements.get(sub_proof) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };
        let end = match self.get_subproof_result(sub_proof) {
            None => return false,
            Some(v) => v,
        };

        if self.current_level != self.statements.get(sub_proof).unwrap().0 - 1 {
            return false;
        }

        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(
                Proposition::new_implies(start, end),
                Rule::IntroImpl,
                vec![sub_proof],
            ),
        ));
        true
    }

    pub fn eliminate_implies(&mut self, assum: usize, left: usize) -> bool {
        let assum_x = match self.statements.get(assum) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };
        let left_x = match self.statements.get(left) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        match assum_x.borrow() {
            Proposition::Implies(l, r) if left_x == l => self.statements.push((
                self.current_level,
                FitchComponent::Deduction(r.clone(), Rule::ElimImpl, vec![assum, left]),
            )),
            _ => return false,
        }
        true
    }

    pub fn introduce_iff(&mut self, left_sub: usize, right_sub: usize) -> bool {
        let left_start = match self.statements.get(left_sub) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };
        let left_end = match self.get_subproof_result(left_sub) {
            None => return false,
            Some(v) => v,
        };
        let right_start = match self.statements.get(right_sub) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };
        let right_end = match self.get_subproof_result(right_sub) {
            None => return false,
            Some(v) => v,
        };

        if self.current_level != self.statements.get(left_sub).unwrap().0 - 1
            || self.current_level != self.statements.get(right_sub).unwrap().0 - 1
        {
            return false;
        }

        if !(left_start == right_end && left_end == right_start) {
            return false;
        }
        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(
                Proposition::new_iff(left_start, right_start),
                Rule::IntroIff,
                vec![left_sub, right_sub],
            ),
        ));
        true
    }

    pub fn eliminate_iff(&mut self, assum: usize, truth: usize) -> bool {
        let imp = match self.statements.get(assum) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };
        let truth_x = match self.statements.get(truth) {
            None => return false,
            Some((_, v)) => v.unwrap(),
        };

        let ris = match imp.borrow() {
            Proposition::Iff(left, right) if left == truth_x => right,
            Proposition::Iff(left, right) if right == truth_x => left,
            _ => return false,
        };

        self.statements.push((
            self.current_level,
            FitchComponent::Deduction(ris.clone(), Rule::ElimIff, vec![assum, truth]),
        ));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Fitch, Proposition};

    #[test]
    fn introduce_and() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        fitch.add_assumption(&t0);
        fitch.add_assumption(&t1);
        let ris = fitch.introduce_and(0, 1);
        assert!(ris);
        assert_eq!(
            *fitch.statements.get(2).unwrap().1.unwrap(),
            Proposition::new_and(&t0, &t1)
        );
    }

    #[test]
    fn eliminate_and() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        let prop = Proposition::new_and(&t0, &t1);
        fitch.add_assumption(&prop);
        let ris = fitch.eliminate_and(0, &t0);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t0);
        let ris = fitch.eliminate_and(0, &t1);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t1);
    }

    #[test]
    fn introduce_or() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        fitch.add_assumption(&t0);
        let mut ris = fitch.introduce_or(0, &Proposition::new_or(&t0, &t1));
        assert!(ris);
        assert_eq!(
            fitch.statements.get(1).unwrap().1.unwrap(),
            &Proposition::new_or(&t0, &t1)
        );
        ris = fitch.introduce_or(0, &Proposition::new_or(&t1, &t0));
        assert!(ris);
        assert_eq!(
            fitch.statements.get(2).unwrap().1.unwrap(),
            &Proposition::new_or(&t1, &t0)
        );
    }

    #[test]
    fn eliminate_or() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        let t2 = Proposition::new_term("C");
        let t1_t2 = Proposition::new_or(&t1, &t2);
        fitch.add_assumption(&t0);
        fitch.add_assumption(&t1_t2);
        fitch.add_subproof(&t1);
        fitch.reiterate(0);
        fitch.end_subproof();
        fitch.add_subproof(&t2);
        fitch.reiterate(0);
        fitch.end_subproof();
        let ris = fitch.eliminate_or(1, 2, 4);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t0);
    }

    #[test]
    fn introduce_absurdum() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_not(&t0);
        fitch.add_assumption(&t0);
        fitch.add_assumption(&t1);
        let ris = fitch.introduce_absurdum(0, 1);
        assert!(ris);
        assert_eq!(
            fitch.statements.last().unwrap().1.unwrap(),
            &Proposition::new_absurdum()
        );
    }

    #[test]
    fn eliminate_absurdum() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_absurdum();
        let t1 = Proposition::new_term("A");
        fitch.add_assumption(&t0);
        let ris = fitch.eliminate_absurdum(0, &t1);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t1);
    }

    #[test]
    fn introduce_not() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_absurdum();
        let t1 = Proposition::new_term("A");
        fitch.add_assumption(&t0);
        fitch.add_subproof(&t1);
        fitch.reiterate(0);
        fitch.end_subproof();
        let ris = fitch.introduce_not(1);
        assert!(ris);
        assert_eq!(
            fitch.statements.last().unwrap().1.unwrap(),
            &Proposition::new_not(&t1)
        );
    }

    #[test]
    fn eliminate_not() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_not(&Proposition::new_not(&t0));
        fitch.add_assumption(&t1);
        let ris = fitch.eliminate_not(0);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t0);
    }

    #[test]
    fn introduce_implies() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        fitch.add_subproof(&t1);
        fitch.add_subproof(&t0);
        fitch.reiterate(0);
        fitch.end_subproof();
        let ris = fitch.introduce_implies(1);
        assert!(ris);
        assert_eq!(
            fitch.statements.last().unwrap().1.unwrap(),
            &Proposition::new_implies(&t0, &t1)
        );
    }

    #[test]
    fn eliminate_implies() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        let imp = Proposition::new_implies(&t0, &t1);
        fitch.add_assumption(&t0);
        fitch.add_assumption(&imp);
        let ris = fitch.eliminate_implies(1, 0);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t1);
    }

    #[test]
    fn introduce_iff() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        fitch.add_assumption(&t0);
        fitch.add_assumption(&t1);
        fitch.add_subproof(&t0);
        fitch.reiterate(1);
        fitch.end_subproof();
        fitch.add_subproof(&t1);
        fitch.reiterate(0);
        fitch.end_subproof();
        let ris = fitch.introduce_iff(2, 4);
        assert!(ris);
        assert_eq!(
            fitch.statements.last().unwrap().1.unwrap(),
            &Proposition::new_iff(&t0, &t1)
        );
    }

    #[test]
    fn eliminate_iff() {
        let mut fitch = Fitch::new();
        let t0 = Proposition::new_term("A");
        let t1 = Proposition::new_term("B");
        let bi = Proposition::new_iff(&t0, &t1);
        fitch.add_assumption(&bi);
        fitch.add_assumption(&t0);
        let ris = fitch.eliminate_iff(0, 1);
        assert!(ris);
        assert_eq!(fitch.statements.last().unwrap().1.unwrap(), &t1);
    }
}
