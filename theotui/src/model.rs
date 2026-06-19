use ratatui::widgets::{ScrollbarState, TableState};
use ratatui_textarea::TextArea;
use strum::{Display, EnumCount, EnumIter, FromRepr};

#[derive(Debug, Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCount)]
pub(crate) enum SelectedTopic {
    #[default]
    #[strum(to_string = "Set Theory")]
    SetTheory,
    #[strum(to_string = "Propositional Logic")]
    PropositionalLogic,
    #[strum(to_string = "DFA")]
    Dfa,
    #[strum(to_string = "Type-3 Grammar")]
    T3Grammar,
    #[strum(to_string = "Type-2 Grammar")]
    T2Grammar,
}

impl SelectedTopic {
    pub(crate) fn previous(self) -> Self {
        let current_index = self as i32;
        let previous_index = (current_index - 1).rem_euclid(SelectedTopic::COUNT as i32);
        Self::from_repr(previous_index as usize).unwrap_or(self)
    }

    pub(crate) fn next(self) -> Self {
        let current_index = self as i32;
        let previous_index = (current_index + 1).rem_euclid(SelectedTopic::COUNT as i32);
        Self::from_repr(previous_index as usize).unwrap_or(self)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum PropLogicResult {
    #[default]
    None,
    Error(String),
    Literal(bool),
    Table(theoinf::propositional_logic::TruthTable),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PropLogicResultFilter {
    OnlyFalse,
    OnlyTrue,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum PropLogicFocus {
    #[default]
    Formula,
    Result,
    Ast,
    Cnf,
    Dnf,
}

#[derive(Debug, Default)]
pub(crate) struct PropositionalLogicModel<'a> {
    pub(crate) formula_textarea: TextArea<'a>,
    pub(crate) result: PropLogicResult,
    pub(crate) result_filter: Option<PropLogicResultFilter>,
    pub(crate) truth_table_state: TableState,
    pub(crate) truth_table_scroll_state: ScrollbarState,
    pub(crate) ast_textarea: TextArea<'a>,
    pub(crate) cnf_textarea: TextArea<'a>,
    pub(crate) dnf_textarea: TextArea<'a>,
    pub(crate) focus: PropLogicFocus,
    pub(crate) show_ast: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum SetTheoryResult {
    #[default]
    None,
    Error(String),
    Expr(theoinf::set_theory::Expr),
}

#[derive(Debug, Default)]
pub(crate) struct SetTheoryModel<'a> {
    pub(crate) term_textarea: TextArea<'a>,
    pub(crate) result: SetTheoryResult,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum DfaFocus {
    #[default]
    Definition,
    WordInput,
    Transitions,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum DfaResult {
    #[default]
    None,
    Error(String),
    Accepted(bool),
}

#[derive(Debug)]
pub(crate) struct DfaModel<'a> {
    pub(crate) definition_textarea: TextArea<'a>,
    pub(crate) input_word_textarea: TextArea<'a>,
    pub(crate) focus: DfaFocus,
    pub(crate) transitions: TextArea<'a>,
    pub(crate) result: DfaResult,
}

impl<'a> Default for DfaModel<'a> {
    fn default() -> Self {
        let mut default_def: TextArea<'a> = Default::default();
        default_def.insert_str("Sigma = { 'a', 'b' }\nS = { s0, s1, s2 }\nstart = s0\nF = { s2 }\ndelta = { (s0, 'a', s1), (s1, 'b', s2) }");
        Self {
            definition_textarea: default_def,
            input_word_textarea: Default::default(),
            focus: Default::default(),
            transitions: Default::default(),
            result: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum T3GrammarFocus {
    #[default]
    Definition,
    WordInput,
    Productions,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum T3GrammarResult {
    #[default]
    None,
    Error(String),
    Produced(usize),
}

#[derive(Debug)]
pub(crate) struct T3GrammarModel<'a> {
    pub(crate) definition_textarea: TextArea<'a>,
    pub(crate) input_word_textarea: TextArea<'a>,
    pub(crate) focus: T3GrammarFocus,
    pub(crate) productions: TextArea<'a>,
    pub(crate) result: T3GrammarResult,
}

impl<'a> Default for T3GrammarModel<'a> {
    fn default() -> Self {
        let mut default_def: TextArea<'a> = Default::default();
        default_def.insert_str("V = { S, T }\nSigma = { 'a', 'b' }\nP = { S -> 'aT', T -> 'b', T -> 'bT', T -> '' }\nS = S");
        Self {
            definition_textarea: default_def,
            input_word_textarea: Default::default(),
            focus: Default::default(),
            productions: Default::default(),
            result: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum T2GrammarFocus {
    #[default]
    Definition,
    WordInput,
    Productions,
    CykProductions,
    Cnf,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum T2GrammarResult {
    #[default]
    None,
    Error(String),
    Produced(bool),
}

#[derive(Debug)]
pub(crate) struct T2GrammarModel<'a> {
    pub(crate) definition_textarea: TextArea<'a>,
    pub(crate) input_word_textarea: TextArea<'a>,
    pub(crate) focus: T2GrammarFocus,
    pub(crate) productions: TextArea<'a>,
    pub(crate) cyk_productions: TextArea<'a>,
    pub(crate) cnf: TextArea<'a>,
    pub(crate) result: T2GrammarResult,
}

impl<'a> Default for T2GrammarModel<'a> {
    fn default() -> Self {
        let mut default_def: TextArea<'a> = Default::default();
        default_def
            .insert_str("V = { S }\nSigma = { '(', ')' }\nP = { S -> '(S)', S -> '()' }\nS = S");
        Self {
            definition_textarea: default_def,
            input_word_textarea: Default::default(),
            focus: Default::default(),
            productions: Default::default(),
            cyk_productions: Default::default(),
            cnf: Default::default(),
            result: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Focus {
    #[default]
    TopicList,
    TopicContent,
}

#[derive(Debug)]
pub(crate) struct Model<'a> {
    pub(crate) running: bool,
    pub(crate) selected_topic: SelectedTopic,
    pub(crate) focus: Focus,
    pub(crate) proplogic_state: PropositionalLogicModel<'a>,
    pub(crate) settheory_state: SetTheoryModel<'a>,
    pub(crate) dfa_state: DfaModel<'a>,
    pub(crate) t3grammar_state: T3GrammarModel<'a>,
    pub(crate) t2grammar_state: T2GrammarModel<'a>,
    pub(crate) show_help: bool,
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Self {
            running: true,
            selected_topic: SelectedTopic::default(),
            focus: Focus::default(),
            proplogic_state: Default::default(),
            settheory_state: Default::default(),
            dfa_state: Default::default(),
            t3grammar_state: Default::default(),
            t2grammar_state: Default::default(),
            show_help: Default::default(),
        }
    }
}
