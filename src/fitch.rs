use std::{cell::RefCell, rc::Rc};

struct Fitch {
    root: Rc<RefCell<Node>>,
    lines: Vec<Expression>,
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

impl Fitch {
    fn new(mut assumptions: Vec<Expression>) -> Self {
        let root = Rc::new(RefCell::new(Node {
            value: assumptions.remove(0),
            next: None,
            child: None,
        }));
        let mut lines = vec![];
        lines.push(root.borrow().value.clone());

        let root = assumptions
            .iter()
            .fold(root, |acc, x| {
                acc.borrow_mut().add_next(x.clone());
                lines.push(x.clone());
                return acc;
            });
        return Fitch { root, lines };
    }
}

impl Node {
    fn new(value: Expression) -> Self {
        return Node { value, next: None, child: None };
    }

    fn add_next(&mut self, next: Expression) {
        if let Some(x) = &self.next {
            x.borrow_mut().add_next(next);
        } else {
            self.next = Some(Rc::new(RefCell::new(Node::new(next))));
        }
    }
}
