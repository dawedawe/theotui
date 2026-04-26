use std::collections::HashSet;

pub mod parser {
    use std::collections::HashSet;

    use winnow::{
        ModalResult, Parser,
        ascii::multispace0,
        combinator::{alt, cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode},
        stream::AsChar,
        token::take_while,
    };

    use crate::type3grammar::{NonTerminal, Terminal, Type3Grammar};

    fn whitespace_wrapped<'i>(s: &str) -> impl Parser<&'i str, &'i str, ErrMode<ContextError>> {
        trace("whitespace_wrapped", delimited(multispace0, s, multispace0))
    }

    /// Parses a non-terminals definition like `V = { S, T, W }`
    pub fn parse_v_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
        let identifier = whitespace_wrapped("V");
        let equals = whitespace_wrapped("=");
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., nonterminal_name(), separator);
        let setp = delimited(
            delimited(multispace0, "{", multispace0),
            comma_sep_list,
            delimited(multispace0, cut_err("}"), multispace0),
        );
        let mut decl = (identifier, equals, setp).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    /// Parses a Sigma definition like `Sigma = { 'a', 'b', 'c' }`
    pub fn parse_sigma_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
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
        let mut decl = (identifier, equals, setp).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    pub fn nonterminal_name<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1.., |c: char| c.is_alpha() && c.is_uppercase())
    }

    pub fn terminal_symbol<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1.., |c: char| c.is_alpha() && c.is_lowercase())
    }

    /// Parses the right side of a production like aT or a
    pub fn production_rhs<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        let term_non_term = (terminal_symbol(), nonterminal_name()).take();
        alt((term_non_term, terminal_symbol(), ""))
    }

    /// Parses a production tuple like `(S, 'aT')`
    pub fn production_tuple<'s>() -> impl Parser<&'s str, (&'s str, &'s str), ErrMode<ContextError>>
    {
        let element = delimited("'", production_rhs(), cut_err("'"));
        let tuple = (nonterminal_name(), whitespace_wrapped("->"), element);
        trace(
            "delta_tuple",
            tuple.map(|(nt, _arrow, rhs)| (nt, rhs.trim_matches('\''))),
        )
    }

    /// Parses a production set like `{ (S, 'aT'), (T, 'b') }`
    pub fn production_set<'s>()
    -> impl Parser<&'s str, Vec<(&'s str, &'s str)>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(0.., production_tuple(), separator);
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
    pub fn parse_productions_definition<'s>(
        input: &'s mut &str,
    ) -> ModalResult<Vec<(&'s str, &'s str)>> {
        let identifier = whitespace_wrapped("P");
        let equals = whitespace_wrapped("=");
        (identifier, equals, production_set())
            .map(|(_, _, x)| x)
            .parse_next(input)
    }

    /// Parse a start symbol definition like `S = S`
    pub fn parse_start_nonterminal_definition<'s>(input: &'s mut &str) -> ModalResult<&'s str> {
        let identifier = whitespace_wrapped("S");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, nonterminal_name(), multispace0);
        (identifier, equals, state)
            .map(|(_, _, x)| x)
            .parse_next(input)
    }

    /// Parse a [Type3Grammar] definition (V, Sigma, P, S)
    pub fn parse_t3grammar_definition(input: &mut &str) -> Result<Type3Grammar, String> {
        let lines: Vec<String> = input
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() != 4 {
            return Err("Incomplete definition".into());
        }

        let mut nonterminals: Option<HashSet<NonTerminal>> = None; // V
        let mut sigma: Option<HashSet<Terminal>> = None;
        let mut productions: Option<HashSet<(NonTerminal, String)>> = None;
        let mut start: Option<NonTerminal> = None;

        for line in lines {
            let mut line: &str = &line;
            if line.starts_with("Sigma") {
                let r =
                    parse_sigma_definition(&mut line).map_err(|_| "Invalid 'Sigma' definition.")?;
                sigma = Some(r.into_iter().map(|s| s.to_string()).collect());
            } else if line.starts_with("S=") || line.starts_with("S ") {
                let r = parse_start_nonterminal_definition(&mut line)
                    .map_err(|_| "Invalid 'S' definition.")?;
                start = Some(r.to_string());
            } else if line.starts_with("V") {
                let r = parse_v_definition(&mut line).map_err(|_| "Invalid 'V' definition.")?;
                nonterminals = Some(r.into_iter().map(|s| s.to_string()).collect());
            } else if line.starts_with("P") {
                let r = parse_productions_definition(&mut line)
                    .map_err(|_| "Invalid 'P' definition.")?;
                productions = Some(
                    r.into_iter()
                        .map(|(nt, rhs)| (nt.to_string(), rhs.to_string()))
                        .collect(),
                );
            } else {
                return Err(format!("Can't parse line '{line}'."));
            }
        }

        if let (Some(nonterminals), Some(sigma), Some(productions), Some(start)) =
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

/// Defines a Type-3 Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type3Grammar {
    pub(crate) nonterminals: HashSet<NonTerminal>,
    pub(crate) sigma: HashSet<Terminal>,
    pub(crate) productions: HashSet<ProductionRule>,
    pub(crate) start: NonTerminal,
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
                        let first = chars.next().unwrap().to_string();
                        let second = chars.next().unwrap().to_string();
                        if !sigma.contains(&first) {
                            unknown_terms.push(first.to_string());
                        }
                        if !nonterminals.contains(&second) {
                            unknown_nonterms.push(second.to_string());
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

        Ok(Type3Grammar {
            nonterminals,
            sigma,
            productions,
            start,
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
                    rhs_len > 0 && word.starts_with(rhs_chars[0]) || rhs_len == 0 && word.is_empty()
                }
            })
            .map(|(lhs, rhs)| {
                let rhs_chars: Vec<_> = rhs.chars().collect();
                let rhs_len = rhs_chars.len();
                let mut rules_acc = rules_acc.clone();
                rules_acc.push((lhs.clone(), rhs.clone()));
                match rhs_len {
                    0 => (word, None, rules_acc),
                    1 => (&word[1..], None, rules_acc),
                    2 => (&word[1..], Some(rhs_chars[1]), rules_acc),
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
        assert_eq!(symbols, vec!["a", "b", "c"]);
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
        assert_eq!(v, vec!["S", "T", "W"]);
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
        assert_eq!(nt, "S");
    }

    #[test]
    fn parse_production_rhs_works() {
        let r = parser::production_rhs().parse_next(&mut "").unwrap();
        assert_eq!(r, "");
        let r = parser::production_rhs().parse_next(&mut "a").unwrap();
        assert_eq!(r, "a");
        let r = parser::production_rhs().parse_next(&mut "aT").unwrap();
        assert_eq!(r, "aT");
    }

    #[test]
    fn parse_production_definition_works() {
        let mut s = "P = { S-> 'aT', T ->'b', B -> '' }";
        let r = parser::parse_productions_definition(&mut s).unwrap();
        assert_eq!(r, vec![("S", "aT"), ("T", "b"), ("B", "")]);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn parse_production_should_fail_for_left_regular_production() {
        let mut s = "P = { S -> 'Ta', T -> 'b' }";
        let r = parser::parse_productions_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_t3grammar_definition_works() {
        let mut s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(&mut s);
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
    fn parse_t3grammar_definition_with_missing_but_duplicated_parts_should_fail() {
        let mut s = "
    Sigma = { 'a', 'b' }
    V = { S, T, W }
    Sigma = { 'a', 'b' }
    S = S
    ";
        let r = parser::parse_t3grammar_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn produces_returns_false_for_impossible_word() {
        let mut s = "
    V = { S }
    Sigma = { 'a' }
    P = { S -> 'a' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(&mut s).unwrap();
        assert!(g.try_find_productions("ab").is_empty());
    }

    #[test]
    fn produces_works_for_single_terminal() {
        let mut s = "
    V = { S }
    Sigma = { 'a', 'b' }
    P = { S -> 'a', S -> 'b' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(&mut s).unwrap();
        let r = &g.try_find_productions("a")[0];
        assert_eq!(r, &vec![("S".into(), "a".into())]);
    }

    #[test]
    fn produces_works_for_2_terminals() {
        let mut s = "
    V = { S, T }
    Sigma = { 'a', 'b' }
    P = { S -> 'aT', T ->'b' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(&mut s).unwrap();
        let r = &g.try_find_productions("ab")[0];
        assert_eq!(
            r,
            &vec![("S".into(), "aT".into()), ("T".into(), "b".into())]
        );
    }

    #[test]
    fn produces_works_for_n_terminals() {
        let mut s = "
    V = { S, T }
    Sigma = { 'a', 'b', 'c' }
    P = { S -> 'aT', T -> 'b', T -> 'bT', T -> 'c' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(&mut s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        assert!(!g.try_find_productions("abbbbbc").is_empty());
    }

    #[test]
    fn produces_works_for_epsilon() {
        let mut s = "
    V = { S, T }
    Sigma = { 'a', 'b', 'c' }
    P = { S -> 'aT', T -> 'b', T -> 'bT', T -> '' }
    S = S
    ";
        let g = parser::parse_t3grammar_definition(&mut s).unwrap();
        assert!(!g.try_find_productions("abbbbb").is_empty());
        let r = &g.try_find_productions("a")[0];
        assert_eq!(r, &vec![("S".into(), "aT".into()), ("T".into(), "".into())]);
    }
}
