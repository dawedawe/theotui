use std::collections::HashSet;
use std::ops::Deref;

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
    SetLiteral(Vec<String>),
    SetDecl(String, Box<Expr>),
    Not(Box<Expr>),
    Intersection(Box<Expr>, Box<Expr>),
    Union(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

pub fn pratt_parser(i: &mut &str) -> ModalResult<Expr> {
    fn parser<'i>(precedence: i64) -> impl Parser<&'i str, Expr, ErrMode<ContextError>> {
        move |i: &mut &str| {
            use Infix::Left;
            expression(
                delimited(
                    multispace0,
                    alt ((
                        dispatch! {peek(any);
                            '(' => delimited('(',  parser(0).map(|e| Expr::Paren(Box::new(e))), cut_err(')')),
                            '{' => delimited('{',  comma_list.map(|s|{
                                    let v = s.iter().map(|x|x.to_string()).collect();
                                    Expr::SetLiteral(v)
                            })   , cut_err('}')),
                            _ => identifier.map(|s| Expr::Var(s.into())),
                        },
                        // "{}".value(Expr::SetLiteral(vec![])),
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
                        'u' => Left(4, |_: &mut _, a, b| Ok(Expr::Union(Box::new(a), Box::new(b)))),
                        'n' => Left(3, |_: &mut _, a, b| Ok(Expr::Intersection(Box::new(a), Box::new(b)))),
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
    separated(0.., ident_with_space, ",").parse_next(i)
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
                (Expr::SetLiteral(items1), Expr::SetLiteral(items2)) => {
                    let set1: HashSet<String> = items1.deref().iter().cloned().collect();
                    let set2: HashSet<String> = items2.deref().iter().cloned().collect();
                    let mut inter: Vec<String> = set1.intersection(&set2).cloned().collect();
                    inter.sort();
                    Expr::SetLiteral(inter)
                }
                _ => todo!(),
            }
        }
        Expr::Union(expr1, expr2) => {
            let expr1 = eval(expr1);
            let expr2 = eval(expr2);
            match (expr1, expr2) {
                (Expr::SetLiteral(items1), Expr::SetLiteral(items2)) => {
                    let set1: HashSet<String> = items1.deref().iter().cloned().collect();
                    let set2: HashSet<String> = items2.deref().iter().cloned().collect();
                    let mut union: Vec<String> = set1.union(&set2).cloned().collect();
                    union.sort();
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
        assert_eq!(Expr::SetLiteral(vec![]), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_singleton_set_literal_works() {
        let mut input = "{a}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::SetLiteral(vec!["a".into()]), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_two_element_set_literal_works() {
        let mut input = "{a,b}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::SetLiteral(vec!["a".into(), "b".into()]),
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
            Expr::SetLiteral(vec!["a".into(), "b".into(), "c".into()]),
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_union_works() {
        let mut input = "{ a , b,c} u {d,e}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        let s0 = Expr::SetLiteral(vec!["a".into(), "b".into(), "c".into()]);
        let s1 = Expr::SetLiteral(vec!["d".into(), "e".into()]);
        assert_eq!(Expr::Union(Box::new(s0), Box::new(s1)), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_an_intersection_works() {
        let mut input = "{ a } n {}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        let s0 = Expr::SetLiteral(vec!["a".into()]);
        let s1 = Expr::SetLiteral(vec![]);
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

        assert_eq!(Expr::SetLiteral(vec!["a".into(), "c".into()]), r.unwrap());
    }

    #[test]
    fn eval_of_an_intersection_with_an_empty_set_works() {
        let r = run("{ a,c } n {}");
        assert!(r.is_ok());

        assert_eq!(Expr::SetLiteral(vec![]), r.unwrap());
    }

    #[test]
    fn eval_of_an_union_works() {
        let r = run("{ a,c } u {b,c}");
        assert!(r.is_ok());

        assert_eq!(
            Expr::SetLiteral(vec!["a".into(), "b".into(), "c".into()]),
            r.unwrap()
        );
    }

    #[test]
    fn eval_of_an_union_with_an_empty_set_works() {
        let r = run("({} u ({b,c}))");
        assert!(r.is_ok());

        assert_eq!(Expr::SetLiteral(vec!["b".into(), "c".into()]), r.unwrap());
    }
}
