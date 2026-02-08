use std::collections::HashMap;

use winnow::ModalResult;
use winnow::Parser;
use winnow::ascii::multispace0;
use winnow::combinator::alt;
use winnow::combinator::cut_err;
use winnow::combinator::delimited;
use winnow::combinator::dispatch;
use winnow::combinator::expression;
use winnow::combinator::fail;
use winnow::combinator::peek;
use winnow::combinator::trace;
use winnow::combinator::{Infix, Prefix};
use winnow::error::ContextError;
use winnow::error::ErrMode;
use winnow::stream::AsChar;
use winnow::token::any;
use winnow::token::one_of;
use winnow::token::take_while;

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Var { name: String },
    Not { x: Box<Expr> },
    Or { x: Box<Expr>, y: Box<Expr> },
    And { x: Box<Expr>, y: Box<Expr> },
    Paren { x: Box<Expr> },
    True,
    False,
}

pub fn pratt_parser(i: &mut &str) -> ModalResult<Expr> {
    fn parser<'i>(precedence: i64) -> impl Parser<&'i str, Expr, ErrMode<ContextError>> {
        move |i: &mut &str| {
            use Infix::Left;
            expression(
                delimited(
                    multispace0,
                    dispatch! {peek(any);
                        '(' => delimited('(',  parser(0).map(|e| Expr::Paren{x: Box::new(e)}), cut_err(')')),
                        _ => alt((
                            identifier.map(|s| Expr::Var{name: s.into()}),
                        )),
                    },
                    multispace0,
                )
            )
            .current_precedence_level(precedence)
            .prefix(
                delimited(
                    multispace0,
                    dispatch! {any;
                        '!' => Prefix(18, |_: &mut _, a| Ok(Expr::Not{x: Box::new(a)})),
                        _ => fail
                    },
                    multispace0,
                )
            )
            .infix(
                alt((
                    dispatch! {any;
                        '&' => Left(8, |_: &mut _, a, b| Ok(Expr::And{x: Box::new(a), y:Box::new(b)})),
                        '|' => Left(7, |_: &mut _, a, b| Ok(Expr::Or{x: Box::new(a), y:Box::new(b)})),
                        _ => fail
                    },
                )),
            )
            .parse_next(i)
        }
    }

    parser(0).parse_next(i)
}

fn identifier<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    trace(
        "identifier",
        (
            one_of(|c: char| c.is_alpha() || c == '_'),
            take_while(0.., |c: char| c.is_alphanum() || c == '_'),
        ),
    )
    .take()
    .parse_next(i)
}

pub fn eval(assignment: &HashMap<&str, bool>, expr: &Expr) -> bool {
    match expr {
        Expr::Var { name: x } => assignment[x.as_str()],
        Expr::Not { x } => !eval(assignment, x),
        Expr::Or { x, y } => eval(assignment, x) || eval(assignment, y),
        Expr::And { x, y } => eval(assignment, x) && eval(assignment, y),
        Expr::True => true,
        Expr::False => false,
        Expr::Paren { x } => eval(assignment, x),
    }
}

pub fn run(formula: &str, assignment: &HashMap<&str, bool>) -> Result<bool, String> {
    let input = formula.to_string().clone();
    match pratt_parser(&mut input.as_str()) {
        Ok(expr) => Ok(eval(assignment, &expr)),
        Err(e) => Result::Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_a_var_works() {
        let mut input = "a";
        let expr = pratt_parser(&mut input);
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
    fn parsing_a_var_in_parens_works() {
        let mut input = "(a)";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::Paren {
                x: Box::new(Expr::Var {
                    name: "a".to_string()
                })
            },
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_not_works() {
        let mut input = "!a";
        let expr = pratt_parser(&mut input);
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
    fn parsing_an_or_works() {
        let mut input = "a | b";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::Or {
                x: Box::new(Expr::Var {
                    name: "a".to_string()
                }),
                y: Box::new(Expr::Var {
                    name: "b".to_string()
                })
            },
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_an_and_works() {
        let mut input = "a & b";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::And {
                x: Box::new(Expr::Var {
                    name: "a".to_string()
                }),
                y: Box::new(Expr::Var {
                    name: "b".to_string()
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
    #[test]
    fn run_works() {
        let mut assignment = HashMap::new();
        assignment.insert("a", true);
        assignment.insert("b", true);
        let r = run("a & b", &assignment);
        assert!(r.is_ok());
        assert!(r.unwrap())
    }
}
