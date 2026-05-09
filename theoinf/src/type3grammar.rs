use std::collections::HashSet;

pub mod parser {
    use std::collections::HashSet;

    use winnow::{
        ModalResult, Parser,
        ascii::multispace0,
        combinator::{alt, cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode, StrContext},
        stream::AsChar,
        token::take_while,
    };

    use crate::{
        parser_utils::whitespace_wrapped,
        type3grammar::{NonTerminal, ProductionRule, Terminal, Type3Grammar},
    };

    /// Expressions of the type-3 grammar definition.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Expr {
        NonTerms(Vec<String>),
        Sigma(Vec<String>),
        Productions(Vec<ProductionRule>),
        Start(String),
    }

    /// Parses a non-terminals definition like `V = { S, T, W }`
    pub fn parse_v_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("V");
        let equals = whitespace_wrapped("=");
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., nonterminal_name(), separator);
        let setp = delimited(
            delimited(multispace0, "{", multispace0),
            comma_sep_list,
            delimited(multispace0, cut_err("}"), multispace0),
        );
        let mut decl = (identifier, equals, setp)
            .context(StrContext::Label("V definition"))
            .map(|(_, _, x): (_, _, Vec<&str>)| {
                Expr::NonTerms(x.iter().map(|s| s.to_string()).collect())
            });
        decl.parse_next(input)
    }

    /// Parses a Sigma definition like `Sigma = { 'a', 'b', 'c' }`
    pub fn parse_sigma_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("Sigma");
        let equals = whitespace_wrapped("=");
        let element = delimited("'", terminal_symbol(), cut_err("'"));
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., element, separator);
        let setp = delimited(
            delimited(multispace0, "{", multispace0),
            comma_sep_list,
            delimited(multispace0, cut_err("}"), multispace0),
        );
        let mut decl = (identifier, equals, setp)
            .context(StrContext::Label("Sigma definition"))
            .map(|(_, _, x): (_, _, Vec<&str>)| {
                Expr::Sigma(x.iter().map(|s| s.to_string()).collect())
            });
        decl.parse_next(input)
    }

    pub fn nonterminal_name<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1..=1, |c: char| c.is_alpha() && c.is_uppercase())
    }

    pub fn terminal_symbol<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1..=1, |c: char| c.is_alpha() && c.is_lowercase())
    }

    /// Parses the right side of a right or left-regular production like aT or Ta or a or epsilon
    pub fn production_rhs<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        let right_reg = (terminal_symbol(), nonterminal_name()).take();
        let left_reg = (nonterminal_name(), terminal_symbol()).take();
        alt((right_reg, left_reg, terminal_symbol(), ""))
    }

    /// Parses a right or left-regular production rule like `S -> 'aT'` or `S -> 'Ta'`
    pub fn production_rule<'s>() -> impl Parser<&'s str, (&'s str, &'s str), ErrMode<ContextError>>
    {
        let element = delimited("'", production_rhs(), cut_err("'"));
        let tuple = (nonterminal_name(), whitespace_wrapped("->"), element);
        trace("production_rule", tuple.map(|(nt, _arrow, rhs)| (nt, rhs)))
    }

    /// Parses a production set like `{ S -> 'aT', T -> 'b' }`
    pub fn production_set<'s>()
    -> impl Parser<&'s str, Vec<(&'s str, &'s str)>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(0.., production_rule(), separator);
        trace(
            "production_set",
            delimited(
                delimited(multispace0, "{", multispace0),
                comma_sep_list,
                delimited(multispace0, cut_err("}"), multispace0),
            ),
        )
    }

    /// Parse a productions definition like `P = { (S, 'aT'), (T, 'b') }`
    pub fn parse_productions_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("P");
        let equals = whitespace_wrapped("=");
        (identifier, equals, production_set())
            .context(StrContext::Label("P definition"))
            .map(|(_, _, x): (_, _, Vec<(&str, &str)>)| {
                Expr::Productions(
                    x.iter()
                        .map(|(l, r)| (l.to_string(), r.to_string()))
                        .collect(),
                )
            })
            .parse_next(input)
    }

    /// Parse a start symbol definition like `S = S`
    pub fn parse_start_nonterminal_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("S");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, nonterminal_name(), multispace0);
        (identifier, equals, state)
            .context(StrContext::Label("S definition"))
            .map(|(_, _, x): (&str, &str, &str)| Expr::Start(x.to_string()))
            .parse_next(input)
    }

    /// Parse a [Type3Grammar] definition (V, Sigma, P, S)
    pub fn parse_t3grammar_definition(input: &str) -> Result<Type3Grammar, String> {
        let mut input = input;
        let mut nonterminals: Option<HashSet<NonTerminal>> = None;
        let mut sigma: Option<HashSet<Terminal>> = None;
        let mut productions: Option<HashSet<(NonTerminal, String)>> = None;
        let mut start: Option<NonTerminal> = None;

        let mut alt_parser = alt((
            parse_sigma_definition,
            parse_v_definition,
            parse_productions_definition,
            parse_start_nonterminal_definition,
        ));

        for _ in 0..4 {
            let r = alt_parser.parse_next(&mut input);
            match r {
                Ok(expr) => match expr {
                    Expr::NonTerms(nonterms) => {
                        nonterminals = Some(nonterms.into_iter().collect());
                    }
                    Expr::Sigma(terms) => {
                        sigma = Some(terms.into_iter().collect());
                    }
                    Expr::Productions(prods) => {
                        productions = Some(prods.into_iter().collect());
                    }
                    Expr::Start(s) => start = Some(s),
                },
                Err(s) => return Err(s.to_string()),
            }
        }

        if !input.trim().is_empty() {
            Err("bad definition".into())
        } else if let (Some(nonterminals), Some(sigma), Some(productions), Some(start)) =
            (nonterminals, sigma, productions, start)
        {
            Type3Grammar::new(nonterminals, sigma, productions, start)
        } else {
            Err("Incomplete definition".into())
        }
    }
}

pub type Terminal = String;
pub type NonTerminal = String;
pub type Rhs = String;
pub type ProductionRule = (NonTerminal, Rhs);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Left,
    Right,
}

/// Defines a Type-3 Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type3Grammar {
    pub(crate) nonterminals: HashSet<NonTerminal>,
    pub(crate) sigma: HashSet<Terminal>,
    pub(crate) productions: HashSet<ProductionRule>,
    pub(crate) start: NonTerminal,
    pub(crate) kind: Kind,
}

impl Type3Grammar {
    /// Constructs a valid [Type3Grammar]
    pub fn new(
        nonterminals: HashSet<NonTerminal>,
        sigma: HashSet<Terminal>,
        productions: HashSet<(NonTerminal, String)>,
        start: NonTerminal,
    ) -> Result<Self, String> {
        if !nonterminals.contains(&start) {
            return Err("start must be an element of V.".into());
        }

        let mut right_reg_detected = false;
        let mut left_reg_detected = false;
        let (mut unknown_prod_nonterms, mut unknown_prod_terms): (Vec<String>, Vec<String>) =
            productions.iter().fold(
                (vec![], vec![]),
                |(mut unknown_nonterms, mut unknown_terms): (Vec<String>, Vec<_>), (nt, rhs)| {
                    if !nonterminals.contains(nt) {
                        unknown_nonterms.push(nt.to_string());
                    }
                    if rhs.len() == 1 && !sigma.contains(rhs) {
                        unknown_terms.push(rhs.to_string());
                    }
                    if rhs.len() == 2 {
                        let mut chars = rhs.chars();
                        let first = chars.next().unwrap();
                        let second = chars.next().unwrap();

                        let (t, nt) = if first.is_lowercase() {
                            right_reg_detected = true;
                            (first.to_string(), second.to_string())
                        } else {
                            left_reg_detected = true;
                            (second.to_string(), first.to_string())
                        };
                        if !sigma.contains(&t) {
                            unknown_terms.push(t);
                        }
                        if !nonterminals.contains(&nt) {
                            unknown_nonterms.push(nt);
                        }
                    }
                    (unknown_nonterms, unknown_terms)
                },
            );

        if !unknown_prod_nonterms.is_empty() {
            unknown_prod_nonterms.sort();
            unknown_prod_nonterms.dedup();
            let s = unknown_prod_nonterms.join(", ");
            let msg = format!("The productions contain the following unknown non-terminals: {s}");
            return Err(msg);
        }

        if !unknown_prod_terms.is_empty() {
            unknown_prod_terms.sort();
            unknown_prod_terms.dedup();
            let s = unknown_prod_terms.join(", ");
            let msg = format!("The productions contain the following unknown symbols: {s}");
            return Err(msg);
        }

        let kind = match (right_reg_detected, left_reg_detected) {
            (true, true) => {
                return Err("Both right and left-regular production rules detected.".into());
            }
            (false, false) => Kind::Right,
            (true, false) => Kind::Right,
            (false, true) => Kind::Left,
        };

        Ok(Type3Grammar {
            nonterminals,
            sigma,
            productions,
            start,
            kind,
        })
    }

    /// The non-terminals of the [Type3Grammar].
    pub fn nonterminals(&self) -> &HashSet<String> {
        &self.nonterminals
    }

    /// The alphabet sigma of the [Type3Grammar].
    pub fn sigma(&self) -> &HashSet<Terminal> {
        &self.sigma
    }

    /// The start non-terminal of the [Type3Grammar].
    pub fn start(&self) -> &NonTerminal {
        &self.start
    }

    /// The productions of the [Type3Grammar].
    pub fn productions(&self) -> &HashSet<(String, String)> {
        &self.productions
    }

    /// The [Kind] of the [Type3Grammar].
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Finds and applies possbile production rules. Returns tuples of remaining word, remaining
    /// non-terminal and rules chain.
    fn find_and_apply_productions<'s>(
        &self,
        word: &'s str,
        nonterm: &str,
        rules_acc: Vec<ProductionRule>,
    ) -> HashSet<(&'s str, Option<char>, Vec<ProductionRule>)> {
        self.productions
            .iter()
            .filter(|(lhs, rhs)| {
                lhs == nonterm && {
                    let rhs_chars: Vec<_> = rhs.chars().collect();
                    let rhs_len = rhs_chars.len();
                    if self.kind == Kind::Right {
                        rhs_len > 0 && word.starts_with(rhs_chars[0])
                            || rhs_len == 0 && word.is_empty()
                    } else {
                        rhs_len == 2 && word.ends_with(rhs_chars[1])
                            || rhs_len == 1 && word.ends_with(rhs_chars[0])
                            || rhs_len == 0 && word.is_empty()
                    }
                }
            })
            .map(|(lhs, rhs)| {
                let rhs_chars: Vec<_> = rhs.chars().collect();
                let rhs_len = rhs_chars.len();
                let mut rules_acc = rules_acc.clone();
                rules_acc.push((lhs.clone(), rhs.clone()));
                match (&self.kind, rhs_len) {
                    (_, 0) => (word, None, rules_acc),
                    (Kind::Left, 1) => (&word[0..word.len() - 1], None, rules_acc),
                    (Kind::Right, 1) => (&word[1..], None, rules_acc),
                    (Kind::Left, 2) => (&word[0..word.len() - 1], Some(rhs_chars[0]), rules_acc),
                    (Kind::Right, 2) => (&word[1..], Some(rhs_chars[1]), rules_acc),
                    _ => panic!("The parser should have caught this invalid rule."),
                }
            })
            .collect()
    }

    /// Applies applicable production rules till word is consumed or grammar fails to produce given
    /// word.
    fn produces_helper(
        &self,
        word: &str,
        non_term: Option<char>,
        rules_acc: Vec<ProductionRule>,
    ) -> Vec<Vec<ProductionRule>> {
        if word.is_empty() && non_term.is_none() {
            // remaining word is empty and no non-terminal needs to be expanded
            vec![rules_acc]
        } else if non_term.is_none() {
            // remaining word is not empty but no non-terminal can be expanded
            vec![]
        } else {
            let nt = non_term.unwrap().to_string();
            self.find_and_apply_productions(word, nt.as_str(), rules_acc)
                .iter()
                .flat_map(|(word, product, acc)| self.produces_helper(word, *product, acc.clone()))
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    /// Tries to find production chains that produce the given word.
    pub fn try_find_productions(&self, word: &str) -> Vec<Vec<ProductionRule>> {
        self.find_and_apply_productions(word, &self.start, vec![])
            .into_iter()
            .flat_map(|(word, non_term, acc)| self.produces_helper(word, non_term, acc))
            .filter(|acc| !acc.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::type3grammar::parser::Expr;

    use super::*;

    #[test]
    fn start_must_be_known() {
        let g = Type3Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), "a".into())]),
            "X".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_nonterms_must_be_known() {
        let g = Type3Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("X".into(), "a".into())]),
            "S".into(),
        );
        assert!(g.is_err());

        let g = Type3Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), "aX".into())]),
            "S".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_symbols_must_be_known() {
        let g = Type3Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), "xT".into())]),
            "S".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn parse_sigma_works() {
        let mut s = "Sigma = { 'a' , 'b','c' } ";
        let symbols = parser::parse_sigma_definition(&mut s).unwrap();
        assert_eq!(
            symbols,
            Expr::Sigma(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn parse_empty_sigma_should_fail() {
        let mut s = "Sigma = { } ";
        let r = parser::parse_sigma_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_v_definition_works() {
        let mut s = "V = { S , T,W  } ";
        let v = parser::parse_v_definition(&mut s).unwrap();
        assert_eq!(v, Expr::NonTerms(vec!["S".into(), "T".into(), "W".into()]));
    }

    #[test]
    fn parse_empty_v_definition_should_fail() {
        let mut s = "V = {} ";
        let r = parser::parse_v_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_start_state_works() {
        let mut s = "S = S";
        let nt = parser::parse_start_nonterminal_definition(&mut s).unwrap();
        assert_eq!(nt, Expr::Start("S".to_string()));
    }

    #[test]
    fn production_rhs_works_for_right_regular() {
        let r = parser::production_rhs().parse_next(&mut "").unwrap();
        assert_eq!(r, "");
        let r = parser::production_rhs().parse_next(&mut "a").unwrap();
        assert_eq!(r, "a");
        let r = parser::production_rhs().parse_next(&mut "aT").unwrap();
        assert_eq!(r, "aT");
    }

    #[test]
    fn production_rhs_works_for_left_regular() {
        let r = parser::production_rhs().parse_next(&mut "").unwrap();
        assert_eq!(r, "");
        let r = parser::production_rhs().parse_next(&mut "a").unwrap();
        assert_eq!(r, "a");
        let r = parser::production_rhs().parse_next(&mut "Ta").unwrap();
        assert_eq!(r, "Ta");
    }

    #[test]
    fn parse_production_definition_works_for_right_regular() {
        let mut s = "P = { S-> 'aT', T ->'b', B -> '' }";
        let r = parser::parse_productions_definition(&mut s).unwrap();
        assert_eq!(
            r,
            Expr::Productions(vec![
                ("S".into(), "aT".into()),
                ("T".into(), "b".into()),
                ("B".into(), "".into())
            ])
        );
    }

    #[test]
    fn production_rule_works_for_left_regular() {
        let mut s = "S -> 'Ta'";
        let r = parser::production_rule().parse_next(&mut s);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), ("S", "Ta"));
    }

    #[test]
    fn parse_production_definition_works_for_left_regular() {
        let mut s = "P = { S -> 'Ta', T -> 'b', B -> '' }";
        let r = parser::parse_productions_definition(&mut s).unwrap();
        assert_eq!(
            r,
            Expr::Productions(vec![
                ("S".into(), "Ta".into()),
                ("T".into(), "b".into()),
                ("B".into(), "".into())
            ])
        );
    }

    #[test]
    fn parse_t3grammar_definition_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_ok());
        let g = r.unwrap();
        assert_eq!(
            g.nonterminals(),
            &HashSet::from_iter(["S".into(), "T".into()])
        );
        assert_eq!(g.sigma(), &HashSet::from_iter(["a".into(), "b".into()]));
        assert_eq!(
            g.productions(),
            &HashSet::from_iter([("S".into(), "aT".into()), ("T".into(), "b".into())])
        );
        assert_eq!(g.start, "S");
    }

    #[test]
    fn parse_left_regular_t3grammar_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'Ta' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_t3grammar_with_left_right_mixed_should_fail() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_t3grammar_definition_with_missing_but_duplicated_parts_should_fail() {
        let s = "
    Sigma = { 'a', 'b' }
    V = { S, T, W }
    Sigma = { 'a', 'b' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn no_productions_are_found_for_impossible_word() {
        let s = "
    V = { S }
    Sigma = { 'a' }
    P = { S -> 'a' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        assert!(g.try_find_productions("ab").is_empty());
    }

    #[test]
    fn productions_are_found_for_single_terminal() {
        let s = "
    V = { S }
    Sigma = { 'a', 'b' }
    P = { S -> 'a', S -> 'b' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        let r = &g.try_find_productions("a")[0];
        assert_eq!(r, &vec![("S".into(), "a".into())]);
    }

    #[test]
    fn productions_are_found_for_2_terminals() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b' }
    P = { S -> 'aT', T ->'b' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        let r = &g.try_find_productions("ab")[0];
        assert_eq!(
            r,
            &vec![("S".into(), "aT".into()), ("T".into(), "b".into())]
        );
    }

    #[test]
    fn productions_are_found_for_2_terminals_left_reg() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b' }
    P = { S -> 'Tb', T ->'a' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        let r = &g.try_find_productions("ab")[0];
        assert_eq!(
            r,
            &vec![("S".into(), "Tb".into()), ("T".into(), "a".into())]
        );
    }

    #[test]
    fn productions_are_found_for_n_terminals() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b', 'c' }
    P = { S -> 'aT', T -> 'b', T -> 'bT', T -> 'c' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        assert!(!g.try_find_productions("abbbbbc").is_empty());
    }

    #[test]
    fn productions_are_found_for_n_terminals_left_reg() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b', 'c' }
    P = { S -> 'Tb', S -> 'Tc', T -> 'Tb', T -> 'a' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        assert!(!g.try_find_productions("abbbbbc").is_empty());
    }

    #[test]
    fn production_work_for_epsilon() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b', 'c' }
    P = { S -> 'aT', T -> 'b', T -> 'bT', T -> '' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        let r = &g.try_find_productions("a")[0];
        assert_eq!(r, &vec![("S".into(), "aT".into()), ("T".into(), "".into())]);
    }

    #[test]
    fn production_work_for_epsilon_left_reg() {
        let s = "
            V = { S, T }
            Sigma = { 'a',
                      'b',
                      'c' }
            P = { S -> 'Tb',
                  S -> 'Ta',
                  T -> 'a',
                  T -> 'Tb',
                  T -> '' }
            S = S ";
        let g = parser::parse_t3grammar_definition(s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        let r = &g.try_find_productions("a")[0];
        assert_eq!(r, &vec![("S".into(), "Ta".into()), ("T".into(), "".into())]);
    }

    #[test]
    fn invalid_prefix_should_fail() {
        let s = "xxx V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S ";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn invalid_postfix_should_fail() {
        let s = "V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S xxx";
        let r = parser::parse_t3grammar_definition(s);
        assert!(r.is_err());
    }
}
