use ratatui::widgets::{ScrollbarState, TableState};
use ratatui_textarea::TextArea;
use strum::{Display, EnumCount, EnumIter, FromRepr};

#[derive(Debug, Default, PartialEq)]
pub(crate) struct InputState {
    pub(crate) value: String,
    pub(crate) cursor: usize,
}

#[derive(Debug, Default, Clone, Copy, Display, FromRepr, EnumIter, EnumCount)]
pub(crate) enum SelectedTopic {
    #[default]
    #[strum(to_string = "Set Theory")]
    SetTheory,
    #[strum(to_string = "Propositional Logic")]
    PropositionalLogic,
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

#[derive(Debug, Default)]
pub(crate) struct PropositionalLogicModel {
    pub(crate) formula_input_state: InputState,
    pub(crate) result: PropLogicResult,
    pub(crate) result_filter: Option<PropLogicResultFilter>,
    pub(crate) truth_table_state: TableState,
    pub(crate) truth_table_scroll_state: ScrollbarState,
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

#[derive(Debug)]
pub(crate) struct Model<'a> {
    pub(crate) running: bool,
    pub(crate) selected_topic: SelectedTopic,
    pub(crate) proplogic_state: PropositionalLogicModel,
    pub(crate) settheory_state: SetTheoryModel<'a>,
    pub(crate) show_help: bool,
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Self {
            running: true,
            selected_topic: SelectedTopic::default(),
            proplogic_state: Default::default(),
            settheory_state: Default::default(),
            show_help: Default::default(),
        }
    }
}
