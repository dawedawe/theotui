use std::collections::HashSet;

pub mod parser {
    use winnow::{
        ModalResult, Parser,
        ascii::{alphanumeric1, multispace0},
        combinator::{cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode},
        token::take_while,
    };

    fn whitespace_wrapped<'i>(s: &str) -> impl Parser<&'i str, &'i str, ErrMode<ContextError>> {
        trace("whitespace_wrapped", delimited(multispace0, s, multispace0))
    }

    /// Parses an alphabet definition like `A = { 'a', 'b', 'c' }`
    pub fn parse_alphabet_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
        let identifier = whitespace_wrapped("A");
        let equals = whitespace_wrapped("=");
        let symbol = take_while(1..=1, |_: char| -> bool { true });
        let element = delimited("'", symbol, cut_err("'"));
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

    /// Parses a state set like `{ s0, s1, s2 }`
    pub fn state_set<'s>() -> impl Parser<&'s str, Vec<&'s str>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., alphanumeric1, separator);
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
        let mut decl = (identifier, equals, state_set()).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    /// Parses a final states definition like `F = { s0, s1, s2 }`
    pub fn parse_final_states_definition<'s>(input: &'s mut &str) -> ModalResult<Vec<&'s str>> {
        let identifier = whitespace_wrapped("F");
        let equals = whitespace_wrapped("=");
        let mut decl = (identifier, equals, state_set()).map(|(_, _, x)| x);
        decl.parse_next(input)
    }

    /// Parse a start state definition like `start = s0`
    pub fn parse_start_state_definition<'s>(input: &'s mut &str) -> ModalResult<&'s str> {
        let identifier = whitespace_wrapped("start");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, alphanumeric1, multispace0);
        (identifier, equals, state)
            .map(|(_, _, x)| x)
            .parse_next(input)
    }

    /// Parses a transition like `(s0, 'a', s1)`
    pub fn transition<'s>()
    -> impl Parser<&'s str, (&'s str, &'s str, &'s str), ErrMode<ContextError>> {
        let symbol = take_while(1..=1, |_: char| -> bool { true });
        let element = delimited("'", symbol, cut_err("'"));
        let open_paren = whitespace_wrapped("(");
        let close_paren = whitespace_wrapped(")");
        let transition = (
            open_paren,
            alphanumeric1,
            whitespace_wrapped(","),
            element,
            whitespace_wrapped(","),
            alphanumeric1,
            close_paren,
        );
        trace(
            "transition",
            transition.map(|(_, s_in, _, sym, _, s_out, _)| (s_in, sym, s_out)),
        )
    }

    /// Parses a transitions set like `{ (s0, 'a', s1), (s1, 'b', s2) }`
    pub fn transitions_set<'s>()
    -> impl Parser<&'s str, Vec<(&'s str, &'s str, &'s str)>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(0.., transition(), separator);
        trace(
            "transitions_set",
            delimited(
                delimited(multispace0, "{", multispace0),
                comma_sep_list,
                delimited(multispace0, cut_err("}"), multispace0),
            ),
        )
    }

    /// Parse a transitions definition like `delta = { (s0, 'a', s1), (s1, 'b', s2) }`
    pub fn parse_transitions_definition<'s>(
        input: &'s mut &str,
    ) -> ModalResult<Vec<(&'s str, &'s str, &'s str)>> {
        let identifier = whitespace_wrapped("delta");
        let equals = whitespace_wrapped("=");
        (identifier, equals, transitions_set())
            .map(|(_, _, x)| x)
            .parse_next(input)
    }
}

type State = String;
type Symbol = char;

/// Defines a deterministic finite automata
pub struct Dfa {
    states: HashSet<State>,
    alphabet: HashSet<Symbol>,
    transitions: HashSet<(State, Symbol, State)>,
    final_states: HashSet<State>,
    start_state: State,
}

/*
A = { 'a', 'b', ... } // alphabet
S = { s0, s1, s2, ...} // states
start = s0 // start state
F = { s2, s3 } // final states
delta = { (s0, 'a', s1), (s1, 'b', s2), ... } // transitions
*/

impl Dfa {
    /// Constructs a valid [Dfa]
    pub fn new(
        states: HashSet<State>,
        alphabet: HashSet<Symbol>,
        transitions: HashSet<(State, Symbol, State)>,
        final_states: HashSet<State>,
        start_state: State,
    ) -> Result<Self, String> {
        if !states.contains(&start_state) {
            return Err("The start state must be contained in the states set.".into());
        }

        if !final_states.is_subset(&states) {
            return Err("The final states must be contained in the states set.".into());
        }

        let (mut unknown_transition_states, mut unknown_transition_symbols) =
            transitions.iter().fold(
                (vec![], vec![]),
                |(mut acc1, mut acc2), (s_in, sym, s_out)| {
                    if !states.contains(s_in) {
                        acc1.push(s_in.as_str());
                    }
                    if !states.contains(s_out) {
                        acc1.push(s_out.as_str());
                    }
                    if !alphabet.contains(sym) {
                        acc2.push(sym.to_string());
                    }
                    (acc1, acc2)
                },
            );
        if !unknown_transition_states.is_empty() {
            unknown_transition_states.sort();
            unknown_transition_states.dedup();
            let s = unknown_transition_states.join(", ");
            let msg = format!("The transition relation contains the following unknown states: {s}");
            return Err(msg);
        }

        if !unknown_transition_symbols.is_empty() {
            unknown_transition_symbols.sort();
            unknown_transition_symbols.dedup();
            let s = unknown_transition_symbols.join(", ");
            let msg =
                format!("The transition relation contains the following unknown symbols: {s}");
            return Err(msg);
        }

        let mut non_deterministic_transitions = transitions.iter().filter_map(|(s_in, sym, _)| {
            let trans = transitions.iter().filter(|(a, b, _)| (a, b) == (s_in, sym));
            if trans.count() == 1 {
                None
            } else {
                Some((s_in, sym))
            }
        });

        if non_deterministic_transitions.any(|_| true) {
            let mut non_deterministic_inputs: Vec<String> = non_deterministic_transitions
                .map(|(state, sym)| format!("({state}, {sym})"))
                .collect();
            non_deterministic_inputs.sort();
            non_deterministic_inputs.dedup();
            let s = non_deterministic_inputs.join(", ");
            let msg = format!(
                "The transition function is non-deterministic for the following inputs: {s}"
            );
            return Err(msg);
        }

        Ok(Dfa {
            states,
            alphabet,
            transitions,
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
    /// Tries to consume the next symbol.
    pub fn transition(&mut self) -> bool {
        match self.remaining_input.first() {
            None => false,
            Some(symbol) => {
                let t = self
                    .dfa
                    .transitions
                    .iter()
                    .find(|(curr, sym, _nxt)| curr == self.current_state && sym == symbol);
                if let Some((_curr, sym, nxt)) = t {
                    self.current_state = nxt;
                    self.accepted_input.push(*sym);
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
            HashSet::from([("s0".into(), 'a', "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        )
        .unwrap();
        let mut dfa_state = RunningDfa {
            dfa: &dfa,
            current_state: &dfa.start_state,
            remaining_input: vec!['a'],
            accepted_input: vec![],
        };
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
            HashSet::from([("s0".into(), 'a', "s1".into())]),
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
            HashSet::from([("s0".into(), 'a', "s1".into())]),
            HashSet::from(["s2".into(), "sX".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn transition_states_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashSet::from([("sX".into(), 'a', "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashSet::from([("s0".into(), 'a', "sX".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn transition_symbols_must_be_known() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashSet::from([("s0".into(), 'x', "s1".into())]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        assert!(dfa.is_err());
    }

    #[test]
    fn non_deterministic_transitions_cant_be_created() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashSet::from([
                ("s0".into(), 'a', "s1".into()),
                ("s0".into(), 'a', "s2".into()),
                ("s0".into(), 'b', "s1".into()),
                ("s0".into(), 'b', "s2".into()),
            ]),
            HashSet::from(["s2".into()]),
            "s0".into(),
        );
        match dfa {
            Err(s) => assert_eq!(
                s,
                "The transition function is non-deterministic for the following inputs: (s0, a), (s0, b)"
            ),
            _ => panic!("expected Err"),
        }
    }

    #[test]
    fn accepts_works() {
        let dfa = Dfa::new(
            HashSet::from(["s0".into(), "s1".into(), "s2".into()]),
            HashSet::from(['a', 'b']),
            HashSet::from([
                ("s0".into(), 'a', "s1".into()),
                ("s1".into(), 'b', "s2".into()),
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
    fn parse_alphabet_works() {
        let mut s = "A = { 'a' , 'b','c', ' ' } ";
        let symbols = parser::parse_alphabet_definition(&mut s).unwrap();
        assert_eq!(symbols, vec!["a", "b", "c", " "]);
    }

    #[test]
    fn parse_states_works() {
        let mut s = "S = { s0 , s1,s2  } ";
        let states = parser::parse_states_definition(&mut s).unwrap();
        assert_eq!(states, vec!["s0", "s1", "s2"]);
    }

    #[test]
    fn parse_final_states_works() {
        let mut s = "F = { s0 , s1  } ";
        let states = parser::parse_final_states_definition(&mut s).unwrap();
        assert_eq!(states, vec!["s0", "s1"]);
    }

    #[test]
    fn parse_start_state_works() {
        let mut s = "start = s0";
        let state = parser::parse_start_state_definition(&mut s).unwrap();
        assert_eq!(state, "s0");
    }

    // delta = { (s0, 'a', s1), (s1, 'b', s2), ... } // transitions
    #[test]
    fn parse_transitions_works() {
        let mut s = "delta = { (s0, 'a', s1), (s1, 'b', s2) }";
        let r = parser::parse_transitions_definition(&mut s).unwrap();
        assert_eq!(r, vec![("s0", "a", "s1"), ("s1", "b", "s2")]);
    }
}
