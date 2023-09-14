use std::{cell::RefCell, rc::Rc};

struct Fitch {
    root: Option<Rc<RefCell<Node>>>,
    lines: Vec<Rc<RefCell<Node>>>,
}

struct Node {
    value: Expression,
    next: Option<Rc<RefCell<Node>>>,
    child: Option<Rc<RefCell<Node>>>,
}

#[derive(Clone)]
enum Expression {
    Proposition(Box<str>),
    Unary(Unary, Box<Expression>),
    Binary(Binary, Box<Expression>, Box<Expression>),
    Absurdum,
}

#[derive(Clone)]
enum Unary {
    Not,
}

#[derive(Clone)]
enum Binary {
    And,
    Or,
    Conditional,
    Biconditional,
}

trait Operation {
    fn introduce(&self, assumptions: Vec<Rc<RefCell<Node>>>);
}

impl Fitch {
    fn new(mut premises: Vec<Expression>) -> Self {
        let root = Rc::new(RefCell::new(Node {
            value: premises.remove(0),
            next: None,
            child: None,
        }));
        let mut lines = vec![];
        lines.push(root.clone());

        let root = premises
            .iter()
            .fold(root, |acc, x| {
                let res = acc.borrow_mut().add_next(x.clone());
                lines.push(res);
                return acc;
            });
        return Fitch { root: Some(root), lines };
    }

    fn empty() -> Self {
        return Fitch { root: None, lines: vec![] };
    }

    // Didn't plan ahead enough, this implementation can't have empty premises
    fn add_assumption(&mut self, assumption: Expression) {
        let last = self.lines.last();
        if last.is_none() {
            let exp = Rc::new(RefCell::new(Node::new(assumption)));
            self.root = Some(exp.clone());
            self.lines.push(exp);
        } else {
            let res = last.unwrap().borrow_mut().add_child(assumption);
            self.lines.push(res);
        }
    }

    fn introduce(&mut self, sentence: Expression, assumptions: Vec<Rc<RefCell<Node>>>) -> bool {
        match sentence.introduce(&sentence, &assumptions) {
            Err(_) => false,
            Ok(reference) => todo!(),
        }
    }
}

impl Node {
    fn new(value: Expression) -> Self {
        return Node { value, next: None, child: None };
    }

    fn add_next(&mut self, next: Expression) -> Rc<RefCell<Node>> {
        if let Some(x) = &self.next {
            return x.borrow_mut().add_next(next);
        } else {
            let result = Rc::new(RefCell::new(Node::new(next)));
            let res = result.clone();
            self.next = Some(result);
            return res;
        }
    }

    fn add_child(&mut self, child: Expression) -> Rc<RefCell<Node>> {
        if let Some(x) = &self.child {
            return x.borrow_mut().add_child(child);
        } else {
            let result = Rc::new(RefCell::new(Node::new(child)));
            let res = result.clone();
            self.child = Some(result);
            return res;
        }
    }
}

impl Expression {
    fn introduce(&self, sentence: &Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        todo!()
    }
}

impl Operation for Binary {
    fn introduce(&self, assumptions: Vec<Rc<RefCell<Node>>>) {
        todo!()
    }
}

impl Operation for Unary {
    fn introduce(&self, assumptions: Vec<Rc<RefCell<Node>>>) {
        todo!()
    }
}
