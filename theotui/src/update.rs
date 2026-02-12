use crate::model::{Model, PropLogicOutput};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::ScrollbarState,
};
use std::collections::HashMap;
use theoinf::propositional_logic::{Assignment, run};
use tui_input::{Input, backend::crossterm::EventHandler};

pub(crate) enum Msg {
    Exit,
    Eval,
    ScrollUp,
    ScrollDown,
}

pub(crate) fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Msg>> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => Result::Ok(on_key_event(model, key)),
        _ => Result::Ok(None),
    }
}

fn on_key_event(model: &mut Model, key: KeyEvent) -> Option<Msg> {
    match key.code {
        KeyCode::Esc => Some(Msg::Exit),
        KeyCode::Enter => Some(Msg::Eval),
        KeyCode::Up => Some(Msg::ScrollUp),
        KeyCode::Down => Some(Msg::ScrollDown),
        _ => {
            let mut input = Input::new(model.formula_input_state.value.clone())
                .with_cursor(model.formula_input_state.cursor);
            input.handle_event(&Event::Key(key));
            model.formula_input_state.cursor = input.cursor();
            model.formula_input_state.value = input.value().into();
            None
        }
    }
}

pub(crate) fn update(model: &mut Model, msg: Msg) {
    match msg {
        Msg::Exit => {
            model.running = false;
        }
        Msg::Eval => {
            let tables =
                theoinf::propositional_logic::truth_table(model.formula_input_state.value.as_str());
            match tables {
                Ok(tables) if !tables.is_empty() => {
                    model.truth_table_state.select(Some(0));
                    model.truth_table_scroll_state = ScrollbarState::new(tables.len());
                    model.output = PropLogicOutput::Table(tables);
                }
                Ok(_) => {
                    let assignment: Assignment = HashMap::new();
                    let r = run(model.formula_input_state.value.as_str(), &assignment);
                    match r {
                        Ok(y) => model.output = PropLogicOutput::Literal(y.to_string()),
                        Err(e) => model.output = PropLogicOutput::Literal(e),
                    }
                }
                Err(e) => model.output = PropLogicOutput::Literal(e),
            }
        }
        Msg::ScrollUp => {
            if let Some(i) = match (&model.output, model.truth_table_state.selected()) {
                (PropLogicOutput::Table(_), Some(i)) => {
                    if i == 0 {
                        Some(i)
                    } else {
                        Some(i - 1)
                    }
                }
                _ => None,
            } {
                model.truth_table_state.select(Some(i));
                model.truth_table_scroll_state = model.truth_table_scroll_state.position(i);
            };
        }
        Msg::ScrollDown => {
            if let Some(i) = match (&model.output, model.truth_table_state.selected()) {
                (PropLogicOutput::Table(table), Some(i)) => {
                    if i >= table.len() - 1 {
                        Some(i)
                    } else {
                        Some(i + 1)
                    }
                }
                _ => None,
            } {
                model.truth_table_state.select(Some(i));
                model.truth_table_scroll_state = model.truth_table_scroll_state.position(i);
            }
        }
    }
}
