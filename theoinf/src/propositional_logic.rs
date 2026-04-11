use std::{collections::HashMap, fmt::Display};

use crate::propositional_logic::parser::pratt_parser;
use winnow::ModalResult;

/// Expressions of the propositional logic language.
#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    True,
    False,
    Var(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Equi(Box<Expr>, Box<Expr>),
    Impl(Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::True => write!(f, "true"),
            Expr::False => write!(f, "false"),
            Expr::Var(v) => write!(f, "{}", v),
            Expr::Not(expr) => write!(f, "!{}", expr),
            Expr::And(expr1, expr2) => write!(f, "{} & {}", expr1, expr2),
            Expr::Or(expr1, expr2) => write!(f, "{} | {}", expr1, expr2),
            Expr::Xor(expr1, expr2) => write!(f, "{} ^ {}", expr1, expr2),
            Expr::Equi(expr1, expr2) => write!(f, "{} <=> {}", expr1, expr2),
            Expr::Impl(expr1, expr2) => write!(f, "{} -> {}", expr1, expr2),
            Expr::Paren(expr) => write!(f, "({})", expr),
        }
    }
}

impl Expr {
    pub fn collect_vars(&self) -> Vec<String> {
        let mut vars = vec![];
        fn helper(expr: &Expr, vars: &mut Vec<String>) {
            match expr {
                Expr::Var(a) => vars.push(a.into()),
                Expr::Not(expr) => helper(expr, vars),
                Expr::And(a, b)
                | Expr::Or(a, b)
                | Expr::Xor(a, b)
                | Expr::Equi(a, b)
                | Expr::Impl(a, b) => {
                    let a_vars = a.collect_vars();
                    a_vars.into_iter().for_each(|v| vars.push(v));
                    let b_vars = b.collect_vars();
                    b_vars.into_iter().for_each(|v| vars.push(v));
                }
                Expr::Paren(a) => {
                    let a_vars = a.collect_vars();
                    a_vars.into_iter().for_each(|v| vars.push(v));
                }
                Expr::True => (),
                Expr::False => (),
            }
        }
        helper(self, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    fn reduce<F>(exprs: &[Expr], f: &F) -> Expr
    where
        F: Fn(Expr, Expr) -> Expr,
    {
        if exprs.is_empty() {
            panic!("can't create disjunction out of empty expressions");
        } else if exprs.len() == 1 {
            exprs[0].clone()
        } else {
            let left = exprs.iter().next().unwrap().clone();
            let right = Expr::reduce(&exprs[1..], f);
            f(left, right)
        }
    }
}

pub mod parser {
    use winnow::ModalResult;
    use winnow::Parser;
    use winnow::ascii::multispace0;
    use winnow::combinator::alt;
    use winnow::combinator::cut_err;
    use winnow::combinator::delimited;
    use winnow::combinator::dispatch;
    use winnow::combinator::eof;
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
    use winnow::token::take;
    use winnow::token::take_while;

    use crate::propositional_logic::Expr;

    /// Parse the input to an [Expr].
    pub fn pratt_parser(input: &mut &str) -> ModalResult<Expr> {
        fn parser<'i>(precedence: i64) -> impl Parser<&'i str, Expr, ErrMode<ContextError>> {
            move |input: &mut &str| {
                use Infix::Left;
                expression(
                delimited(
                    multispace0,
                    dispatch! {peek(any);
                        '(' => delimited('(',  parser(0).map(|e| Expr::Paren(Box::new(e))), cut_err(')')),
                        _ => alt((
                            false_lit.map(|_| {Expr::False}),
                            true_lit.map(|_| {Expr::True}),
                            identifier.map(|s| Expr::Var( s.into())),
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
                        '!' => Prefix(100, |_: &mut _, a| Ok(Expr::Not(Box::new(a)))),
                        _ => fail
                    },
                    multispace0,
                )
            )
            .infix(
                alt((
                    dispatch! {any;
                        '&' => Left(80, |_: &mut _, a, b| Ok(Expr::And(Box::new(a), Box::new(b)))),
                        '^' => Left(70, |_: &mut _, a, b| Ok(Expr::Xor(Box::new(a), Box::new(b)))),
                        '|' => Left(60, |_: &mut _, a, b| Ok(Expr::Or(Box::new(a), Box::new(b)))),
                        _ => fail
                    },
                    dispatch! {take(3usize);
                        "<=>" =>  Left(10, |_: &mut _, a, b| Ok(Expr::Equi(Box::new(a), Box::new(b)))),
                        _ => fail
                    },
                    dispatch! {take(2usize);
                        "->" =>  Left(20, |_: &mut _, a, b| Ok(Expr::Impl(Box::new(a), Box::new(b)))),
                        _ => fail
                    },
                )),
            )
            .parse_next(input)
            }
        }

        match parser(0).parse_next(input) {
            Ok(r) => {
                if eof::<&str, ErrMode<ContextError>>.parse_next(input).is_ok() {
                    Ok(r)
                } else {
                    Err(ErrMode::Cut(ContextError::default()))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
        trace(
            "identifier",
            (
                one_of(|c: char| c.is_alpha() || c == '_'),
                take_while(0.., |c: char| c.is_alphanum() || c == '_'),
            ),
        )
        .take()
        .parse_next(input)
    }

    fn false_lit<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
        trace("false_lit", "false").take().parse_next(input)
    }

    fn true_lit<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
        trace("true_lit", "true").take().parse_next(input)
    }
}

/// Evaluate the given [Expr] using the given [Assignment].
pub fn eval(assignment: &Assignment, expr: &Expr) -> bool {
    match expr {
        Expr::Var(a) => assignment[a.as_str()],
        Expr::Not(a) => !eval(assignment, a),
        Expr::Or(a, b) => eval(assignment, a) || eval(assignment, b),
        Expr::Xor(a, b) => eval(assignment, a) ^ eval(assignment, b),
        Expr::And(a, b) => eval(assignment, a) && eval(assignment, b),
        Expr::Equi(a, b) => eval(assignment, a) == eval(assignment, b),
        Expr::Impl(a, b) => !eval(assignment, a) || eval(assignment, b),
        Expr::True => true,
        Expr::False => false,
        Expr::Paren(a) => eval(assignment, a),
    }
}

/// Parse and evaluate the given formula using the given [Assignment].
pub fn run(formula: &str, assignment: &Assignment) -> Result<bool, String> {
    let input = formula.to_string();
    match pratt_parser(&mut input.as_str()) {
        Ok(expr) => Ok(eval(assignment, &expr)),
        Err(e) => Result::Err(e.to_string()),
    }
}

/// Assign boolean values to vars.
pub type Assignment = HashMap<String, bool>;

/// Truth table containing assignments and their result.
#[derive(Clone, Debug, PartialEq)]
pub struct TruthTable {
    pub rows: Vec<(Assignment, bool)>,
}

impl TruthTable {
    pub fn new() -> Self {
        TruthTable { rows: vec![] }
    }

    pub fn is_sat(&self) -> bool {
        self.rows.iter().any(|e| e.1)
    }

    pub fn is_tautology(&self) -> bool {
        self.rows.iter().all(|e| e.1)
    }

    pub fn is_contradiction(&self) -> bool {
        self.rows.iter().all(|e| !e.1)
    }

    pub fn vars(&self) -> Vec<String> {
        if self.rows.is_empty() {
            vec![]
        } else {
            let mut keys: Vec<String> = self.rows[0].0.keys().map(|s| s.to_string()).collect();
            keys.sort();
            keys
        }
    }

    /// The max_terms (false rows in the truth table) of a formula in CNF
    pub fn max_terms(&self) -> Vec<Expr> {
        fn collect_as_maxterm(assignment: &[(&str, bool)]) -> Expr {
            if assignment.is_empty() {
                panic!("can't create minterm out of empty vars");
            } else if assignment.len() == 1 {
                let (k, v) = assignment.iter().next().unwrap();
                let var = Expr::Var(k.to_string());
                if *v { Expr::Not(Box::new(var)) } else { var }
            } else {
                let (k, v) = assignment.iter().next().unwrap();
                let var = Expr::Var(k.to_string());
                let left = if *v { Expr::Not(Box::new(var)) } else { var };
                let right = collect_as_maxterm(&assignment[1..]);
                Expr::Or(Box::new(left), Box::new(right))
            }
        }

        self.rows
            .iter()
            .filter_map(|(assignment, result)| {
                if !result && !assignment.is_empty() {
                    let mut assi: Vec<(&str, bool)> =
                        assignment.iter().map(|(k, v)| (k.as_str(), *v)).collect();
                    assi.sort_by_key(|(k, _)| k.to_string());
                    let term = collect_as_maxterm(&assi);
                    Some(term)
                } else {
                    None
                }
            })
            .collect()
    }

    /// The min_terms (true rows in the truth table) of a formula in DNF
    pub fn min_terms(&self) -> Vec<Expr> {
        fn collect_as_minterm(assignment: &[(&str, bool)]) -> Expr {
            if assignment.is_empty() {
                panic!("can't create minterm out of empty vars");
            } else if assignment.len() == 1 {
                let (k, v) = assignment.iter().next().unwrap();
                let var = Expr::Var(k.to_string());
                if *v { var } else { Expr::Not(Box::new(var)) }
            } else {
                let (k, v) = assignment.iter().next().unwrap();
                let var = Expr::Var(k.to_string());
                let left = if *v { var } else { Expr::Not(Box::new(var)) };
                let right = collect_as_minterm(&assignment[1..]);
                Expr::And(Box::new(left), Box::new(right))
            }
        }

        self.rows
            .iter()
            .filter_map(|(assignment, result)| {
                if *result && !assignment.is_empty() {
                    let mut assi: Vec<(&str, bool)> =
                        assignment.iter().map(|(k, v)| (k.as_str(), *v)).collect();
                    assi.sort_by_key(|(k, _)| k.to_string());
                    let term = collect_as_minterm(&assi);
                    Some(term)
                } else {
                    None
                }
            })
            .collect()
    }

    /// The CNF (Conjunctive Normal Form), a conjunction of the maxterms.
    pub fn cnf(&self) -> Option<Expr> {
        let terms: Vec<_> = self
            .max_terms()
            .into_iter()
            .map(|expr| Expr::Paren(Box::new(expr)))
            .collect();
        if terms.is_empty() {
            None
        } else {
            let f = |left, right| Expr::And(Box::new(left), Box::new(right));
            Some(Expr::reduce(&terms, &f))
        }
    }

    /// The DNF (Disjunctive Normal Form), a disjunction of the minterms.
    pub fn dnf(&self) -> Option<Expr> {
        let terms: Vec<_> = self
            .min_terms()
            .into_iter()
            .map(|expr| Expr::Paren(Box::new(expr)))
            .collect();
        if terms.is_empty() {
            None
        } else {
            let f = |left, right| Expr::Or(Box::new(left), Box::new(right));
            Some(Expr::reduce(&terms, &f))
        }
    }
}

impl Default for TruthTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Construct all possible [Assignment]s for the given vars.
pub fn all_assignments(vars: Vec<String>) -> Vec<Assignment> {
    let mut vars = vars.clone();
    vars.sort();
    vars.reverse();
    let mut assignments = vec![];
    let s = vars.len();
    if s > 0 {
        for bit_assignment in 0..2usize.pow(s as u32) {
            let mut assignment = HashMap::new();
            (0..s).for_each(|idx| {
                let a = (bit_assignment >> idx) & 0x01 == 1;
                assignment.insert(vars[idx].to_string(), a);
            });
            assignments.push(assignment);
        }
    }
    assignments
}

/// Construct the [TruthTable] for all possible assignments for the given formula.
pub fn truth_table(formula: &str) -> std::result::Result<TruthTable, String> {
    let input = formula.to_string();
    match pratt_parser(&mut input.as_str()) {
        Ok(expr) => {
            let vars = expr.collect_vars();
            let assignments = all_assignments(vars);
            let mut table = TruthTable::new();
            assignments.into_iter().for_each(|a| {
                let r = eval(&a, &expr);
                let row = (a, r);
                table.rows.push(row);
            });
            std::result::Result::Ok(table)
        }
        ModalResult::Err(_) => std::result::Result::Err("parse error".to_string()),
    }
}

/// The max_terms (false rows in the truth table) of a formula in CNF
pub fn max_terms(formula: &str) -> std::result::Result<Vec<Expr>, String> {
    match truth_table(formula) {
        Ok(table) => Ok(table.max_terms()),
        Err(e) => Err(e),
    }
}

/// The min_terms (true rows in the truth table) of a formula in DNF
pub fn min_terms(formula: &str) -> std::result::Result<Vec<Expr>, String> {
    match truth_table(formula) {
        Ok(table) => Ok(table.min_terms()),
        Err(e) => Err(e),
    }
}

/// The CNF (Conjunctive Normal Form) of the formula, a conjunction of the maxterms.
pub fn cnf(formula: &str) -> std::result::Result<Option<Expr>, String> {
    match truth_table(formula) {
        Ok(table) => Ok(table.cnf()),
        Err(e) => Err(e),
    }
}

/// The DNF (Disjunctive Normal Form) of the formula, a disjunction of the minterms.
pub fn dnf(formula: &str) -> std::result::Result<Option<Expr>, String> {
    match truth_table(formula) {
        Ok(table) => Ok(table.dnf()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_empty_input_errors() {
        let expr = pratt_parser(&mut "");
        assert!(expr.is_err());
    }

    #[test]
    fn running_empty_input_errors() {
        let r = run("", &HashMap::new());
        assert!(r.is_err());
    }

    #[test]
    fn parsing_a_bool_literal_works() {
        let mut input = "true";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::True, expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_var_works() {
        let mut input = "a";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(Expr::Var("a".to_string()), expr.unwrap());
        assert_eq!("", input);
    }

    #[test]
    fn parsing_a_dangling_var_should_fail() {
        let mut input = "a b";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_err());
    }

    #[test]
    fn parsing_a_var_in_parens_works() {
        let mut input = "(a)";
        let expr = pratt_parser(&mut input);
        assert!(expr.is_ok());
        assert_eq!(
            Expr::Paren(Box::new(Expr::Var("a".to_string()))),
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
            Expr::Not(Box::new(Expr::Var("a".to_string()))),
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
            Expr::Or(
                Box::new(Expr::Var("a".to_string())),
                Box::new(Expr::Var("b".to_string()))
            ),
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
            Expr::And(
                Box::new(Expr::Var("a".to_string())),
                Box::new(Expr::Var("b".to_string()))
            ),
            expr.unwrap()
        );
        assert_eq!("", input);
    }

    #[test]
    fn assignment_works() {
        let a = Expr::Var("a".to_string());
        let b = Expr::Var("b".to_string());
        let mut assignment = HashMap::new();
        assignment.insert("a".into(), false);
        assignment.insert("b".into(), true);
        assert!(!eval(&assignment, &a));
        assert!(eval(&assignment, &b))
    }

    #[test]
    fn not_works() {
        let assignment = HashMap::new();
        let not_true = Expr::Not(Box::new(Expr::True));
        let not_false = Expr::Not(Box::new(Expr::False));
        assert!(!eval(&assignment, &not_true));
        assert!(eval(&assignment, &not_false))
    }

    #[test]
    fn or_works() {
        let assignment = HashMap::new();

        let expr = Expr::Or(Box::new(Expr::False), Box::new(Expr::False));
        assert!(!eval(&assignment, &expr));

        let expr = Expr::Or(Box::new(Expr::False), Box::new(Expr::True));
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or(Box::new(Expr::True), Box::new(Expr::False));
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or(Box::new(Expr::True), Box::new(Expr::True));
        assert!(eval(&assignment, &expr));
    }

    #[test]
    fn and_works() {
        let assignment = HashMap::new();

        let expr = Expr::And(Box::new(Expr::False), Box::new(Expr::False));
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And(Box::new(Expr::False), Box::new(Expr::True));
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And(Box::new(Expr::True), Box::new(Expr::False));
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And(Box::new(Expr::True), Box::new(Expr::True));
        assert!(eval(&assignment, &expr));
    }

    #[test]
    fn run_works() {
        let assignment = HashMap::new();
        assert!(run("!false", &assignment).unwrap());
        assert!(!run("!true", &assignment).unwrap());

        assert!(!run("false & false", &assignment).unwrap());
        assert!(!run("false & true", &assignment).unwrap());
        assert!(!run("true & false", &assignment).unwrap());
        assert!(run("true & true", &assignment).unwrap());

        assert!(run("false <=> false", &assignment).unwrap());
        assert!(!run("false <=> true", &assignment).unwrap());
        assert!(!run("true <=> false", &assignment).unwrap());
        assert!(run("true <=> true", &assignment).unwrap());

        assert!(run("false -> false", &assignment).unwrap());
        assert!(run("false -> true", &assignment).unwrap());
        assert!(!run("true -> false", &assignment).unwrap());
        assert!(run("true -> true", &assignment).unwrap());

        assert!(!run("false ^ false", &assignment).unwrap());
        assert!(run("false ^ true", &assignment).unwrap());
        assert!(run("true ^ false", &assignment).unwrap());
        assert!(!run("true ^ true", &assignment).unwrap());
    }

    #[test]
    fn precedence_works() {
        assert!(run("true | false & false", &HashMap::new()).unwrap());
    }

    #[test]
    fn material_implication_works() {
        let mut assignment = HashMap::new();

        assignment.insert("a".into(), false);
        assignment.insert("b".into(), false);
        assert!(run("a -> b <=> !a | b", &assignment).unwrap());

        assignment.insert("a".into(), false);
        assignment.insert("b".into(), true);
        assert!(run("a -> b <=> !a | b", &assignment).unwrap());

        assignment.insert("a".into(), true);
        assignment.insert("b".into(), false);
        assert!(run("a -> b <=> !a | b", &assignment).unwrap());

        assignment.insert("a".into(), true);
        assignment.insert("b".into(), true);
        assert!(run("a -> b <=> !a | b", &assignment).unwrap());
    }

    #[test]
    fn collect_vars_works() {
        let expr = pratt_parser(&mut "a").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a"]);

        let expr = pratt_parser(&mut "!a").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a"]);

        let expr = pratt_parser(&mut "a | b").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a", "b"]);

        let expr = pratt_parser(&mut "a & b").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a", "b"]);

        let expr = pratt_parser(&mut "a <=> b").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a", "b"]);

        let expr = pratt_parser(&mut "a -> b").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a", "b"]);

        let expr = pratt_parser(&mut "b -> (a -> b)").unwrap();
        let vars = expr.collect_vars();
        assert_eq!(vars, vec!["a", "b"]);
    }

    #[test]
    fn all_assignments_works() {
        let expr = pratt_parser(&mut "true").unwrap();
        let vars = expr.collect_vars();
        let assignments = all_assignments(vars);
        assert_eq!(assignments.len(), 0);

        let expr = pratt_parser(&mut "a").unwrap();
        let vars = expr.collect_vars();
        let assignments = all_assignments(vars);
        assert_eq!(assignments.len(), 2);

        let expr = pratt_parser(&mut "a & b | c -> d").unwrap();
        let vars = expr.collect_vars();
        let assignments = all_assignments(vars);
        assert_eq!(assignments.len(), 16);
    }

    #[test]
    fn truth_table_works() {
        let table = truth_table("a | b");
        assert!(table.is_ok());
        let table = table.unwrap();
        assert_eq!(table.rows.len(), 4);
        assert!(table.is_sat());
        assert!(!table.is_tautology());
        assert!(!table.is_contradiction());
        assert_eq!(table.vars(), vec!["a", "b"]);
    }

    #[test]
    fn max_terms_works() {
        let terms = max_terms("a & !a").unwrap();
        assert_eq!(terms.len(), 2);
        let terms = max_terms("a | !a").unwrap();
        assert!(terms.is_empty());

        let terms =
            max_terms("(a | b | c) & (a | !b | !c) & (!a | b | !c) & (!a | !b | c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "a | b | c");
        assert_eq!(terms[1].to_string(), "a | !b | !c");
        assert_eq!(terms[2].to_string(), "!a | b | !c");
        assert_eq!(terms[3].to_string(), "!a | !b | c");

        let terms =
            max_terms("(!a & !b & c) | (!a & b & !c) | (a & !b & !c) | (a & b & c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "a | b | c");
        assert_eq!(terms[1].to_string(), "a | !b | !c");
        assert_eq!(terms[2].to_string(), "!a | b | !c");
        assert_eq!(terms[3].to_string(), "!a | !b | c");

        let terms = max_terms("(!a & b) | (a & c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "a | b | c");
        assert_eq!(terms[1].to_string(), "a | b | !c");
        assert_eq!(terms[2].to_string(), "!a | b | c");
        assert_eq!(terms[3].to_string(), "!a | !b | c");
    }

    #[test]
    fn min_terms_works() {
        let terms = min_terms("a & !a").unwrap();
        assert!(terms.is_empty());
        let terms = min_terms("a | !a").unwrap();
        assert_eq!(terms.len(), 2);

        let terms =
            min_terms("(!a & !b & c) | (!a & b & !c) | (a & !b & !c) | (a & b & c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "!a & !b & c");
        assert_eq!(terms[0].to_string(), "!a & !b & c");
        assert_eq!(terms[1].to_string(), "!a & b & !c");
        assert_eq!(terms[2].to_string(), "a & !b & !c");
        assert_eq!(terms[3].to_string(), "a & b & c");

        let terms =
            min_terms("(a | b | c) & (a | !b | !c) & (!a | b | !c) & (!a | !b | c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "!a & !b & c");
        assert_eq!(terms[1].to_string(), "!a & b & !c");
        assert_eq!(terms[2].to_string(), "a & !b & !c");
        assert_eq!(terms[3].to_string(), "a & b & c");

        let terms = min_terms("(!a & b) | (a & c)").unwrap();
        assert_eq!(terms.len(), 4);
        assert_eq!(terms[0].to_string(), "!a & b & !c");
        assert_eq!(terms[1].to_string(), "!a & b & c");
        assert_eq!(terms[2].to_string(), "a & !b & c");
        assert_eq!(terms[3].to_string(), "a & b & c");
    }

    #[test]
    fn cnf_works() {
        let f = cnf("(!a & b) | (a & c)").unwrap().unwrap();
        assert_eq!(
            f.to_string(),
            "(a | b | c) & (a | b | !c) & (!a | b | c) & (!a | !b | c)"
        );
    }

    #[test]
    fn cnf_works_for_tautology() {
        let formula = "a | true";
        let terms = max_terms(formula).unwrap();
        assert_eq!(terms.len(), 0);
        assert!(cnf(formula).unwrap().is_none());
    }

    #[test]
    fn dnf_works() {
        let f = dnf("(!a & b) | (a & c)").unwrap().unwrap();
        assert_eq!(
            f.to_string(),
            "(!a & b & !c) | (!a & b & c) | (a & !b & c) | (a & b & c)"
        );
    }

    #[test]
    fn dnf_works_for_contradiction() {
        let formula = "a & false";
        let terms = min_terms(formula).unwrap();
        assert_eq!(terms.len(), 0);
        assert!(dnf(formula).unwrap().is_none());
    }
}
