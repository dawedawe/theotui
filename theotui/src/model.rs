use ratatui::widgets::{ScrollbarState, TableState};

#[derive(Debug, Default, PartialEq)]
pub(crate) struct InputState {
    pub(crate) value: String,
    pub(crate) cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum PropLogicOutput {
    #[default]
    None,
    Literal(String),
    Table(theoinf::propositional_logic::TruthTable),
}

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) running: bool,
    pub(crate) formula_input_state: InputState,
    pub(crate) output: PropLogicOutput,
    pub(crate) truth_table_state: TableState,
    pub(crate) truth_table_scroll_state: ScrollbarState,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running: true,
            formula_input_state: Default::default(),
            output: Default::default(),
            truth_table_state: TableState::default(),
            truth_table_scroll_state: ScrollbarState::default(),
        }
    }
}
