use std::collections::HashSet;
use std::fmt::Display;
use std::panic::panic_any;

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

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum SetElement {
    Elem(String),
    Set(HashSet<SetElement>),
}

impl From<&str> for SetElement {
    fn from(value: &str) -> Self {
        let value = value.replace(" ", "");
        if value.trim() == "" {
            panic!("empty string given to SetElement::From")
        }

        if value == "{}" {
            return SetElement::Set(HashSet::new());
        }

        // element is {} or {...} or {{..}} or {{}, a}
        if value.starts_with('{') && value.ends_with('}') {
            let inner = value[1..value.len() - 1].trim();

            let inbalanced = {
                let open = inner.find('{');
                let close = inner.find('}');
                if let (Some(o), Some(c)) = (open, close) {
                    o > c
                } else {
                    false
                }
            };

            if inbalanced {
                let elems: Vec<&str> = value.split_terminator(&[','][..]).collect();
                let hash_set: HashSet<SetElement> =
                    elems.into_iter().map(|e| e.trim().into()).collect();
                SetElement::Set(hash_set)
            } else if inner.starts_with('{') && inner.ends_with('}') {
                let element: SetElement = inner.into();
                let mut set: HashSet<SetElement> = HashSet::new();
                set.insert(element);
                SetElement::Set(set)
            } else {
                let elems: Vec<&str> = inner.split_terminator(&[','][..]).collect();
                let hash_set: HashSet<SetElement> =
                    elems.into_iter().map(|e| e.trim().into()).collect();
                SetElement::Set(hash_set)
            }
        } else {
            if value.contains('{') || value.contains('}') {
                let s = format!("inner set missed by parser '{}'", value);
                panic_any(s)
            }
            SetElement::Elem(value.to_string())
        }
    }
}

impl std::hash::Hash for SetElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Display for SetElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetElement::Elem(e) => write!(f, "{}", e),
            SetElement::Set(hash_set) => {
                let elems: Vec<SetElement> = hash_set.iter().cloned().collect();
                let items: Vec<String> = elems.iter().map(|e| e.to_string()).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub enum Expr {
    Var(String),
    SetLiteral(HashSet<SetElement>),
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
                let mut items: Vec<String> = items.iter().map(|e| e.to_string()).collect();
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
                                        let set: Vec<String> = s.iter().map(|x|x.to_string()).collect();
                                        let elems: HashSet<SetElement> = set.iter().map(|e|SetElement::from(e.as_str())).collect();
                                        Expr::SetLiteral(elems)
                                    }),
                                    cut_err('}')),
                                empty_set.map(|_|Expr::SetLiteral([].into())),
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

// {a,b,c}
fn non_empty_set<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    delimited(
        '{',
        comma_list.map(|s| {
            let set: Vec<String> = s.iter().map(|x| x.to_string()).collect();
            let elems: HashSet<SetElement> =
                set.iter().map(|e| SetElement::from(e.as_str())).collect();
            Expr::SetLiteral(elems)
        }),
        cut_err('}'),
    )
    .take()
    .parse_next(i)
}

// {}
fn empty_set<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    delimited(
        '{',
        multispace0.map(|_| Expr::SetLiteral(HashSet::new())),
        cut_err('}'),
    )
    .take()
    .parse_next(i)
}

// abc123
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

// abc or {}
fn set_element<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    fn char_or_num_element<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
        let element = (
            one_of(|c: char| c.is_alphanum()),
            take_while(0.., |c: char| c.is_alphanum() || c == '_'),
        );
        trace("set_element", element).take().parse_next(i)
    }

    let element = alt((char_or_num_element, non_empty_set, empty_set));
    trace("set_element", element).take().parse_next(i)
}

fn comma_list<'i>(i: &mut &'i str) -> ModalResult<Vec<&'i str>> {
    let ident_with_space = delimited(multispace0, set_element, multispace0);
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
                    let inter: HashSet<_> = set1.intersection(&set2).cloned().collect();
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
                    let union: HashSet<_> = set1.union(&set2).cloned().collect();
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
        let mut input = "{a,2,{}}";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::SetLiteral(["a".into(), "2".into(), "{}".into()].into()),
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

    #[test]
    #[should_panic]
    fn set_literal_from_empty_string_should_panic() {
        let _ = Expr::SetLiteral(["".into()].into());
    }

    #[test]
    fn set_literal_from_empty_works() {
        let nested = pratt_parser(&mut "{{{ {}, b }}}");
        assert!(nested.is_ok());
        match nested.unwrap() {
            Expr::SetLiteral(inner1) => {
                let inner1: Vec<&SetElement> = inner1.iter().collect();
                if let SetElement::Set(inner2) = inner1[0] {
                    let inner2: Vec<&SetElement> = inner2.iter().collect();
                    if let SetElement::Set(inner3) = inner2[0] {
                        assert_eq!(inner3.len(), 2)
                    } else if let SetElement::Elem(v) = inner2[0] {
                        assert_eq!(v, "")
                    } else {
                        panic!("expected something else for inner2")
                    }
                } else {
                    panic!("expected something else for inner1")
                }
            }
            _ => panic!("expected something else"),
        }
    }
}
