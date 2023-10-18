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

enum Operation {
    IntroAnd,
    IntroOr,
    IntroNot,
    IntroConditional,
    IntorBiconditional,
    IntroAbsurdum,
    ElimAnd,
    ElimOr,
    ElimNot,
    ElimConditional,
    ElimBiconditional,
    ElimAbsurdum,
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

    // TODO check for availability
    fn from_operation(&mut self, sentence: Expression, operation: Operation, assumptions: &Vec<Rc<RefCell<Node>>>) -> bool {
        let new_sentence = match operation {
            Operation::IntroAnd => Binary::introduce_and(assumptions),
            Operation::IntroOr => Binary::introduce_or(sentence, assumptions),
            Operation::IntroConditional => Binary::introduce_condition(assumptions),
            Operation::IntorBiconditional => Binary::introduce_bicondition(assumptions),
            Operation::IntroAbsurdum => Expression::introduce_absurbdum(assumptions),
            _ => todo!()
        };

        if !new_sentence.is_ok() {
            return false;
        }

        let new_last = self.root.borrow_mut().add_next(self.lines.last().unwrap().clone(), new_sentence.unwrap());
        self.lines.push(new_last);
        return true;
    }

    // TODO remove this function
    fn internal_introduce(exp: Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        match exp {
            Expression::Binary(oper, left, right) => {
                let left = left.as_ref().clone();
                let right = right.as_ref().clone();
                match oper {
                    //Binary::And => Binary::introduce_and(assumptions),
                    //Binary::Or => Binary::introduce_or((left, right), assumptions),
                    //Binary::Conditional => Binary::introduce_condition((left, right), assumptions),
                    //Binary::Biconditional => Binary::introduce_bicondition((left, right), assumptions),
                    _ => unreachable!()
                }
            },
            Expression::Unary(Unary::Not, center) => Unary::introduce_not(center.as_ref(), assumptions),
            // TODO write test for absurdum introduction
            Expression::Absurdum => {
                let a1 = assumptions[0].borrow().value().clone();
                let a2 = assumptions[1].borrow().value().clone();
                let p1 = Expression::Unary(Unary::Not, Box::new(a1.clone()));
                let p2 = Expression::Unary(Unary::Not, Box::new(a2.clone()));
                if a1 == p2 || a2 == p1 {
                    return Ok(Rc::new(RefCell::new(Node::new(exp.clone()))));
                }
                return Err(());
            },
            _ => Err(()),
        }
    }

    // TODO remove this function
    fn internal_eliminate(exp: Expression, operation: Expression, assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Rc<RefCell<Node>>, ()> {
        match exp {
            // TODO write test for absurdum elimination
            Expression::Absurdum => {
                if !(*assumptions[0].borrow().value() == Expression::Absurdum) {
                    return Err(());
                }
                return Ok(Rc::new(RefCell::new(Node::new(exp))));
            },
            Expression::Unary(Unary::Not, center) => Unary::eliminate_not(center.as_ref(), assumptions),
            _ => todo!(),
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
    fn introduce_absurbdum(assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Expression, ()> {
        if assumptions.len() != 2 { return Err(()) };
        let left = assumptions[0].borrow();
        let right = assumptions[1].borrow();
        if Expression::Unary(Unary::Not, Box::new(left.value().clone())) == *right.value() {
            return Ok(Expression::Absurdum);
        } else if Expression::Unary(Unary::Not, Box::new(right.value().clone())) == *left.value() {
            return Ok(Expression::Absurdum);
        }
        return Err(());
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

    fn eliminate_not(center: &Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        match assumptions[0].borrow().value() {
            Expression::Unary(Unary::Not, inner) => {
                match inner.as_ref() {
                    Expression::Unary(Unary::Not, exp) => {
                        if *exp.as_ref() == *center {
                            return Ok(Rc::new(RefCell::new(Node::new(center.clone()))));
                        }
                        return Err(());
                    },
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}

impl Binary {
    fn generate_node(operator: Self, left: Expression, right: Expression) -> Rc<RefCell<Node>> {
        return Rc::new(RefCell::new(Node::new(Expression::Binary(operator, Box::new(left), Box::new(right)))));
    }

    fn introduce_and(assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Expression, ()> {
        if assumptions.len() != 2 { return Err(()); }
        let left = assumptions[0].as_ref().borrow();
        let right = assumptions[1].as_ref().borrow();
        return Ok(Expression::Binary(Binary::And, Box::new(left.value().clone()), Box::new(right.value().clone())));
    }

    fn introduce_or(sentence: Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Expression, ()> {
        if assumptions.len() != 1 { return Err(()); };
        match sentence.clone() {
            Expression::Binary(Binary::Or, left, right) => {
                if *left == *assumptions[0].borrow().value() || *right == *assumptions[0].borrow().value() {
                    return Ok(sentence);
                }
                return Err(());
            },
            _ => Err(())
        }
    }

    fn introduce_condition(assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Expression, ()> {
        if assumptions.len() != 1 { return Err(()) }
        let cond = assumptions[0].borrow().value().clone();
        let res = assumptions[0].borrow().last_expression().clone();
        return Ok(Expression::Binary(Binary::Conditional, Box::new(cond), Box::new(res)));
    }

    fn introduce_bicondition(assumptions: &Vec<Rc<RefCell<Node>>>)
    -> Result<Expression, ()> {
        if assumptions.len() != 2 { return Err(()); }
        let a1 = assumptions[0].borrow().value().clone();
        let a2 = assumptions[0].borrow().last_expression();
        let b1 = assumptions[1].borrow().value().clone();
        let b2 = assumptions[1].borrow().last_expression();
        if !(a1 == b2 && a2 == b1) { return Err(()); }
        return Ok(Expression::Binary(Binary::Biconditional, Box::new(a1), Box::new(a2)));
    }

    fn eliminate_and(exp: &Expression, assumptions: &Vec<Rc<RefCell<Node>>>) -> Result<Rc<RefCell<Node>>, ()> {
        if assumptions.len() != 1 {
            return Err(());
        }
        match assumptions[0].borrow().value() {
            Expression::Binary(Binary::And, left, right) => {
                if left.as_ref() == exp || right.as_ref() == exp {
                    return Ok(Rc::new(RefCell::new(Node::new(exp.clone()))));
                }
                return Err(());
            },
            _ => return Err(()),
        }
    }
}

mod tests {
    use std::{rc::Rc, cell::RefCell};

    use super::{Expression, Node};

    fn make_node(exp: Expression) -> Rc<RefCell<Node>> {
        return Rc::new(RefCell::new(Node::new(exp)));
    }

    #[cfg(test)]
    mod test_expression {
        use crate::fitch::{Expression, Unary};

        use super::make_node;

        #[test]
        fn introduce_absurbdum() {
            let p1 = Expression::Proposition(String::from("A"));
            let p2 = Expression::Unary(
                Unary::Not,
                Box::new(Expression::Proposition(String::from("A")))
            );

            let vv = Expression::introduce_absurbdum(
                &vec![make_node(p1), make_node(p2)]
            );
            assert!(vv.is_ok());

            let p2 = Expression::Proposition(String::from("A"));
            let p1 = Expression::Unary(
                Unary::Not,
                Box::new(Expression::Proposition(String::from("A")))
            );

            let vv = Expression::introduce_absurbdum(
                &vec![make_node(p1), make_node(p2)]
            );
            assert!(vv.is_ok());

            let p1 = Expression::Proposition(String::from("A"));
            let p2 = Expression::Proposition(String::from("B"));

            let vv = Expression::introduce_absurbdum(
                &vec![make_node(p1), make_node(p2)]
            );
            assert!(vv.is_err());
        }
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
                &vec![make_node(operands.0.clone()), make_node(operands.1.clone())]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_and(
                &vec![make_node(operands.0.clone()), make_node(Expression::Empty), make_node(Expression::Empty)]
            );
            assert!(vv.is_err());
    
            let vv = Binary::introduce_and(
                &vec![make_node(operands.1.clone()), make_node(operands.0.clone())]
            );
            assert!(vv.is_ok());
        }
    
        #[test]
        fn introduce_or() {
            let operand = Expression::Binary(
                Binary::Or,
                Box::new(Expression::Proposition(String::from("A"))),
                Box::new(Expression::Proposition(String::from("B")))
            );
            let vv = Binary::introduce_or(
                operand.clone(),
                &vec![make_node(Expression::Proposition(String::from("A")))]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_or(
                operand.clone(),
                &vec![make_node(Expression::Proposition(String::from("B")))]
            );
            assert!(vv.is_ok());
    
            let vv = Binary::introduce_or(
                operand.clone(),
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
                &vec![assump.clone()]
            );
            assert!(vv.is_ok());

            let vv = Binary::introduce_condition(
                &vec![assump, make_node(Expression::Absurdum)]
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
                &vec![assump1, assump2]
            );
            assert!(vv.is_ok());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), operands.1.clone());
            let assump2 = make_node(operands.1.clone());
            assump2.borrow_mut().add_next(assump2.clone(), Expression::Absurdum);

            let vv = Binary::introduce_bicondition(
                &vec![assump1, assump2]
            );
            assert!(vv.is_err());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), Expression::Absurdum);
            let assump2 = make_node(operands.1.clone());
            assump2.borrow_mut().add_next(assump2.clone(), operands.0.clone());

            let vv = Binary::introduce_bicondition(
                &vec![assump1, assump2]
            );
            assert!(vv.is_err());

            let assump1 = make_node(operands.0.clone());
            assump1.borrow_mut().add_next(assump1.clone(), Expression::Absurdum);

            let vv = Binary::introduce_bicondition(
                &vec![assump1.clone(), assump1]
            );
            assert!(vv.is_err());
        }

        #[test]
        fn eliminate_and() {
            let intt = Expression::Proposition(String::from("A"));
            let vv = Binary::eliminate_and(
                &intt,
                &vec![make_node(Expression::Binary(Binary::And, Box::new(intt.clone()), Box::new(Expression::Absurdum)))]
            );
            assert!(vv.is_ok());

            let vv = Binary::eliminate_and(
                &intt,
                &vec![make_node(Expression::Binary(Binary::And, Box::new(Expression::Absurdum), Box::new(intt.clone())))]
            );
            assert!(vv.is_ok());

            let vv = Binary::eliminate_and(
                &intt,
                &vec![make_node(Expression::Binary(Binary::And, Box::new(Expression::Absurdum), Box::new(Expression::Absurdum)))]
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

        #[test]
        fn eliminate_not() {
            let assump = Expression::Proposition(String::from("A"));
            let vv = Unary::eliminate_not(
                &assump,
                &vec![make_node(Expression::Unary(
                    Unary::Not,
                    Box::new(Expression::Unary(Unary::Not, Box::new(assump.clone())))
                ))]
            );
            assert!(vv.is_ok());

            let vv = Unary::eliminate_not(
                &assump,
                &vec![make_node(Expression::Unary(
                    Unary::Not,
                    Box::new(assump.clone())
                ))]
            );
            assert!(vv.is_err());

            let vv = Unary::eliminate_not(
                &assump,
                &vec![make_node(Expression::Unary(
                    Unary::Not,
                    Box::new(Expression::Unary(Unary::Not, Box::new(Expression::Absurdum)))
                ))]
            );
            assert!(vv.is_err());
        }
    }
}
