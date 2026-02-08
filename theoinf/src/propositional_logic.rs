use std::collections::HashMap;

use winnow::Parser;
use winnow::Result;
use winnow::stream::AsChar;
use winnow::token::take_while;

#[derive(PartialEq, Debug)]
pub enum Expr<'a> {
    Var { name: &'a str },
    Not { x: &'a Expr<'a> },
    Or { x: &'a Expr<'a>, y: &'a Expr<'a> },
    And { x: &'a Expr<'a>, y: &'a Expr<'a> },
    True,
    False,
}

pub fn parse_var<'s>(input: &mut &'s str) -> winnow::Result<Expr<'s>> {
    match take_while(0.., AsChar::is_alpha).parse_next(input) {
        Result::Ok(v) => Ok(Expr::Var { name: v }),
        Result::Err(e) => Result::Err(e),
    }
}

pub fn eval(assignment: &HashMap<&str, bool>, expr: &Expr) -> bool {
    match expr {
        Expr::Var { name: x } => assignment[x],
        Expr::Not { x } => !eval(assignment, x),
        Expr::Or { x, y } => eval(assignment, x) || eval(assignment, y),
        Expr::And { x, y } => eval(assignment, x) && eval(assignment, y),
        Expr::True => true,
        Expr::False => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_a_var_works() {
        let mut input = "a";
        let expr = parse_var(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::Var { name: "a" }, expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn assignment_works() {
        let a = Expr::Var { name: "a" };
        let b = Expr::Var { name: "b" };
        let mut assignment = HashMap::new();
        assignment.insert("a", false);
        assignment.insert("b", true);
        assert!(!eval(&assignment, &a));
        assert!(eval(&assignment, &b))
    }

    #[test]
    fn not_works() {
        let assignment = HashMap::new();
        let not_true = Expr::Not { x: &Expr::True };
        let not_false = Expr::Not { x: &Expr::False };
        assert!(!eval(&assignment, &not_true));
        assert!(eval(&assignment, &not_false))
    }

    #[test]
    fn or_works() {
        let assignment = HashMap::new();

        let expr = Expr::Or {
            x: &Expr::False,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::False,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::True,
            y: &Expr::False,
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::True,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));
    }

    #[test]
    fn and_works() {
        let assignment = HashMap::new();

        let expr = Expr::And {
            x: &Expr::False,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::False,
            y: &Expr::True,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::True,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::True,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));
    }
}
