use std::collections::HashMap;

use winnow::Parser;
use winnow::Result;
use winnow::stream::AsChar;
use winnow::token::literal;
use winnow::token::take_while;

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Var { name: String },
    Not { x: Box<Expr> },
    Or { x: Box<Expr>, y: Box<Expr> },
    And { x: Box<Expr>, y: Box<Expr> },
    True,
    False,
}

pub fn parse_var(input: &mut &str) -> winnow::Result<Expr> {
    match take_while(0.., AsChar::is_alpha).parse_next(input) {
        Result::Ok(v) => Ok(Expr::Var {
            name: v.to_string(),
        }),
        Result::Err(e) => Result::Err(e),
    }
}

pub fn parse_not(input: &mut &str) -> winnow::Result<Expr> {
    match (literal("!")).parse_next(input) {
        Result::Ok(_) => match parse_var(input) {
            Ok(e) => Ok(Expr::Not { x: Box::new(e) }),
            Err(_) => todo!(),
        },
        Result::Err(e) => Result::Err(e),
    }
}

pub fn eval(assignment: &HashMap<&str, bool>, expr: &Expr) -> bool {
    match expr {
        Expr::Var { name: x } => assignment[x.as_str()],
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
        assert_eq!(
            Expr::Var {
                name: "a".to_string()
            },
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_not_works() {
        let mut input = "!a";
        let expr = parse_not(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::Not {
                x: Box::new(Expr::Var {
                    name: "a".to_string()
                })
            },
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn assignment_works() {
        let a = Expr::Var {
            name: "a".to_string(),
        };
        let b = Expr::Var {
            name: "b".to_string(),
        };
        let mut assignment = HashMap::new();
        assignment.insert("a", false);
        assignment.insert("b", true);
        assert!(!eval(&assignment, &a));
        assert!(eval(&assignment, &b))
    }

    #[test]
    fn not_works() {
        let assignment = HashMap::new();
        let not_true = Expr::Not {
            x: Box::new(Expr::True),
        };
        let not_false = Expr::Not {
            x: Box::new(Expr::False),
        };
        assert!(!eval(&assignment, &not_true));
        assert!(eval(&assignment, &not_false))
    }

    #[test]
    fn or_works() {
        let assignment = HashMap::new();

        let expr = Expr::Or {
            x: Box::new(Expr::False),
            y: Box::new(Expr::False),
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::Or {
            x: Box::new(Expr::False),
            y: Box::new(Expr::True),
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: Box::new(Expr::True),
            y: Box::new(Expr::False),
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: Box::new(Expr::True),
            y: Box::new(Expr::True),
        };
        assert!(eval(&assignment, &expr));
    }

    #[test]
    fn and_works() {
        let assignment = HashMap::new();

        let expr = Expr::And {
            x: Box::new(Expr::False),
            y: Box::new(Expr::False),
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: Box::new(Expr::False),
            y: Box::new(Expr::True),
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: Box::new(Expr::True),
            y: Box::new(Expr::False),
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: Box::new(Expr::True),
            y: Box::new(Expr::True),
        };
        assert!(eval(&assignment, &expr));
    }
}
