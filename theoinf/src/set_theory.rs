use std::collections::HashMap;
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

const UNI_IDENT: &str = "UNI";

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
            .expect("Can't construct SetElement from given value.")
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
            SetElement::Elem(e) => write!(f, "{e}"),
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
    Complement(Box<Expr>),
    Intersection(Box<Expr>, Box<Expr>),
    Union(Box<Expr>, Box<Expr>),
    Difference(Box<Expr>, Box<Expr>),
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
            Expr::SetDecl(ident, expr) => write!(f, "{ident} = {expr}"),
            Expr::Complement(expr) => write!(f, "!{expr}"),
            Expr::Intersection(expr1, expr2) => write!(f, "{expr1} n {expr2}"),
            Expr::Union(expr1, expr2) => write!(f, "{expr1} u {expr2}"),
            Expr::Difference(expr1, expr2) => write!(f, "{expr1} \\ {expr2}"),
            Expr::Paren(expr) => write!(f, "({expr})"),
            Expr::Element(set_element) => write!(f, "{set_element}"),
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
                            _ => identifier.map(|s| Expr::Var(s.to_string())),
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
                        '!' => Prefix(100, |_: &mut _, a| Ok(Expr::Complement(Box::new(a)))),
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
                        '\\' => Left(2, |_: &mut _, a, b| Ok(Expr::Difference(Box::new(a), Box::new(b)))),
                        '=' => Left(1, |_: &mut _, a, b| {
                            match(a, &b) {
                                (Expr::Var(i), Expr::SetLiteral(_)) => Ok(Expr::SetDecl(i, Box::new(b))),
                                _ => Err(ErrMode::Cut(ContextError::default()))
                            }
                        }),
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

fn identifier<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
    let identifier = (
        one_of(|c: char| c.is_alpha()),
        take_while(0.., |c: char| c.is_alphanum() || c == '_'),
    );
    trace("identifier", identifier).take().parse_next(i)
}

type Assignment = HashMap<String, Expr>;

pub fn eval(assignment: &mut Assignment, expr: &Expr) -> Result<Expr, String> {
    match expr {
        Expr::Element(_) => Ok(expr.clone()),
        Expr::Var(ident) => {
            let expr = assignment
                .get(ident)
                .cloned()
                .ok_or(format!("Identifier {ident} not found."))?;
            eval(assignment, &expr)
        }
        Expr::Paren(expr) => eval(assignment, expr),
        Expr::SetLiteral(items) => {
            if let Some(uni) = assignment.get(UNI_IDENT).cloned() {
                match uni {
                    Expr::SetLiteral(hash_set) => {
                        for item in items {
                            if !hash_set.contains(item) {
                                return Err(format!(
                                    "Element '{item}' not in declared universe set '{UNI_IDENT}'."
                                ));
                            }
                        }
                    }
                    _ => return Err("UNI must be a set literal.".to_string()),
                }
            }

            Ok(expr.clone())
        }
        Expr::SetDecl(ident, set_expr) => {
            if ident == UNI_IDENT && !assignment.is_empty() {
                Err(format!(
                    "The universe set '{UNI_IDENT}' must be the first declaration."
                ))
            } else {
                assignment.insert(ident.to_string(), *set_expr.clone());
                Ok(expr.clone())
            }
        }
        Expr::Complement(expr) => {
            let uni = assignment.get(UNI_IDENT).cloned().ok_or(format!(
                "For complement operations the universe '{UNI_IDENT}' must be declared as the first set."
            ))?;
            let diff = Expr::Difference(Box::new(uni), expr.clone());
            eval(assignment, &diff)
        }
        Expr::Intersection(expr1, expr2) => {
            let expr1 = eval(assignment, expr1)?;
            let expr2 = eval(assignment, expr2)?;
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let inter: HashSet<_> = set1.intersection(&set2).cloned().collect();
                    Ok(Expr::SetLiteral(inter))
                }
                _ => todo!(),
            }
        }
        Expr::Union(expr1, expr2) => {
            let expr1 = eval(assignment, expr1)?;
            let expr2 = eval(assignment, expr2)?;
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let union: HashSet<_> = set1.union(&set2).cloned().collect();
                    Ok(Expr::SetLiteral(union))
                }
                _ => todo!(),
            }
        }
        Expr::Difference(expr1, expr2) => {
            let expr1 = eval(assignment, expr1)?;
            let expr2 = eval(assignment, expr2)?;
            match (expr1, expr2) {
                (Expr::SetLiteral(set1), Expr::SetLiteral(set2)) => {
                    let union: HashSet<_> = set1.difference(&set2).cloned().collect();
                    Ok(Expr::SetLiteral(union))
                }
                _ => todo!(),
            }
        }
    }
}

pub fn run(formula: &str) -> Result<Expr, String> {
    let lines = formula.trim().lines();
    let mut a = HashMap::new();
    let results: Vec<_> = lines
        .into_iter()
        .map(|mut line| match pratt_parser(&mut line) {
            Ok(expr) => eval(&mut a, &expr),
            Err(e) => Result::Err(e.to_string()),
        })
        .collect();
    let (oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(|r| (r).is_ok());
    if errs.is_empty() {
        oks.into_iter().last().unwrap()
    } else {
        errs.into_iter().next().unwrap()
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

    #[test]
    fn parsing_a_declaration_works() {
        let s = pratt_parser(&mut "A = { a, b }");
        assert!(s.is_ok());
        let s = s.unwrap();
        let lit = Expr::SetLiteral(["a".into(), "b".into()].into());
        let decl = Expr::SetDecl("A".into(), Box::new(lit));
        assert_eq!(s, decl)
    }

    #[test]
    fn parsing_a_complement_works() {
        let ast = pratt_parser(&mut "!A");
        assert!(ast.is_ok());
        let s = ast.unwrap();
        let comp = Expr::Complement(Box::new(Expr::Var("A".into())));
        assert_eq!(comp, s)
    }

    #[test]
    fn evaluating_an_ident_works() {
        let lit1 = Expr::SetLiteral(["a".into(), "b".into()].into());
        let lit2 = Expr::SetLiteral(["c".into(), "d".into()].into());
        let mut assignment: Assignment = HashMap::new();
        assignment.insert("A".into(), lit1.clone());
        assignment.insert("B".into(), lit2.clone());
        let expr = pratt_parser(&mut "A");
        assert!(expr.is_ok());
        let expr = expr.unwrap();
        let r = eval(&mut assignment, &expr);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), lit1)
    }

    #[test]
    fn run_with_assignments_works() {
        let r = run("A = {a,b}\nB = {b,c}\nA n B");
        assert!(r.is_ok());
        assert_eq!(Expr::SetLiteral(["b".into()].into()), r.unwrap());
    }

    #[test]
    fn run_returns_first_parsing_error() {
        let r = run("A = {a,b}\nB = #b,c}\nA n B");
        assert!(r.is_err());
    }

    #[test]
    fn run_return_first_eval_error() {
        let r = run("A = {a,b}\nB\nA");
        assert!(r.is_err());
        assert!(r.err().unwrap().contains("B not found"));
    }

    #[test]
    fn evaluation_of_complement_works() {
        let r = run("UNI = {a,b,c}\nA = {a}\n!A");
        assert!(r.is_ok());
        assert_eq!(
            Expr::SetLiteral(["b".into(), "c".into()].into()),
            r.unwrap()
        );
    }

    #[test]
    fn enforcing_uni_works() {
        let r = run("UNI = {a,b,c}\nA = {22}\nA");
        assert!(r.is_err());
    }

    #[test]
    fn uni_must_be_the_first_declaration() {
        let r = run("A = {a}\nUNI = {a,b,c}\nA");
        assert!(r.is_err());
    }
}
