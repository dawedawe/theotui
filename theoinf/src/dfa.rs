use std::collections::{HashMap, HashSet};

pub mod parser {
    use std::collections::{HashMap, HashSet};

    use winnow::{
        ModalResult, Parser,
        ascii::multispace0,
        combinator::{alt, cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode, StrContext},
        token::{any, take_while},
    };

    use crate::dfa::{Dfa, State, Symbol};

    /// Expressions of the [Dfa] definition.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Expr {
        Sigma(Vec<Symbol>),
        States(Vec<State>),
        Start(State),
        FinalStates(Vec<State>),
        Delta(Vec<(String, Symbol, String)>),
    }

    fn whitespace_wrapped<'i>(s: &str) -> impl Parser<&'i str, &'i str, ErrMode<ContextError>> {
        trace("whitespace_wrapped", delimited(multispace0, s, multispace0))
    }

    /// Parses a Sigma definition like `Sigma = { 'a', 'b', 'c' }`
    pub fn parse_sigma_definition(input: &mut &str) -> ModalResult<Expr> {
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
        let mut decl = (identifier, equals, setp)
            .context(StrContext::Label("Sigma definition"))
            .map(|(_, _, x)| Expr::Sigma(x));
        decl.parse_next(input)
    }

    /// Parses a state name like `s0`
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
    pub fn parse_states_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("S");
        let equals = whitespace_wrapped("=");
        let mut decl = (identifier, equals, state_set(1))
            .context(StrContext::Label("S definition"))
            .map(|(_, _, x)| Expr::States(x.iter().map(|s| s.to_string()).collect()));
        decl.parse_next(input)
    }

    /// Parses a final states definition like `F = { s0, s1, s2 }`
    pub fn parse_final_states_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("F");
        let equals = whitespace_wrapped("=");
        let mut decl = (identifier, equals, state_set(0))
            .context(StrContext::Label("F definition"))
            .map(|(_, _, x)| Expr::FinalStates(x.iter().map(|s| s.to_string()).collect()));
        decl.parse_next(input)
    }

    /// Parse a start state definition like `start = s0`
    pub fn parse_start_state_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("start");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, state_name(), multispace0);
        (identifier, equals, state)
            .context(StrContext::Label("start definition"))
            .map(|(_, _, x)| Expr::Start(x.to_string()))
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
    pub fn parse_delta_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("delta");
        let equals = whitespace_wrapped("=");
        let r = (identifier, equals, delta_set())
            .context(StrContext::Label("delta definition"))
            .map(|(_, _, x): (&str, &str, Vec<(&str, char, &str)>)| {
                let x = x
                    .into_iter()
                    .map(|(a, b, c)| (a.to_string(), b, c.to_string()))
                    .collect();
                Expr::Delta(x)
            })
            .parse_next(input);
        if let Ok(Expr::Delta(delta)) = &r {
            let mut delta_inputs = HashSet::new();
            if delta
                .iter()
                .any(|(s_in, sym, _)| !delta_inputs.insert((s_in, sym)))
            {
                let mut context_error = ContextError::new();
                context_error.push(StrContext::Label(
                    "The delta definition is non-deterministic.",
                ));
                return Err(ErrMode::Cut(context_error));
            }
        }
        r
    }

    /// Parse a [Dfa] definition
    pub fn parse_dfa_definition(input: &str) -> Result<Dfa, String> {
        let mut input = input;
        let mut sigma: Option<HashSet<char>> = None;
        let mut states: Option<HashSet<String>> = None;
        let mut start: Option<String> = None;
        let mut final_states: Option<HashSet<String>> = None;
        let mut delta: Option<HashMap<(String, Symbol), String>> = None;

        let mut alt_parser = alt((
            parse_sigma_definition,
            parse_states_definition,
            parse_start_state_definition,
            parse_final_states_definition,
            parse_delta_definition,
        ));

        for _ in 0..5 {
            let r = alt_parser.parse_next(&mut input);

            match r {
                Ok(expr) => match expr {
                    Expr::Sigma(symbols) => sigma = Some(symbols.into_iter().collect()),
                    Expr::States(s) => states = Some(s.into_iter().collect()),
                    Expr::Start(s) => start = Some(s),
                    Expr::FinalStates(fs) => final_states = Some(fs.into_iter().collect()),
                    Expr::Delta(items) => {
                        delta = Some(items.into_iter().map(|(a, b, c)| ((a, b), c)).collect());
                    }
                },
                Err(s) => return Err(s.to_string()),
            }
        }

        if let (Some(sigma), Some(states), Some(start), Some(final_states), Some(delta)) =
            (sigma, states, start, final_states, delta)
        {
            Dfa::new(states, sigma, delta, final_states, start)
        } else {
            Err("Incomplete definition".into())
        }
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
            return Err("start must be an element of S.".into());
        }

        if !final_states.is_subset(&states) {
            return Err("F must be a subset of S.".into());
        }

        let (mut unknown_delta_states, mut unknown_delta_symbols) = delta.iter().fold(
            (vec![], vec![]),
            |(mut unknown_states, mut unknown_symbols), ((s_in, sym), s_out)| {
                if !states.contains(s_in) {
                    unknown_states.push(s_in.as_str());
                }
                if !states.contains(s_out) {
                    unknown_states.push(s_out.as_str());
                }
                if !sigma.contains(sym) {
                    unknown_symbols.push(sym.to_string());
                }
                (unknown_states, unknown_symbols)
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

    pub fn sigma(&self) -> &HashSet<Symbol> {
        &self.sigma
    }

    pub fn states(&self) -> &HashSet<State> {
        &self.states
    }

    pub fn start_state(&self) -> &State {
        &self.start_state
    }

    pub fn final_states(&self) -> &HashSet<State> {
        &self.final_states
    }

    pub fn delta(&self) -> &HashMap<(State, Symbol), State> {
        &self.delta
    }

    /// The [Dfa] accepts the given word.
    pub fn accepts(&self, word: &str) -> bool {
        let mut running_dfa = RunningDfa::new(self, word);
        while running_dfa.transition() {}
        running_dfa.accepted()
    }
}

/// Models a running [Dfa].
pub struct RunningDfa<'a> {
    pub(crate) dfa: &'a Dfa,
    pub(crate) current_state: &'a State,
    pub(crate) transitions: Vec<(Symbol, &'a State)>,
    pub(crate) remaining_input: Vec<Symbol>,
    pub(crate) accepted_input: Vec<Symbol>,
}

impl<'a> RunningDfa<'a> {
    /// Creates a [RunningDfa] in the [Dfa]'s start state and the word not yet processed
    pub fn new(dfa: &'a Dfa, word: &str) -> Self {
        Self {
            dfa,
            current_state: &dfa.start_state,
            transitions: vec![],
            remaining_input: word.to_string().chars().collect(),
            accepted_input: vec![],
        }
    }

    /// Tries to consume the next symbol.
    pub fn transition(&mut self) -> bool {
        match self.remaining_input.first() {
            None => false,
            Some(symbol) => {
                if let Some(next_state) = self.dfa.delta.get(&(self.current_state.clone(), *symbol))
                {
                    self.current_state = next_state;
                    self.transitions.push((*symbol, next_state));
                    self.accepted_input.push(*symbol);
                    self.remaining_input.remove(0);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// The [Dfa] accepts the word.
    pub fn accepts(&mut self) -> bool {
        while self.transition() {}
        self.accepted()
    }

    /// The [Dfa] is in a final state and the word has been fully consumed.
    pub fn accepted(&self) -> bool {
        self.dfa.final_states.contains(self.current_state) && self.remaining_input.is_empty()
    }

    /// The state transitions of the [Dfa] while processing the word.
    pub fn transitions(&self) -> &Vec<(char, &'a String)> {
        &self.transitions
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::parser::Expr;

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
        assert_eq!(symbols, Expr::Sigma(vec!['a', 'b', 'c', ' ']));
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
        assert_eq!(
            states,
            Expr::States(vec!["s0".into(), "s1".into(), "s2".into()])
        );
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
        assert_eq!(states, Expr::FinalStates(vec!["s_0".into(), "s1".into()]));
    }

    #[test]
    fn parse_empty_final_states_works() {
        let mut s = "F = {   } ";
        let states = parser::parse_final_states_definition(&mut s).unwrap();
        assert_eq!(states, Expr::FinalStates(vec![]));
    }

    #[test]
    fn parse_start_state_works() {
        let mut s = "start = s0";
        let state = parser::parse_start_state_definition(&mut s).unwrap();
        assert_eq!(state, Expr::Start("s0".into()));
    }

    #[test]
    fn parse_delta_works() {
        let mut s = "delta = { (s0, 'a', s1), (s1, 'b', s2) }";
        let r = parser::parse_delta_definition(&mut s).unwrap();
        assert_eq!(
            r,
            Expr::Delta(vec![
                ("s0".into(), 'a', "s1".into()),
                ("s1".into(), 'b', "s2".into())
            ])
        );
    }

    #[test]
    fn parse_dfa_definition_works() {
        let s = "
Sigma = { 'a', 'b'  }

S = { s0, s1, s2 }
start = s0
F = { s2  }
delta = { (s0, 'a', s1), (s1, 'b', s2) }
";
        let r = parser::parse_dfa_definition(s);
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

    #[test]
    fn parse_dfa_definition_with_missing_but_duplicated_parts_should_fail() {
        let s = "
Sigma = { 'a', 'b'  }
S = { s0, s1, s2 }
Sigma = { 'a', 'b'  }
F = { s2  }
delta = { (s0, 'a', s1), (s1, 'b', s2) }
";
        let r = parser::parse_dfa_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn runningdfa_transitions_works() {
        let s = "
Sigma = { 'a', 'b'  }
S = { s0, s1, s2 }
start = s0
F = { s2  }
delta = { (s0, 'a', s1),
          (s1, 'b', s2),
          (s2, 'b', s2) }
";
        let r = parser::parse_dfa_definition(s);
        assert!(r.is_ok());
        let dfa = r.unwrap();
        let mut running_dfa = RunningDfa::new(&dfa, "abb");
        assert!(running_dfa.accepts());
        assert_eq!(
            vec![
                ('a', &"s1".to_string()),
                ('b', &"s2".to_string()),
                ('b', &"s2".to_string())
            ],
            running_dfa.transitions
        );
    }
}
