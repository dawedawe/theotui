use std::collections::{HashMap, HashSet};

pub mod parser {
    use std::collections::{HashMap, HashSet};

    use winnow::{
        ModalResult, Parser,
        ascii::multispace0,
        combinator::{cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode, StrContext},
        token::{any, take_while},
    };

    use crate::dfa::{Dfa, Symbol};

    fn whitespace_wrapped<'i>(s: &str) -> impl Parser<&'i str, &'i str, ErrMode<ContextError>> {
        trace("whitespace_wrapped", delimited(multispace0, s, multispace0))
    }

    /// Parses a Sigma definition like `Sigma = { 'a', 'b', 'c' }`
    pub fn parse_sigma_definition(input: &mut &str) -> ModalResult<Vec<char>> {
        let identifier = whitespace_wrapped("Sigma");
        let equals = whitespace_wrapped("=");
        let element = delimited("'", any, cut_err("'"));
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

    pub fn state_name<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1.., |c: char| c.is_alphanumeric() || c == '_')
    }

    /// Parses a state set like `{ s0, s1, s2 }`
    pub fn state_set<'s>(min: usize) -> impl Parser<&'s str, Vec<&'s str>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(min.., state_name(), separator);
        trace(
            "state_set",
            delimited(
                delimited(multispace0, "{", multispace0),
                comma_sep_list,
                delimited(multispace0, cut_err("}"), multispace0),
            ),
        )
    }

    /// Parses a states definition like `S = { s0, s1, s2 }`
    pub fn parse_states_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
        let identifier = whitespace_wrapped("S");
        let equals = whitespace_wrapped("=");
        let mut decl = (identifier, equals, state_set(1)).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    /// Parses a final states definition like `F = { s0, s1, s2 }`
    pub fn parse_final_states_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
        let identifier = whitespace_wrapped("F");
        let equals = whitespace_wrapped("=");
        let mut decl = (identifier, equals, state_set(0)).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    /// Parse a start state definition like `start = s0`
    pub fn parse_start_state_definition<'s>(input: &'s mut &str) -> ModalResult<&'s str> {
        let identifier = whitespace_wrapped("start");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, state_name(), multispace0);
        (identifier, equals, state)
            .map(|(_, _, x)| x)
            .parse_next(input)
    }

    /// Parses a delta tuple like `(s0, 'a', s1)`
    pub fn delta_tuple<'s>() -> impl Parser<&'s str, (&'s str, char, &'s str), ErrMode<ContextError>>
    {
        let element = delimited("'", any, cut_err("'"));
        let open_paren = whitespace_wrapped("(");
        let close_paren = whitespace_wrapped(")");
        let tuple = (
            open_paren,
            state_name(),
            whitespace_wrapped(","),
            element,
            whitespace_wrapped(","),
            state_name(),
            close_paren,
        );
        trace(
            "delta_tuple",
            tuple.map(|(_, s_in, _, sym, _, s_out, _)| (s_in, sym, s_out)),
        )
    }

    /// Parses a delta set like `{ (s0, 'a', s1), (s1, 'b', s2) }`
    pub fn delta_set<'s>()
    -> impl Parser<&'s str, Vec<(&'s str, char, &'s str)>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(0.., delta_tuple(), separator);
        trace(
            "delta_set",
            delimited(
                delimited(multispace0, "{", multispace0),
                comma_sep_list,
                delimited(multispace0, cut_err("}"), multispace0),
            ),
        )
    }

    /// Parse a delta definition like `delta = { (s0, 'a', s1), (s1, 'b', s2) }`
    pub fn parse_delta_definition<'s>(
        input: &'s mut &str,
    ) -> ModalResult<Vec<(&'s str, char, &'s str)>> {
        let identifier = whitespace_wrapped("delta");
        let equals = whitespace_wrapped("=");
        let r = (identifier, equals, delta_set())
            .map(|(_, _, x)| x)
            .parse_next(input);
        match r {
            Ok(delta) => {
                let mut non_deterministic_delta = delta.iter().filter_map(|(s_in, sym, _)| {
                    let filtered = delta.iter().filter(|(a, b, _)| (a, b) == (s_in, sym));
                    if filtered.count() == 1 {
                        None
                    } else {
                        Some((s_in, sym))
                    }
                });

                if non_deterministic_delta.any(|_| true) {
                    let mut context_error = ContextError::new();

                    context_error.push(StrContext::Label(
                        "The delta definition is non-deterministic.",
                    ));
                    Err(ErrMode::Cut(context_error))
                } else {
                    Ok(delta)
                }
            }
            e => e,
        }
    }

    /// Parse a [Dfa] definition
    pub fn parse_dfa_definition(input: &mut &str) -> ModalResult<Dfa> {
        let lines: Vec<String> = input
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if lines.len() != 5 {
            return Err(ErrMode::Cut(ContextError::default()));
        }

        let mut sigma: Option<HashSet<char>> = None;
        let mut states: Option<HashSet<String>> = None;
        let mut start: Option<String> = None;
        let mut final_states: Option<HashSet<String>> = None;
        let mut delta: Option<HashMap<(String, Symbol), String>> = None;

        for line in lines {
            let mut line: &str = &line;
            if line.starts_with("Sigma") {
                let r = parse_sigma_definition(&mut line)?;
                sigma = Some(r.into_iter().collect());
            } else if line.starts_with("start") {
                let r = parse_start_state_definition(&mut line)?;
                start = Some(r.to_string());
            } else if line.starts_with("S") {
                let r = parse_states_definition(&mut line)?;
                states = Some(r.into_iter().map(|s| s.to_string()).collect());
            } else if line.starts_with("F") {
                let r = parse_final_states_definition(&mut line)?;
                final_states = Some(r.into_iter().map(|s| s.to_string()).collect());
            } else if line.starts_with("delta") {
                let r = parse_delta_definition(&mut line)?;
                delta = Some(
                    r.into_iter()
                        .map(|(s_in, sym, s_out)| ((s_in.to_string(), sym), s_out.to_string()))
                        .collect(),
                );
            } else {
                return Err(ErrMode::Cut(ContextError::default()));
            }
        }

        Dfa::new(
            states.expect("parsed S expected"),
            sigma.expect("parsed Sigma expected"),
            delta.expect("parsed delta expected"),
            final_states.expect("parsed F expected"),
            start.expect("parsed start expected"),
        )
        .map_err(|_| ErrMode::Cut(ContextError::default()))
    }
}

type State = String;
type Symbol = char;

/// Defines a deterministic finite automata
pub struct Dfa {
    pub(crate) states: HashSet<State>,
    pub(crate) sigma: HashSet<Symbol>,
    pub(crate) delta: HashMap<(State, Symbol), State>,
    pub(crate) final_states: HashSet<State>,
    pub(crate) start_state: State,
}

impl Dfa {
    /// Constructs a valid [Dfa]
    pub fn new(
        states: HashSet<State>,
        sigma: HashSet<Symbol>,
        delta: HashMap<(State, Symbol), State>,
        final_states: HashSet<State>,
        start_state: State,
    ) -> Result<Self, String> {
        if !states.contains(&start_state) {
            return Err("The start state must be contained in the states set.".into());
        }

        if !final_states.is_subset(&states) {
            return Err("The final states must be contained in the states set.".into());
        }

        let (mut unknown_delta_states, mut unknown_delta_symbols) = delta.iter().fold(
            (vec![], vec![]),
            |(mut acc1, mut acc2), ((s_in, sym), s_out)| {
                if !states.contains(s_in) {
                    acc1.push(s_in.as_str());
                }
                if !states.contains(s_out) {
                    acc1.push(s_out.as_str());
                }
                if !sigma.contains(sym) {
                    acc2.push(sym.to_string());
                }
                (acc1, acc2)
            },
        );
        if !unknown_delta_states.is_empty() {
            unknown_delta_states.sort();
            unknown_delta_states.dedup();
            let s = unknown_delta_states.join(", ");
            let msg = format!("The delta relation contains the following unknown states: {s}");
            return Err(msg);
        }

        if !unknown_delta_symbols.is_empty() {
            unknown_delta_symbols.sort();
            unknown_delta_symbols.dedup();
            let s = unknown_delta_symbols.join(", ");
            let msg = format!("The delta relation contains the following unknown symbols: {s}");
            return Err(msg);
        }

        Ok(Dfa {
            states,
            sigma,
            delta,
            final_states,
            start_state,
        })
    }

    /// The [Dfa] accepts the given word.
    pub fn accepts(&self, word: &str) -> bool {
        let mut running_dfa = RunningDfa {
            dfa: self,
            current_state: &self.start_state,
            remaining_input: word.chars().collect(),
            accepted_input: vec![],
        };
        while running_dfa.transition() {}
        running_dfa.accepts()
    }
}

/// Models a running [Dfa].
pub struct RunningDfa<'a> {
    pub dfa: &'a Dfa,
    pub current_state: &'a State,
    pub remaining_input: Vec<Symbol>,
    pub accepted_input: Vec<Symbol>,
}

impl<'a> RunningDfa<'a> {
    /// Creates a [RunningDfa] in the [Dfa]'s start state and the word not yet processed
    pub fn new(dfa: &'a Dfa, word: &str) -> Self {
        Self {
            dfa,
            current_state: &dfa.start_state,
            remaining_input: word.to_string().chars().collect(),
            accepted_input: vec![],
        }
    }

    /// Tries to consume the next symbol.
    pub fn transition(&mut self) -> bool {
        match self.remaining_input.first() {
            None => false,
            Some(symbol) => {
                let next_state = self.dfa.delta.get(&(self.current_state.clone(), *symbol));
                if let Some(next_state) = next_state {
                    self.current_state = next_state;
                    self.accepted_input.push(*symbol);
                    self.remaining_input.remove(0);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// The [Dfa] is in a final state and the word has been fully consumed.
    pub fn accepts(&self) -> bool {
        self.dfa.final_states.contains(self.current_state) && self.remaining_input.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_possible_transition_works() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("s0".into(), 'a'), "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        )
        .unwrap();
        let mut dfa_state = RunningDfa::new(&dfa, "a");
        assert!(dfa_state.transition());
        assert_eq!(dfa_state.current_state, "s1");
        assert_eq!(dfa_state.accepted_input, vec!['a']);
        assert!(dfa_state.remaining_input.is_empty());
    }

    #[test]
    fn start_state_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("s0".into(), 'a'), "s1".into())]),
            HashSet::from(["s2".into()]),
            "sX".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn final_states_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("s0".into(), 'a'), "s1".into())]),
            HashSet::from(["s2".into(), "sX".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn delta_states_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("sX".into(), 'a'), "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("s0".into(), 'a'), "sX".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn delta_symbols_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([(("s0".into(), 'x'), "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn parsing_non_deterministic_delta_should_fail() {
        let mut s = "delta = { (s0, 'a', s1), (s0, 'a', s2), (s0, 'b', s1), (s0, 'b', s2) }";
        let r = parser::parse_delta_definition(&mut s);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("non-deterministic"));
    }

    #[test]
    fn accepts_works() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashMap::from([
                (("s0".into(), 'a'), "s1".into()),
                (("s1".into(), 'b'), "s2".into()),
            ]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        )
        .unwrap();
        assert!(dfa.accepts("ab"));
        assert!(!dfa.accepts("abb"));
        assert!(!dfa.accepts("aa"));
        assert!(!dfa.accepts("a"));
        assert!(!dfa.accepts("x"));
        assert!(!dfa.accepts(""));
    }

    #[test]
    fn parse_sigma_works() {
        let mut s = "Sigma = { 'a' , 'b','c', ' ' } ";
        let symbols = parser::parse_sigma_definition(&mut s).unwrap();
        assert_eq!(symbols, vec!['a', 'b', 'c', ' ']);
    }

    #[test]
    fn parse_empty_sigma_should_fail() {
        let mut s = "Sigma = { } ";
        let r = parser::parse_sigma_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_states_works() {
        let mut s = "S = { s0 , s1,s2  } ";
        let states = parser::parse_states_definition(&mut s).unwrap();
        assert_eq!(states, vec!["s0", "s1", "s2"]);
    }

    #[test]
    fn parse_empty_states_should_fail() {
        let mut s = "S = {} ";
        let r = parser::parse_states_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_final_states_works() {
        let mut s = "F = { s_0 , s1  } ";
        let states = parser::parse_final_states_definition(&mut s).unwrap();
        assert_eq!(states, vec!["s_0", "s1"]);
    }

    #[test]
    fn parse_empty_final_states_should_fail() {
        let mut s = "F = {   } ";
        let states = parser::parse_final_states_definition(&mut s).unwrap();
        assert!(states.is_empty());
    }

    #[test]
    fn parse_start_state_works() {
        let mut s = "start = s0";
        let state = parser::parse_start_state_definition(&mut s).unwrap();
        assert_eq!(state, "s0");
    }

    #[test]
    fn parse_delta_works() {
        let mut s = "delta = { (s0, 'a', s1), (s1, 'b', s2) }";
        let r = parser::parse_delta_definition(&mut s).unwrap();
        assert_eq!(r, vec![("s0", 'a', "s1"), ("s1", 'b', "s2")]);
    }

    #[test]
    fn parse_dfa_definition_works() {
        let mut s = "
Sigma = { 'a', 'b'  }

S = { s0, s1, s2 }
start = s0
F = { s2  }
delta = { (s0, 'a', s1), (s1, 'b', s2) }
";
        let r = parser::parse_dfa_definition(&mut s);
        assert!(r.is_ok());
        let dfa = r.unwrap();
        assert_eq!(
            dfa.states,
            HashSet::from_iter(["s0".into(), "s1".into(), "s2".into()])
        );
        assert_eq!(dfa.sigma, HashSet::from_iter(['a', 'b']));
        assert_eq!(dfa.start_state, "s0");
        assert_eq!(dfa.final_states, HashSet::from_iter(["s2".into()]));
        assert_eq!(
            dfa.delta,
            HashMap::from_iter([
                (("s0".into(), 'a'), "s1".into()),
                (("s1".into(), 'b'), "s2".into())
            ])
        );
        assert!(dfa.accepts("ab"));
        assert!(!dfa.accepts("abc"));
    }
}
