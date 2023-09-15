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

#[derive(Clone, PartialEq)]
enum Expression {
    Proposition(Box<str>),
    Unary(Unary, Box<Expression>),
    Binary(Binary, Box<Expression>, Box<Expression>),
    Absurdum,
}

#[derive(Clone, PartialEq)]
enum Unary {
    Not,
}

#[derive(Clone, PartialEq)]
enum Binary {
    And,
    Or,
    Conditional,
    Biconditional,
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
        match sentence.introduce(&assumptions) {
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

    fn value(&self) -> &Expression {
        return &self.value;
    }

    fn last_expression(&self) -> Expression {
        if self.next.is_none() {
            return self.value().clone();
        }
        return self.next.as_ref().unwrap().borrow().last_expression();
    }
}

impl Expression {
    fn introduce(&self, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        match self {
            Self::Binary(Binary::And, left, right) => {
                return Binary::introduce_and((left.as_ref().clone(), right.as_ref().clone()), assumptions);
            },
            Self::Binary(Binary::Or, left, right) => {
                return Binary::introduce_or((left.as_ref().clone(), right.as_ref().clone()), assumptions);
            },
            _ => todo!(),
        }
    }
}

impl Binary {
    fn generate_node(operator: Self, left: Expression, right: Expression) -> Rc<RefCell<Node>> {
        return Rc::new(RefCell::new(Node::new(Expression::Binary(operator, Box::new(left), Box::new(right)))));
    }

    fn introduce_and(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 2 { return Err(()) };
        let (left, right) = operands;
        let l = assumptions[0].as_ref().borrow();
        let r = assumptions[1].as_ref().borrow();
        let a = left == *l.value() && right == *r.value();
        let b = right == *l.value() && left == *r.value();
        if a || b {
            return Ok(Self::generate_node(Self::And, left, right));
        }
        return Err(());
    }

    fn introduce_or(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 1 { return Err(()) };
        let (left, right) = operands;
        let value = assumptions[0].borrow();
        if left == *value.value() || right == *value.value() {
            return Ok(Self::generate_node(Self::Or, left, right));
        }
        return Err(());
    }

    fn introduce_condition(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 1 {
            return Err(());
        }
        let (left, right) = operands;
        if left == *assumptions[0].borrow().value() && right == assumptions[0].borrow().last_expression() {
            return Ok(Self::generate_node(Self::Conditional, left, right));
        }
        return Err(());
    }
}
