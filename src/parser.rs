use crate::fitch::Proposition;
use std::rc::Rc;

enum Result<'a> {
    Success(Rc<Proposition>, &'a str),
    Failure,
}

fn parse_absurdum(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '#' {
        Result::Failure
    } else {
        Result::Success(Proposition::new_absurdum(), &queue[1..])
    }
}

fn parse_term(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() {
        return Result::Failure;
    }

    let mut buffer = String::new();
    let mut index = 0;
    for (i, x) in queue.chars().enumerate() {
        if x >= 'A' && x <= 'Z' {
            buffer.push(x);
        } else {
            index = i;
            break;
        }
    }

    if buffer.is_empty() {
        Result::Failure
    } else {
        Result::Success(Proposition::new_term(&buffer), &queue[index..])
    }
}

fn parse_and(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '(' {
        return Result::Failure;
    }
    let queue = queue[1..].trim_start();

    match parse_term(queue) {
        Result::Failure => Result::Failure,
        Result::Success(left, rest) => {
            let queue = rest.trim_start();
            if queue.is_empty() || queue.chars().nth(0).unwrap() != '^' {
                return Result::Failure;
            }
            let queue = queue[1..].trim_start();

            match parse_term(queue) {
                Result::Failure => Result::Failure,
                Result::Success(right, rest) => {
                    let queue = rest.trim_start();
                    if queue.is_empty() || queue.chars().nth(0).unwrap() != ')' {
                        return Result::Failure;
                    }

                    let queue = &queue[1..];
                    Result::Success(Proposition::new_and(&left, &right), queue)
                }
            }
        }
    }
}

fn parse_or(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '(' {
        return Result::Failure;
    }
    let queue = queue[1..].trim_start();

    match parse_term(queue) {
        Result::Failure => Result::Failure,
        Result::Success(left, rest) => {
            let queue = rest.trim_start();
            if queue.is_empty() || queue.chars().nth(0).unwrap() != 'v' {
                return Result::Failure;
            }
            let queue = queue[1..].trim_start();

            match parse_term(queue) {
                Result::Failure => Result::Failure,
                Result::Success(right, rest) => {
                    let queue = rest.trim_start();
                    if queue.is_empty() || queue.chars().nth(0).unwrap() != ')' {
                        return Result::Failure;
                    }

                    let queue = &queue[1..];
                    Result::Success(Proposition::new_or(&left, &right), queue)
                }
            }
        }
    }
}

fn parse_not(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '~' {
        return Result::Failure;
    }
    let queue = queue[1..].trim_start();

    match parse_term(queue) {
        Result::Failure => Result::Failure,
        Result::Success(t, rest) => Result::Success(Proposition::new_not(&t), rest),
    }
}

fn parse_implies(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '(' {
        return Result::Failure;
    }
    let queue = queue[1..].trim_start();

    match parse_term(queue) {
        Result::Failure => Result::Failure,
        Result::Success(left, rest) => {
            let queue = rest.trim_start();
            if queue.len() < 2 || &queue[0..2] != "=>" {
                return Result::Failure;
            }
            let queue = queue[2..].trim_start();

            match parse_term(queue) {
                Result::Failure => Result::Failure,
                Result::Success(right, rest) => {
                    let queue = rest.trim_start();
                    if queue.is_empty() || queue.chars().nth(0).unwrap() != ')' {
                        return Result::Failure;
                    }
                    let queue = &queue[1..];
                    Result::Success(Proposition::new_implies(&left, &right), queue)
                }
            }
        }
    }
}

fn parse_iff(queue: &str) -> Result {
    let queue = queue.trim_start();
    if queue.is_empty() || queue.chars().nth(0).unwrap() != '(' {
        return Result::Failure;
    }
    let queue = queue[1..].trim_start();

    match parse_term(queue) {
        Result::Failure => Result::Failure,
        Result::Success(left, rest) => {
            let queue = rest.trim_start();
            if queue.len() < 3 || &queue[0..3] != "<=>" {
                return Result::Failure;
            }
            let queue = queue[3..].trim_start();

            match parse_term(queue) {
                Result::Failure => Result::Failure,
                Result::Success(right, rest) => {
                    let queue = rest.trim_start();
                    if queue.is_empty() || queue.chars().nth(0).unwrap() != ')' {
                        return Result::Failure;
                    }
                    let queue = &queue[1..];
                    Result::Success(Proposition::new_iff(&left, &right), queue)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_absurdum, parse_and, parse_iff, parse_implies, parse_not, parse_or, parse_term, Result};
    use crate::fitch::Proposition;

    #[test]
    fn parse_absurdum_test() {
        let queue = "  #  ";
        match parse_absurdum(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_absurdum());
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_term_test() {
        let queue = "  CIAO  ";
        match parse_term(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_term("CIAO"));
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_and_test() {
        let queue = "  (A ^ B)  ";
        let a = Proposition::new_term("A");
        let b = Proposition::new_term("B");
        match parse_and(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_and(&a, &b));
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_or_test() {
        let queue = "  (A v B)  ";
        let a = Proposition::new_term("A");
        let b = Proposition::new_term("B");
        match parse_or(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_or(&a, &b));
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_not_test() {
        let queue = "  ~A  ";
        let a = Proposition::new_term("A");
        match parse_not(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_not(&a));
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_implies_test() {
        let queue = "  (A => B)  ";
        let a = Proposition::new_term("A");
        let b = Proposition::new_term("B");
        match parse_implies(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_implies(&a, &b));
                assert_eq!(rest, "  ");
            }
        }
    }

    #[test]
    fn parse_iff_test() {
        let queue = "  (A <=> B)  ";
        let a = Proposition::new_term("A");
        let b = Proposition::new_term("B");
        match parse_iff(queue) {
            Result::Failure => assert!(false),
            Result::Success(p, rest) => {
                assert_eq!(p, Proposition::new_iff(&a, &b));
                assert_eq!(rest, "  ");
            }
        }
    }
}
