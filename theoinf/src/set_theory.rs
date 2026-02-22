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
use winnow::combinator::opt;
use winnow::combinator::peek;
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
    Comma(Box<(SetElement, SetElement)>),
}

impl From<&str> for SetElement {
    fn from(value: &str) -> Self {
        element_parser(0)
            .parse(value)
            .expect("can't construct SetElement from given value")
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
            SetElement::Comma(ab) => write!(f, "{}, {}", ab.0, ab.1),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub enum Expr {
    Var(String),
    Element(String),
    SetLiteral(HashSet<SetElement>),
    SetDecl(String, Box<Expr>),
    Not(Box<Expr>),
    Intersection(Box<Expr>, Box<Expr>),
    Union(Box<Expr>, Box<Expr>),
    Diff(Box<Expr>, Box<Expr>),
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
            Expr::Diff(expr1, expr2) => write!(f, "{} \\ {}", expr1, expr2),
            Expr::Paren(expr) => write!(f, "({})", expr),
            Expr::Element(set_element) => write!(f, "{}", set_element),
        }
    }
}

fn traverse_comma((a, b): &(SetElement, SetElement)) -> HashSet<SetElement> {
    match (a, b) {
        (SetElement::Elem(_), SetElement::Elem(_))
        | (SetElement::Elem(_), SetElement::Set(_))
        | (SetElement::Set(_), SetElement::Elem(_))
        | (SetElement::Set(_), SetElement::Set(_)) => {
            let mut hash_set = HashSet::new();
            hash_set.insert(a.clone());
            hash_set.insert(b.clone());
            hash_set
        }
        (SetElement::Comma(ab), b) => {
            let mut hash_set = HashSet::new();
            let left = traverse_comma(ab);
            left.into_iter().for_each(|e| {
                hash_set.insert(e);
            });
            hash_set.insert(b.clone());
            hash_set
        }
        _ => todo!(),
    }
}

fn element_parser<'i>(precedence: i64) -> impl Parser<&'i str, SetElement, ErrMode<ContextError>> {
    move |i: &mut &str| -> Result<SetElement, ErrMode<ContextError>> {
        use Infix::Left;
        expression(delimited(
            multispace0,
            alt((dispatch! {peek(any);
                '{' => delimited('{',  opt(element_parser(0)).map(|e|
                    {
                        let mut hash_set: HashSet<SetElement> = HashSet::new();
                        if let Some(e) = e {
                                match &e {
                                    SetElement::Elem(x) => {
                                        let e = SetElement::Elem(x.to_string());
                                        hash_set.insert(e);
                                    },
                                    SetElement::Set(s) => {
                                        let e = SetElement::Set(s.clone());
                                        hash_set.insert(e);
                                    },
                                    SetElement::Comma(ab) => {
                                        let se = traverse_comma(ab);
                                        se.into_iter().for_each(|e|{hash_set.insert(e);});
                                    },
                                };
                        }
                        SetElement::Set(hash_set)
                    }
                    ), cut_err('}')),
                _ => char_or_num_element.map(|s| {
                        SetElement::Elem(s.to_string())
                    }),
            },)),
            multispace0,
        ))
        .current_precedence_level(precedence)
        .infix(alt((dispatch! {any;
            ',' => Left(0, |_: &mut _, a, b| {
                Ok(SetElement::Comma(Box::new((a, b))))
            }),
            _ => fail
        },)))
        .parse_next(i)
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
                            '{' => delimited('{',  opt(element_parser(0)).map(|e|
                                {
                                    let mut hash_set = HashSet::new();
                                    if let Some(elem) = e {
                                        match elem {
                                            SetElement::Elem(_) => { hash_set.insert(elem); },
                                            SetElement::Set(set) => {
                                                let nested_set = SetElement::Set(set.clone());
                                                hash_set.insert(nested_set);
                                            },
                                            SetElement::Comma(ab) => {
                                                let set = traverse_comma(&ab);
                                                set.into_iter().for_each(|e| {hash_set.insert(e);})
                                            },
                                        }
                                    }
                                    Expr::SetLiteral(hash_set)
                                }
                                ), cut_err('}')),
                            _ => char_or_num_element.map(|s| Expr::Element(s.to_string())),
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
                        '\\' => Left(4, |_: &mut _, a, b| Ok(Expr::Diff(Box::new(a), Box::new(b)))),
                        _ => fail
                    },
                )),
            )
            .parse_next(i)
        }
    }

    parser(0).parse_next(i)
}

fn char_or_num_element<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    let element = (
        one_of(|c: char| c.is_alphanum()),
        take_while(0.., |c: char| c.is_alphanum() || c == '_'),
    );
    trace("char_or_num_element", element).take().parse_next(i)
}

pub fn eval(expr: &Expr) -> Expr {
    match expr {
        Expr::Element(_) => expr.clone(),
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
        Expr::Diff(expr1, expr2) => {
            let expr1 = eval(expr1);
            let expr2 = eval(expr2);
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let union: HashSet<_> = set1.difference(&set2).cloned().collect();
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
        assert_eq!(Expr::Element("a".into()), expr.unwrap());
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
        let r = run("({} u ({b,c}))");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["b".into(), "c".into()].into()),
            r.unwrap()
        );
    }

    #[test]
    fn eval_of_a_diff_works() {
        let r = run("{ a,c } \\ {b,c}");
        assert!(r.is_ok());
        assert_eq!(Expr::SetLiteral(["a".into()].into()), r.unwrap());
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

    #[test]
    fn parsing_nested_empty_works() {
        let s = pratt_parser(&mut "{ {} }");
        assert!(s.is_ok());
        let s = s.unwrap();
        let mut hash_set = HashSet::new();
        hash_set.insert(SetElement::Set(HashSet::new()));
        let nested_empty = Expr::SetLiteral(hash_set);
        assert_eq!(s, nested_empty)
    }

    #[test]
    fn parsing_double_nested_empty_works() {
        let s = pratt_parser(&mut "{ { {} } }");
        assert!(s.is_ok());
        let s = s.unwrap();
        let mut hash_set1 = HashSet::new();
        let mut hash_set2 = HashSet::new();
        hash_set1.insert(SetElement::Set(HashSet::new()));
        hash_set2.insert(SetElement::Set(hash_set1));
        let nested_empty = Expr::SetLiteral(hash_set2);
        assert_eq!(s, nested_empty)
    }
}
