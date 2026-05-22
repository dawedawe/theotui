use std::collections::HashSet;

pub type Terminal = String;
pub type NonTerminal = String;
pub type Rhs = String;
pub type ProductionRule = (NonTerminal, Rhs);

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
        type2grammar::{NonTerminal, ProductionRule, Terminal, Type2Grammar},
    };

    /// Expressions of the [Type2Grammar] definition.
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
        take_while(1..=1, |c: char| {
            !c.is_whitespace() && !c.is_uppercase() && !c.is_control() && c != '\''
        })
    }

    /// Parses the right side of a production
    pub fn production_rhs<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(0.., |c: char| {
            !c.is_whitespace() && !c.is_control() && c != '\''
        })
    }

    /// Parses a production rule like `S -> 'aTa'`
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

    /// Parse a [Type2Grammar] definition (V, Sigma, P, S)
    pub fn parse_t2grammar_definition(input: &str) -> Result<Type2Grammar, String> {
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
            Type2Grammar::new(nonterminals, sigma, productions, start)
        } else {
            Err("Incomplete definition".into())
        }
    }
}

/// Defines a Type-2 Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type2Grammar {
    pub(crate) nonterminals: HashSet<NonTerminal>,
    pub(crate) sigma: HashSet<Terminal>,
    pub(crate) productions: HashSet<ProductionRule>,
    pub(crate) start: NonTerminal,
}

impl Type2Grammar {
    /// Constructs a valid [Type2Grammar]
    pub fn new(
        nonterminals: HashSet<NonTerminal>,
        sigma: HashSet<Terminal>,
        productions: HashSet<(NonTerminal, String)>,
        start: NonTerminal,
    ) -> Result<Self, String> {
        if !nonterminals.contains(&start) {
            return Err("start must be an element of V.".into());
        }

        let (mut unknown_prod_nonterms, mut unknown_prod_terms): (Vec<String>, Vec<String>) =
            productions.iter().fold(
                (vec![], vec![]),
                |(mut unknown_nonterms, mut unknown_terms): (Vec<String>, Vec<_>), (nt, rhs)| {
                    if !nonterminals.contains(nt) {
                        unknown_nonterms.push(nt.to_string());
                    }

                    let terms: Vec<_> = rhs.chars().filter(|c| !c.is_uppercase()).collect();
                    terms.iter().for_each(|c| {
                        if !sigma.contains(&c.to_string()) {
                            unknown_terms.push(c.to_string());
                        }
                    });

                    let non_terms: Vec<_> = rhs.chars().filter(|c| c.is_uppercase()).collect();
                    non_terms.iter().for_each(|c| {
                        if !nonterminals.contains(&c.to_string()) {
                            unknown_nonterms.push(c.to_string());
                        }
                    });
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

        Ok(Type2Grammar {
            nonterminals,
            sigma,
            productions,
            start,
        })
    }

    /// The non-terminals of the [Type2Grammar].
    pub fn nonterminals(&self) -> &HashSet<String> {
        &self.nonterminals
    }

    /// The alphabet sigma of the [Type2Grammar].
    pub fn sigma(&self) -> &HashSet<Terminal> {
        &self.sigma
    }

    /// The start non-terminal of the [Type2Grammar].
    pub fn start(&self) -> &NonTerminal {
        &self.start
    }

    /// The productions of the [Type2Grammar].
    pub fn productions(&self) -> &HashSet<(String, String)> {
        &self.productions
    }

    fn possible_productions(&self, nonterm_to_expand: String) -> Vec<ProductionRule> {
        self.productions
            .iter()
            .filter(|p| p.0 == nonterm_to_expand)
            .cloned()
            .collect()
    }

    fn process_stack_top(
        &self,
        word: String,
        mut stack: Vec<String>,
        acc: Vec<ProductionRule>,
    ) -> Vec<(String, Vec<String>, Vec<ProductionRule>)> {
        // bail out if we are already longer that the input word
        if stack.iter().filter(|c| self.sigma.contains(*c)).count() > word.len() {
            return vec![];
        }

        match stack.pop() {
            Some(popped) => {
                if self.nonterminals.contains(&popped) {
                    let possible_productions = self.possible_productions(popped);
                    let mut stacks = vec![];
                    for p in possible_productions {
                        let mut stack_for_p = stack.clone();
                        let input_for_p = word.to_string().clone();
                        p.1.chars()
                            .rev()
                            .for_each(|c| stack_for_p.push(c.to_string()));
                        let mut acc = acc.clone();
                        acc.push(p);
                        stacks.push((input_for_p, stack_for_p, acc));
                    }
                    stacks
                } else if self.sigma.contains(&popped) {
                    if word.starts_with(popped.as_str()) {
                        let word = word.replacen(popped.as_str(), "", 1);
                        vec![(word, stack, acc)]
                    } else {
                        vec![]
                    }
                } else if popped.is_empty() {
                    vec![(word, stack, acc)]
                } else {
                    panic!("unknown stack top")
                }
            }
            None => vec![],
        }
    }

    /// Tries to find a production chain that produces the given word.
    pub fn try_find_productions(&self, word: &str) -> Option<Vec<ProductionRule>> {
        let word = word.to_string();
        let stack: Vec<String> = vec![self.start.clone()];
        let acc: Vec<ProductionRule> = vec![];
        let mut states = vec![(word, stack, acc)];
        let mut found = None; // vec![];

        while found.is_none() && !states.is_empty() {
            states = states
                .into_iter()
                .flat_map(|(w, s, acc)| self.process_stack_top(w, s, acc))
                .collect();
            found = states.iter().find_map(|(w, s, acc)| {
                if w.is_empty() && s.is_empty() {
                    Some(acc.clone())
                } else {
                    None
                }
            });
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::type2grammar::parser::Expr;

    use super::*;

    #[test]
    fn start_must_be_known() {
        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), "a".into())]),
            "X".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_nonterms_must_be_known() {
        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("X".into(), "a".into())]),
            "S".into(),
        );
        assert!(g.is_err());

        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), "aX".into())]),
            "S".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_symbols_must_be_known() {
        let g = Type2Grammar::new(
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
    fn parse_t2grammar_definition_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
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
    fn parse_left_regular_t2grammar_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'Ta' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_t2grammar_with_left_right_mixed_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_t2grammar_definition_with_missing_but_duplicated_parts_should_fail() {
        let s = "
    Sigma = { 'a', 'b' }
    V = { S, T, W }
    Sigma = { 'a', 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn productions_work_for_epsilon() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> '', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("");
        assert_eq!(r, Some(vec![("S".into(), "".into())]));
    }

    #[test]
    fn productions_work_for_single_symbol_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> 'a', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("a");
        assert_eq!(r, Some(vec![("S".into(), "a".into())]));
    }

    #[test]
    fn productions_work_for_two_symbol_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("ab");
        assert_eq!(
            r,
            Some(vec![("S".into(), "aT".into()), ("T".into(), "b".into())])
        );
    }

    #[test]
    fn productions_work_for_balanced_parens() {
        let s = "
    V = { S, T }
    Sigma = { '(', ')' }
    P = { S -> '(S)', S -> '()' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("()").is_some());
        assert!(g.try_find_productions("(())").is_some());
        assert!(g.try_find_productions("((()))").is_some());
    }

    #[test]
    fn productions_work_for_aba() {
        let s = "
    V = { S, A, B }
    Sigma = { 'a', 'b' }
    P = { S -> 'AB', S -> 'ABA', A -> 'aA', A -> 'a', B -> 'Bb', B -> '' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("a").is_some());
        assert!(g.try_find_productions("aa").is_some());
        assert!(g.try_find_productions("aabbaa").is_some());
        assert!(g.try_find_productions("abbbbb").is_some());
    }

    #[test]
    fn productions_work_for_arith() {
        let s = "
    V = { E, O }
    Sigma = { 'a', '(', ')', '+', '-', '*', '/' }
    P = { E -> 'a', E -> 'EOE', E -> '(E)', O -> '+', O -> '-', O -> '*', O -> '/' }
    S = E
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("a*((a-a)/a)").is_some());
    }

    #[test]
    fn productions_stop_for_impossible_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("aba").is_none());
    }

    #[test]
    fn productions_stop_for_word_with_invalid_terminals() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("abx").is_none());
    }

    #[test]
    fn invalid_prefix_should_fail() {
        let s = "xxx V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn invalid_postfix_should_fail() {
        let s = "V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S xxx";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }
}
