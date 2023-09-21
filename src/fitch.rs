use std::{cell::RefCell, rc::Rc};

struct Fitch {
    root: Rc<RefCell<Node>>,
    lines: Vec<Rc<RefCell<Node>>>,
}

struct Node {
    value: Expression,
    next: Option<Rc<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,
    parent: Option<Rc<RefCell<Node>>>,
}

#[derive(Clone, PartialEq, Debug)]
enum Expression {
    Proposition(String),
    Unary(Unary, Box<Expression>),
    Binary(Binary, Box<Expression>, Box<Expression>),
    Absurdum,
    Empty,
}

#[derive(Clone, PartialEq, Debug)]
enum Unary {
    Not,
}

#[derive(Clone, PartialEq, Debug)]
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
            children: vec![],
            prev: None,
            parent: None,
        }));
        let mut lines = vec![];
        lines.push(root.clone());

        let root = premises
            .iter()
            .fold(root, |acc, x| {
                let res = acc.borrow_mut().add_next(acc.clone(), x.clone());
                lines.push(res);
                return acc;
            });
        return Fitch { root, lines };
    }

    fn empty() -> Self {
        return Fitch { root: Rc::new(RefCell::new(Node::new(Expression::Empty))), lines: vec![] };
    }

    fn add_assumption(&mut self, assumption: Expression) {
        let last = self.lines.last();
        let last = last.unwrap();
        let res = last.borrow_mut().add_child(last.clone(), assumption);
        self.lines.push(res);
    }

    fn introduce(&mut self, sentence: Expression, assumptions: Vec<Rc<RefCell<Node>>>) -> bool {
        let available = self.lines.last().unwrap().borrow().available_sentences();
        match sentence.introduce(available, &assumptions) {
            Err(_) => false,
            Ok(reference) => todo!(),  // TODO
        }
    }
}

impl Node {
    fn new(value: Expression) -> Self {
        return Node { value, next: None, children: vec![], prev: None, parent: None };
    }

    fn add_next(&mut self, self_rc: Rc<RefCell<Node>>, next: Expression) -> Rc<RefCell<Node>> {
        if let Some(x) = &self.next {
            return x.borrow_mut().add_next(self_rc, next);
        } else {
            let result = RefCell::new(Node::new(next));
            result.borrow_mut().prev = Some(self_rc);
            let result = Rc::new(result);
            let res = result.clone();
            self.next = Some(result);
            return res;
        }
    }

    fn add_child(&mut self, self_rc: Rc<RefCell<Node>>, child: Expression) -> Rc<RefCell<Node>> {
        let res = Rc::new(RefCell::new(Node::new(child)));
        res.borrow_mut().parent = Some(self_rc);
        self.children.push(res.clone());
        return res;
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

    fn internal_available_sentences(&self, self_rc: Option<Rc<RefCell<Node>>>) -> Vec<Expression> {
        if self.prev.is_some() {
            let prev = self.prev.as_ref().unwrap();
            let mut res = prev.borrow().internal_available_sentences(Some(prev.clone()));
            if let Some(rc) = self_rc {
                res.push(rc.borrow().value().clone());
            }
            return res;
        } else if self.parent.is_some() {
            let parent = self.parent.as_ref().unwrap();
            let mut res = parent.borrow().internal_available_sentences(Some(parent.clone()));
            if let Some(rc) = self_rc {
                res.push(rc.borrow().value().clone());
            }
            return res;
        } else {
            let mut res = vec![];
            if let Some(rc) = self_rc {
                res.push(rc.borrow().value().clone());
            }
            return res;
        }
    }

    fn available_sentences(&self) -> Vec<Expression> {
        return self.internal_available_sentences(None);
    }
}

impl Expression {
    fn is_available(available: &Vec<Expression>, exp: &Expression) -> bool {
        return available.contains(exp);
    }

    fn introduce(&self, available: Vec<Expression>, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        match self {
            Self::Binary(oper, left, right) => {
                if !(Self::is_available(&available, left) && Self::is_available(&available, right)) {
                    return Err(());
                }
                let left = left.as_ref().clone();
                let right = right.as_ref().clone();
                match oper {
                    Binary::And => Binary::introduce_and((left, right), assumptions),
                    Binary::Or => Binary::introduce_or((left, right), assumptions),
                    Binary::Conditional => Binary::introduce_condition((left, right), assumptions),
                    Binary::Biconditional => Binary::introduce_bicondition((left, right), assumptions),
                }
            },
            Self::Unary(Unary::Not, center) => {
                if !Self::is_available(&available, center) {
                    return Err(());
                }
                return Unary::introduce_not(center, assumptions);
            }
            _ => todo!(),
        }
    }
}

impl Unary {
    fn introduce_not(center: &Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 1 { return Err(()); }
        let res = Expression::Unary(Unary::Not, Box::new(assumptions[0].borrow().value().clone()));
        if *center != res { return Err(()) };
        if assumptions[0].borrow().last_expression() != Expression::Absurdum { return Err(()); }
        return Ok(Rc::new(RefCell::new(Node::new(res))));
    }
}

impl Binary {
    fn generate_node(operator: Self, left: Expression, right: Expression) -> Rc<RefCell<Node>> {
        return Rc::new(RefCell::new(Node::new(Expression::Binary(operator, Box::new(left), Box::new(right)))));
    }

    fn introduce_and(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 2 { return Err(()); }
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
        if assumptions.len() != 1 { return Err(()); };
        let (left, right) = operands;
        let value = assumptions[0].borrow();
        if left == *value.value() || right == *value.value() {
            return Ok(Self::generate_node(Self::Or, left, right));
        }
        return Err(());
    }

    fn introduce_condition(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 1 { return Err(()) }
        let (left, right) = operands;
        if left == *assumptions[0].borrow().value() && right == assumptions[0].borrow().last_expression() {
            return Ok(Self::generate_node(Self::Conditional, left, right));
        }
        return Err(());
    }

    fn introduce_bicondition(operands: (Expression, Expression), assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 2 { return Err(()); }
        let (left, right) = operands;
        let a1 = assumptions[0].borrow().value().clone();
        let a2 = assumptions[0].borrow().last_expression();
        let b1 = assumptions[1].borrow().value().clone();
        let b2 = assumptions[1].borrow().last_expression();
        if !(a1 == b2 && a2 == b1) {return Err(()); }

        let p1 = left == a1 && right == a2;
        let p2 = left == b1 && right == b2;
        if p1 || p2 {
            return Ok(Self::generate_node(Self::Biconditional, left, right));
        }
        return Err(());
    }
}

mod tests {
    use std::{rc::Rc, cell::RefCell};

    use super::{Expression, Node};

    fn make_node(exp: Expression) -> Rc<RefCell<Node>> {
        return Rc::new(RefCell::new(Node::new(exp)));
    }

    #[cfg(test)]
    mod test_node {
        use std::{rc::Rc, cell::RefCell, vec};

        use crate::fitch::{Node, Expression, Binary};
    
        #[test]
        fn available_sentences() {
            let last = Expression::Absurdum;
            let first = Rc::new(RefCell::new(Node::new(Expression::Binary(
                    Binary::And,
                    Box::new(Expression::Proposition(String::from("A"))),
                    Box::new(Expression::Proposition(String::from("B"))))
            )));
            let second = Expression::Binary(
                    Binary::Or,
                    Box::new(Expression::Proposition(String::from("A"))),
                    Box::new(Expression::Proposition(String::from("B")))
            );
            let third = Expression::Binary(
                    Binary::Or,
                    Box::new(Expression::Proposition(String::from("A"))),
                    Box::new(Expression::Proposition(String::from("B")))
            );

            let second = first.borrow_mut().add_child(first.clone(), second);
            let third = second.borrow_mut().add_child(second.clone(), third);
            let last = third.borrow_mut().add_next(third.clone(), last);

            let available = last.borrow().available_sentences();
            let oracle = vec![
                first.borrow().value().clone(),
                second.borrow().value().clone(),
                third.borrow().value().clone(),
            ];

            assert_eq!(available, oracle);
        }
    }
    
    #[cfg(test)]
    mod test_binary {
        use std::{rc::Rc, cell::RefCell};

        use crate::fitch::{Expression, Node, Binary, tests::make_node};
    
        #[test]
        fn introduce_and() {
            let operands = (
                Expression::Proposition(String::from("A")),
                Expression::Proposition(String::from("B"))
            );
            let vv = Binary::introduce_and(
                operands.clone(),
                &vec![make_node(operands.0.clone()), make_node(operands.1.clone())]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_and(
                operands.clone(),
                &vec![make_node(operands.0.clone()), make_node(Expression::Empty)]
            );
            assert!(vv.is_err());
    
            let vv = Binary::introduce_and(
                operands.clone(),
                &vec![make_node(Expression::Empty), make_node(operands.1.clone())]
            );
            assert!(vv.is_err());
    
            let vv = Binary::introduce_and(
                operands.clone(),
                &vec![make_node(operands.1.clone()), make_node(operands.0.clone())]
            );
            assert!(vv.is_ok());
        }
    
        #[test]
        fn introduce_or() {
            let operands = (
                Expression::Proposition(String::from("A")),
                Expression::Proposition(String::from("B"))
            );
            let vv = Binary::introduce_or(
                operands.clone(),
                &vec![make_node(operands.0.clone())]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_or(
                operands.clone(),
                &vec![make_node(operands.1.clone())]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_or(
                operands.clone(),
                &vec![make_node(Expression::Absurdum)]
            );
            assert!(vv.is_err());
        }
    
        #[test]
        fn introduce_condition() {
            let operands = (
                Expression::Proposition(String::from("A")),
                Expression::Proposition(String::from("B"))
            );
    
            let assump = make_node(operands.0.clone());
            assump.borrow_mut().add_next(assump.clone(), operands.1.clone());
    
            let vv = Binary::introduce_condition(
                operands.clone(),
                &vec![assump]
            );
            assert!(vv.is_ok());
    
            let assump = make_node(operands.0.clone());
            assump.borrow_mut().add_next(assump.clone(), Expression::Absurdum);
    
            let vv = Binary::introduce_condition(
                operands.clone(),
                &vec![assump]
            );
            assert!(vv.is_err());
    
            let assump = make_node(Expression::Empty);
            assump.borrow_mut().add_next(assump.clone(), operands.1.clone());
    
            let vv = Binary::introduce_condition(
                operands.clone(),
                &vec![assump]
            );
            assert!(vv.is_err());
        }

        #[test]
        fn introduce_bicondition() {
            let operands = (
                Expression::Proposition(String::from("A")),
                Expression::Proposition(String::from("B"))
            );
            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), operands.1.clone());
            let assump2 = make_node(operands.1.clone());
            assump2.borrow_mut().add_next(assump2.clone(), operands.0.clone());

            let vv = Binary::introduce_bicondition(
                operands.clone(),
                &vec![assump1, assump2]
            );
            assert!(vv.is_ok());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), operands.1.clone());
            let assump2 = make_node(operands.1.clone());
            assump2.borrow_mut().add_next(assump2.clone(), Expression::Absurdum);

            let vv = Binary::introduce_bicondition(
                operands.clone(),
                &vec![assump1, assump2]
            );
            assert!(vv.is_err());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), Expression::Absurdum);
            let assump2 = make_node(operands.1.clone());
            assump2.borrow_mut().add_next(assump2.clone(), operands.0.clone());

            let vv = Binary::introduce_bicondition(
                operands.clone(),
                &vec![assump1, assump2]
            );
            assert!(vv.is_err());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), Expression::Absurdum);

            let vv = Binary::introduce_bicondition(
                operands.clone(),
                &vec![assump1.clone(), assump1]
            );
            assert!(vv.is_err());
        }
    }

    #[cfg(test)]
    mod test_unary {
        use crate::fitch::{Expression, Unary};

        use super::make_node;

        #[test]
        fn introduce_not() {
            let assump = Expression::Proposition(String::from("A"));
            let operand = make_node(assump.clone());
            operand.borrow_mut().add_next(operand.clone(), Expression::Absurdum);

            let vv = Unary::introduce_not(
                &Expression::Unary(Unary::Not, Box::new(assump.clone())),
                &vec![operand.clone()]
            );
            assert!(vv.is_ok());

            let operand = make_node(assump.clone());
            operand.borrow_mut().add_next(operand.clone(), Expression::Proposition(String::from("B")));

            let vv = Unary::introduce_not(
                &Expression::Unary(Unary::Not, Box::new(assump.clone())),
                &vec![operand.clone()]
            );
            assert!(vv.is_err());

            let operand = make_node(assump.clone());
            operand.borrow_mut().add_next(operand.clone(), Expression::Absurdum);

            let vv = Unary::introduce_not(
                &Expression::Unary(Unary::Not, Box::new(Expression::Proposition(String::from("B")))),
                &vec![operand.clone()]
            );
            assert!(vv.is_err());
        }
    }
}
