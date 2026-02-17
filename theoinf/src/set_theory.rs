use std::collections::HashSet;
use std::fmt::Display;

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
use winnow::combinator::separated;
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
    Var(String),
    SetLiteral(HashSet<String>),
    SetDecl(String, Box<Expr>),
    Not(Box<Expr>),
    Intersection(Box<Expr>, Box<Expr>),
    Union(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(v) => write!(f, "{v}"),
            Expr::SetLiteral(items) => {
                let mut items: Vec<String> = items.iter().cloned().collect();
                items.sort();
                write!(f, "{{{}}}", items.join(", "))
            }
            Expr::SetDecl(_, _expr) => todo!(),
            Expr::Not(_expr) => todo!(),
            Expr::Intersection(expr1, expr2) => write!(f, "{} n {}", expr1, expr2),
            Expr::Union(expr1, expr2) => write!(f, "{} u {}", expr1, expr2),
            Expr::Paren(expr) => write!(f, "({})", expr),
        }
    }
}

pub fn pratt_parser(i: &mut &str) -> ModalResult<Expr> {
    fn parser<'i>(precedence: i64) -> impl Parser<&'i str, Expr, ErrMode<ContextError>> {
        move |i: &mut &str| -> Result<Expr, ErrMode<ContextError>> {
            use Infix::Left;
            expression(
                delimited(
                    multispace0,
                    alt ((
                        dispatch! {peek(any);
                            '(' => delimited('(',  parser(0).map(|e| Expr::Paren(Box::new(e))), cut_err(')')),
                            '{' => alt((
                                 delimited(
                                    '{',
                                    comma_list.map(|s|{
                                        let set: HashSet<String> = s.iter().map(|x|x.to_string()).collect();
                                        Expr::SetLiteral(set)
                                    }),
                                    cut_err('}')),
                                delimited(
                                    '{',
                                    multispace0.map(|_| Expr::SetLiteral(HashSet::new())),
                                    cut_err('}')),
                            )),
                            _ => identifier.map(|s| Expr::Var(s.into())),
                        },
                    )),
                    multispace0,
                )
            )
            .current_precedence_level(precedence)
            .prefix(
                delimited(
                    multispace0,
                    dispatch! {any;
                        '!' => Prefix(100, |_: &mut _, a| Ok(Expr::Not(Box::new(a)))),
                        _ => fail
                    },
                    multispace0,
                )
            )
            .infix(
                alt((
                    dispatch! {any;
                        'u' => Left(3, |_: &mut _, a, b| Ok(Expr::Union(Box::new(a), Box::new(b)))),
                        'n' => Left(4, |_: &mut _, a, b| Ok(Expr::Intersection(Box::new(a), Box::new(b)))),
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

fn comma_list<'i>(i: &mut &'i str) -> ModalResult<Vec<&'i str>> {
    let ident_with_space = delimited(multispace0, identifier, multispace0);
    separated(1.., ident_with_space, ",").parse_next(i)
}

pub fn eval(expr: &Expr) -> Expr {
    match expr {
        Expr::Var(_) => expr.clone(),
        Expr::Paren(a) => eval(a),
        Expr::SetLiteral(_items) => expr.clone(),
        Expr::SetDecl(_, _expr) => todo!(),
        Expr::Not(_expr) => todo!(),
        Expr::Intersection(expr1, expr2) => {
            let expr1 = eval(expr1);
            let expr2 = eval(expr2);
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let inter: HashSet<String> = set1.intersection(&set2).cloned().collect();
                    Expr::SetLiteral(inter)
                }
                _ => todo!(),
            }
        }
        Expr::Union(expr1, expr2) => {
            let expr1 = eval(expr1);
            let expr2 = eval(expr2);
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let union: HashSet<String> = set1.union(&set2).cloned().collect();
                    Expr::SetLiteral(union)
                }
                _ => todo!(),
            }
        }
    }
}

pub fn run(formula: &str) -> Result<Expr, String> {
    let input = formula.to_string();
    match pratt_parser(&mut input.as_str()) {
        Ok(expr) => Ok(eval(&expr)),
        Err(e) => Result::Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_an_identifier_works() {
        let mut input = "a";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::Var("a".into()), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_an_empty_set_literal_works() {
        let mut input = "{}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::SetLiteral([].into()), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_singleton_set_literal_works() {
        let mut input = "{a}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::SetLiteral(["a".into()].into()), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_two_element_set_literal_works() {
        let mut input = "{a,b}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "b".into()].into()),
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_three_element_set_literal_works() {
        let mut input = "{a,b,c}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "b".into(), "c".into()].into()),
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_union_works() {
        let mut input = "{ a , b,c} u {d,e}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        let s0 = Expr::SetLiteral(["a".into(), "b".into(), "c".into()].into());
        let s1 = Expr::SetLiteral(["d".into(), "e".into()].into());
        assert_eq!(Expr::Union(Box::new(s0), Box::new(s1)), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_an_intersection_works() {
        let mut input = "{ a } n {}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        let s0 = Expr::SetLiteral(["a".into()].into());
        let s1 = Expr::SetLiteral([].into());
        assert_eq!(
            Expr::Intersection(Box::new(s0), Box::new(s1)),
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_comma_list_works() {
        let mut input = "a,b, c";
        let expr = comma_list(&mut input);
        assert!(expr.is_ok());
        assert_eq!(vec!["a", "b", "c"], expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn eval_of_an_intersection_works() {
        let r = run("{ a,c } n {a,b,c}");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "c".into()].into()),
            r.unwrap()
        );
    }

    #[test]
    fn eval_of_an_intersection_with_an_empty_set_works() {
        let r = run("{ a,c } n {}");
        assert!(r.is_ok());
        assert_eq!(Expr::SetLiteral([].into()), r.unwrap());
    }

    #[test]
    fn eval_of_an_union_works() {
        let r = run("{ a,c } u {b,c}");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "b".into(), "c".into()].into()),
            r.unwrap()
        );
    }

    #[test]
    fn eval_of_an_union_with_an_empty_set_works() {
        let r = run("({   } u ({b,c}))");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["b".into(), "c".into()].into()),
            r.unwrap()
        );
    }

    #[test]
    fn precedence_works() {
        let r = run("{a} u {b} n {c}");
        assert!(r.is_ok());
        assert_eq!(Expr::SetLiteral(["a".into()].into()), r.unwrap());
    }

    #[test]
    fn parentheses_works() {
        let r = run("({a} u {b}) n {c}");
        assert!(r.is_ok());
        assert_eq!(Expr::SetLiteral([].into()), r.unwrap());
    }

    #[test]
    fn dedup_works() {
        let r = run("{a,b,a,b}");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "b".into()].into()),
            r.unwrap()
        );
    }
}
