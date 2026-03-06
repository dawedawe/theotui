use crate::model::{Model, PropLogicResult, PropLogicResultFilter, SelectedTopic, SetTheoryResult};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ScrollbarState,
};
use std::collections::HashMap;
use theoinf::propositional_logic::{Assignment, run};
use tui_input::{Input, backend::crossterm::EventHandler};

pub(crate) enum PropLogicMsg {
    Eval,
    FilterTrueRows,
    FilterFalseRows,
    ScrollUp,
    ScrollDown,
}

pub(crate) enum SetTheoryMsg {
    Eval,
}

pub(crate) enum Msg {
    Exit,
    NextTab,
    PrevTab,
    PropLogicMsg(PropLogicMsg),
    SetTheoryMsg(SetTheoryMsg),
}

pub(crate) fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Msg>> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => Result::Ok(on_key_event(model, key)),
        _ => Result::Ok(None),
    }
}

fn on_key_event(model: &mut Model, key: KeyEvent) -> Option<Msg> {
    match (model.selected_topic, key.code) {
        (_, KeyCode::Esc) => Some(Msg::Exit),
        (SelectedTopic::PropositionalLogic, KeyCode::Enter) => {
            Some(Msg::PropLogicMsg(PropLogicMsg::Eval))
        }
        (SelectedTopic::PropositionalLogic, KeyCode::Up) => {
            Some(Msg::PropLogicMsg(PropLogicMsg::ScrollUp))
        }
        (SelectedTopic::PropositionalLogic, KeyCode::Down) => {
            Some(Msg::PropLogicMsg(PropLogicMsg::ScrollDown))
        }
        (SelectedTopic::SetTheory, KeyCode::F(5)) => Some(Msg::SetTheoryMsg(SetTheoryMsg::Eval)),
        (_, KeyCode::Tab) => Some(Msg::NextTab),
        (_, KeyCode::BackTab) => Some(Msg::PrevTab),
        (SelectedTopic::PropositionalLogic, KeyCode::Char('f'))
            if key.modifiers.intersects(KeyModifiers::CONTROL) =>
        {
            Some(Msg::PropLogicMsg(PropLogicMsg::FilterFalseRows))
        }
        (SelectedTopic::PropositionalLogic, KeyCode::Char('t'))
            if key.modifiers.intersects(KeyModifiers::CONTROL) =>
        {
            Some(Msg::PropLogicMsg(PropLogicMsg::FilterTrueRows))
        }
        (SelectedTopic::PropositionalLogic, _) => {
            let mut tmp_input = Input::new(model.proplogic_state.formula_input_state.value.clone())
                .with_cursor(model.proplogic_state.formula_input_state.cursor);
            tmp_input.handle_event(&Event::Key(key));
            model.proplogic_state.formula_input_state.cursor = tmp_input.cursor();
            model.proplogic_state.formula_input_state.value = tmp_input.value().into();
            None
        }
        (SelectedTopic::SetTheory, _) => {
            model.settheory_state.term_textarea.input(key);
            None
        }
    }
}

pub(crate) fn update(model: &mut Model, msg: Msg) {
    match msg {
        Msg::Exit => {
            model.running = false;
        }
        Msg::PropLogicMsg(PropLogicMsg::FilterFalseRows) => {
            match model.proplogic_state.result_filter {
                Some(PropLogicResultFilter::OnlyFalse) => {
                    model.proplogic_state.result_filter = None
                }
                Some(PropLogicResultFilter::OnlyTrue) | None => {
                    model.proplogic_state.result_filter = Some(PropLogicResultFilter::OnlyFalse)
                }
            }
        }
        Msg::PropLogicMsg(PropLogicMsg::FilterTrueRows) => {
            match model.proplogic_state.result_filter {
                Some(PropLogicResultFilter::OnlyTrue) => model.proplogic_state.result_filter = None,
                Some(PropLogicResultFilter::OnlyFalse) | None => {
                    model.proplogic_state.result_filter = Some(PropLogicResultFilter::OnlyTrue)
                }
            }
        }
        Msg::PropLogicMsg(PropLogicMsg::Eval) => {
            let table = theoinf::propositional_logic::truth_table(
                model.proplogic_state.formula_input_state.value.as_str(),
            );
            match table {
                Ok(table) if !table.rows.is_empty() => {
                    model.proplogic_state.truth_table_state.select(Some(0));
                    model.proplogic_state.truth_table_scroll_state =
                        ScrollbarState::new(table.rows.len());
                    model.proplogic_state.result = PropLogicResult::Table(table);
                }
                Ok(_) => {
                    let assignment: Assignment = HashMap::new();
                    let r = run(
                        model.proplogic_state.formula_input_state.value.as_str(),
                        &assignment,
                    );
                    match r {
                        Ok(r) => model.proplogic_state.result = PropLogicResult::Literal(r),
                        Err(e) => model.proplogic_state.result = PropLogicResult::Error(e),
                    }
                }
                Err(e) => model.proplogic_state.result = PropLogicResult::Error(e),
            }
        }
        Msg::PropLogicMsg(PropLogicMsg::ScrollUp) => {
            if let Some(i) = match (
                &model.proplogic_state.result,
                model.proplogic_state.truth_table_state.selected(),
            ) {
                (PropLogicResult::Table(_), Some(i)) => {
                    if i == 0 {
                        Some(i)
                    } else {
                        Some(i - 1)
                    }
                }
                _ => None,
            } {
                model.proplogic_state.truth_table_state.select(Some(i));
                model.proplogic_state.truth_table_scroll_state =
                    model.proplogic_state.truth_table_scroll_state.position(i);
            };
        }
        Msg::PropLogicMsg(PropLogicMsg::ScrollDown) => {
            if let Some(i) = match (
                &model.proplogic_state.result,
                model.proplogic_state.truth_table_state.selected(),
            ) {
                (PropLogicResult::Table(table), Some(i)) => {
                    if i >= table.rows.len() - 1 {
                        Some(i)
                    } else {
                        Some(i + 1)
                    }
                }
                _ => None,
            } {
                model.proplogic_state.truth_table_state.select(Some(i));
                model.proplogic_state.truth_table_scroll_state =
                    model.proplogic_state.truth_table_scroll_state.position(i);
            }
        }
        Msg::SetTheoryMsg(SetTheoryMsg::Eval) => {
            let terms = model.settheory_state.term_textarea.lines().join("\n");
            let r = theoinf::set_theory::run(terms.as_str());
            match r {
                Ok(expr) => model.settheory_state.result = SetTheoryResult::Expr(expr),
                Err(e) => model.settheory_state.result = SetTheoryResult::Error(e),
            }
        }
        Msg::NextTab => model.selected_topic = model.selected_topic.next(),
        Msg::PrevTab => model.selected_topic = model.selected_topic.previous(),
    }
}
