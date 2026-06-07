use crate::model::{
    DfaFocus, DfaResult, Focus, Model, PropLogicResult, PropLogicResultFilter, SelectedTopic,
    SetTheoryResult, T2GrammarFocus, T2GrammarResult, T3GrammarFocus, T3GrammarResult,
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ScrollbarState,
};
use ratatui_textarea::{CursorMove, TextArea};
use std::{collections::HashMap, ops::Deref};
use theoinf::{
    dfa::{self, RunningDfa},
    propositional_logic::{Assignment, run},
    type2grammar::{self},
    type3grammar,
};

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

pub(crate) enum DfaMsg {
    Eval,
}

pub(crate) enum T3GrammarMsg {
    Eval,
}

pub(crate) enum T2GrammarMsg {
    Eval,
}

pub(crate) enum Msg {
    Exit,
    NextTab,
    PrevTab,
    NextFocus,
    PrevFocus,
    PropLogicMsg(PropLogicMsg),
    SetTheoryMsg(SetTheoryMsg),
    DfaMsg(DfaMsg),
    T3GrammarMsg(T3GrammarMsg),
    T2GrammarMsg(T2GrammarMsg),
    ToggleHelp,
}

pub(crate) fn handle_event(model: &mut Model) -> color_eyre::Result<Option<Msg>> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => Result::Ok(on_key_event(model, key)),
        _ => Result::Ok(None),
    }
}

fn is_nav_keycode(key_event: KeyEvent) -> bool {
    matches!(
        key_event.code,
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End
    )
}

fn on_key_event(model: &mut Model, key: KeyEvent) -> Option<Msg> {
    match (model.selected_topic, key.code) {
        (_, KeyCode::Esc) => Some(Msg::Exit),
        (_, KeyCode::F(1)) => Some(Msg::ToggleHelp),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) if model.focus == Focus::TopicList => {
            Some(Msg::NextTab)
        }
        (_, KeyCode::Up) | (_, KeyCode::Char('k')) if model.focus == Focus::TopicList => {
            Some(Msg::PrevTab)
        }
        (_, KeyCode::Tab) => Some(Msg::NextFocus),
        (_, KeyCode::BackTab) => Some(Msg::PrevFocus),
        (SelectedTopic::PropositionalLogic, KeyCode::Enter)
        | (SelectedTopic::PropositionalLogic, KeyCode::F(5)) => {
            Some(Msg::PropLogicMsg(PropLogicMsg::Eval))
        }
        (SelectedTopic::PropositionalLogic, KeyCode::Up) if model.focus == Focus::TopicContent => {
            Some(Msg::PropLogicMsg(PropLogicMsg::ScrollUp))
        }
        (SelectedTopic::PropositionalLogic, KeyCode::Down)
            if model.focus == Focus::TopicContent =>
        {
            Some(Msg::PropLogicMsg(PropLogicMsg::ScrollDown))
        }
        (SelectedTopic::SetTheory, KeyCode::F(5)) => Some(Msg::SetTheoryMsg(SetTheoryMsg::Eval)),
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
        (SelectedTopic::PropositionalLogic, _) if model.focus == Focus::TopicContent => {
            model.proplogic_state.formula_textarea.input(key);
            None
        }
        (SelectedTopic::SetTheory, _) if model.focus == Focus::TopicContent => {
            model.settheory_state.term_textarea.input(key);
            None
        }
        (SelectedTopic::Dfa, KeyCode::F(5)) => Some(Msg::DfaMsg(DfaMsg::Eval)),
        (SelectedTopic::Dfa, _) if model.focus == Focus::TopicContent => {
            match model.dfa_state.focus {
                crate::model::DfaFocus::Definition => {
                    model.dfa_state.definition_textarea.input(key);
                    None
                }
                crate::model::DfaFocus::WordInput => {
                    if key.code == KeyCode::Enter {
                        Some(Msg::DfaMsg(DfaMsg::Eval))
                    } else {
                        model.dfa_state.input_word_textarea.input(key);
                        None
                    }
                }
                crate::model::DfaFocus::Transitions => {
                    if key.code == KeyCode::Up || key.code == KeyCode::Down {
                        model.dfa_state.transitions.input(key);
                    }
                    None
                }
            }
        }
        (SelectedTopic::T3Grammar, KeyCode::F(5)) => Some(Msg::T3GrammarMsg(T3GrammarMsg::Eval)),
        (SelectedTopic::T3Grammar, _) if model.focus == Focus::TopicContent => {
            match model.t3grammar_state.focus {
                crate::model::T3GrammarFocus::Definition => {
                    model.t3grammar_state.definition_textarea.input(key);
                    None
                }
                crate::model::T3GrammarFocus::WordInput => {
                    if key.code == KeyCode::Enter {
                        Some(Msg::T3GrammarMsg(T3GrammarMsg::Eval))
                    } else {
                        model.t3grammar_state.input_word_textarea.input(key);
                        None
                    }
                }
                crate::model::T3GrammarFocus::Productions => {
                    if is_nav_keycode(key) {
                        model.t3grammar_state.productions.input(key);
                    }
                    None
                }
            }
        }
        (SelectedTopic::T2Grammar, KeyCode::F(5)) => Some(Msg::T2GrammarMsg(T2GrammarMsg::Eval)),
        (SelectedTopic::T2Grammar, _) if model.focus == Focus::TopicContent => {
            match model.t2grammar_state.focus {
                crate::model::T2GrammarFocus::Definition => {
                    model.t2grammar_state.definition_textarea.input(key);
                    None
                }
                crate::model::T2GrammarFocus::WordInput => {
                    if key.code == KeyCode::Enter {
                        Some(Msg::T2GrammarMsg(T2GrammarMsg::Eval))
                    } else {
                        model.t2grammar_state.input_word_textarea.input(key);
                        None
                    }
                }
                crate::model::T2GrammarFocus::Productions => {
                    if is_nav_keycode(key) {
                        model.t2grammar_state.productions.input(key);
                    }
                    None
                }
                crate::model::T2GrammarFocus::Cnf => {
                    if is_nav_keycode(key) {
                        model.t2grammar_state.cnf.input(key);
                    }
                    None
                }
            }
        }
        _ => None,
    }
}

fn set_textarea(productions: &mut TextArea, content: String) {
    productions.select_all();
    productions.cut();
    productions.insert_str(content);
    productions.move_cursor(CursorMove::Top);
    productions.move_cursor(CursorMove::End);
}

pub(crate) fn update(model: &mut Model, msg: Msg) {
    match msg {
        Msg::Exit => {
            model.running = false;
        }
        Msg::NextTab => model.selected_topic = model.selected_topic.next(),
        Msg::PrevTab => model.selected_topic = model.selected_topic.previous(),
        Msg::ToggleHelp => model.show_help = !model.show_help,
        Msg::NextFocus => match model.selected_topic {
            SelectedTopic::SetTheory => match model.focus {
                Focus::TopicList => model.focus = Focus::TopicContent,
                Focus::TopicContent => model.focus = Focus::TopicList,
            },
            SelectedTopic::PropositionalLogic => match model.focus {
                Focus::TopicList => model.focus = Focus::TopicContent,
                Focus::TopicContent => model.focus = Focus::TopicList,
            },
            SelectedTopic::Dfa => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.dfa_state.focus = DfaFocus::Definition
                }
                Focus::TopicContent => match model.dfa_state.focus {
                    DfaFocus::Definition => model.dfa_state.focus = DfaFocus::WordInput,
                    DfaFocus::WordInput => model.dfa_state.focus = DfaFocus::Transitions,
                    DfaFocus::Transitions => model.focus = Focus::TopicList,
                },
            },
            SelectedTopic::T3Grammar => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.t3grammar_state.focus = T3GrammarFocus::Definition
                }
                Focus::TopicContent => match model.t3grammar_state.focus {
                    T3GrammarFocus::Definition => {
                        model.t3grammar_state.focus = T3GrammarFocus::WordInput
                    }
                    T3GrammarFocus::WordInput => {
                        model.t3grammar_state.focus = T3GrammarFocus::Productions
                    }
                    T3GrammarFocus::Productions => model.focus = Focus::TopicList,
                },
            },
            SelectedTopic::T2Grammar => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.t2grammar_state.focus = T2GrammarFocus::Definition
                }
                Focus::TopicContent => match model.t2grammar_state.focus {
                    T2GrammarFocus::Definition => {
                        model.t2grammar_state.focus = T2GrammarFocus::WordInput
                    }
                    T2GrammarFocus::WordInput => {
                        model.t2grammar_state.focus = T2GrammarFocus::Productions
                    }
                    T2GrammarFocus::Productions => {
                        model.t2grammar_state.focus = T2GrammarFocus::Cnf
                    }
                    T2GrammarFocus::Cnf => model.focus = Focus::TopicList,
                },
            },
        },
        Msg::PrevFocus => match model.selected_topic {
            SelectedTopic::SetTheory => match model.focus {
                Focus::TopicList => model.focus = Focus::TopicContent,
                Focus::TopicContent => model.focus = Focus::TopicList,
            },
            SelectedTopic::PropositionalLogic => match model.focus {
                Focus::TopicList => model.focus = Focus::TopicContent,
                Focus::TopicContent => model.focus = Focus::TopicList,
            },
            SelectedTopic::Dfa => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.dfa_state.focus = DfaFocus::Transitions
                }
                Focus::TopicContent => match model.dfa_state.focus {
                    DfaFocus::Definition => model.focus = Focus::TopicList,
                    DfaFocus::WordInput => model.dfa_state.focus = DfaFocus::Definition,
                    DfaFocus::Transitions => model.dfa_state.focus = DfaFocus::WordInput,
                },
            },
            SelectedTopic::T3Grammar => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.t3grammar_state.focus = T3GrammarFocus::Productions
                }
                Focus::TopicContent => match model.t3grammar_state.focus {
                    T3GrammarFocus::Definition => model.focus = Focus::TopicList,
                    T3GrammarFocus::WordInput => {
                        model.t3grammar_state.focus = T3GrammarFocus::Definition
                    }
                    T3GrammarFocus::Productions => {
                        model.t3grammar_state.focus = T3GrammarFocus::WordInput
                    }
                },
            },
            SelectedTopic::T2Grammar => match model.focus {
                Focus::TopicList => {
                    model.focus = Focus::TopicContent;
                    model.t2grammar_state.focus = T2GrammarFocus::Cnf
                }
                Focus::TopicContent => match model.t2grammar_state.focus {
                    T2GrammarFocus::Definition => model.focus = Focus::TopicList,
                    T2GrammarFocus::WordInput => {
                        model.t2grammar_state.focus = T2GrammarFocus::Definition
                    }
                    T2GrammarFocus::Productions => {
                        model.t2grammar_state.focus = T2GrammarFocus::WordInput
                    }
                    T2GrammarFocus::Cnf => {
                        model.t2grammar_state.focus = T2GrammarFocus::Productions
                    }
                },
            },
        },
        Msg::PropLogicMsg(PropLogicMsg::FilterFalseRows) => {
            model.proplogic_state.result_filter = match model.proplogic_state.result_filter {
                Some(PropLogicResultFilter::OnlyFalse) => None,
                Some(PropLogicResultFilter::OnlyTrue) | None => {
                    Some(PropLogicResultFilter::OnlyFalse)
                }
            }
        }
        Msg::PropLogicMsg(PropLogicMsg::FilterTrueRows) => {
            model.proplogic_state.result_filter = match model.proplogic_state.result_filter {
                Some(PropLogicResultFilter::OnlyTrue) => None,
                Some(PropLogicResultFilter::OnlyFalse) | None => {
                    Some(PropLogicResultFilter::OnlyTrue)
                }
            }
        }
        Msg::PropLogicMsg(PropLogicMsg::Eval) => {
            model.proplogic_state.result_filter = None;
            let formula = model.proplogic_state.formula_textarea.lines();
            let formula = formula.first().map(|w| w.deref()).unwrap_or("");
            let table = theoinf::propositional_logic::truth_table(formula);
            match table {
                Ok(table) if !table.rows.is_empty() => {
                    model.proplogic_state.truth_table_state.select(Some(0));
                    model.proplogic_state.truth_table_scroll_state =
                        ScrollbarState::new(table.rows.len());
                    model.proplogic_state.result = PropLogicResult::Table(table);
                }
                Ok(_) => {
                    let assignment: Assignment = HashMap::new();
                    let r = run(formula, &assignment);
                    model.proplogic_state.result = match r {
                        Ok(r) => PropLogicResult::Literal(r),
                        Err(e) => PropLogicResult::Error(e),
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
            model.settheory_state.result = match r {
                Ok(expr) => SetTheoryResult::Expr(expr),
                Err(e) => SetTheoryResult::Error(e),
            }
        }
        Msg::DfaMsg(DfaMsg::Eval) => {
            let def = model.dfa_state.definition_textarea.lines().join("\n");
            let dfa = dfa::parser::parse_dfa_definition(def.as_str());

            let (result, transitions) = match dfa {
                Ok(dfa) => {
                    let word = model.dfa_state.input_word_textarea.lines();
                    let word = word.first().map(|w| w.deref()).unwrap_or("");
                    let mut running_dfa = RunningDfa::new(&dfa, word);
                    let accepts = running_dfa.accepts();
                    let transitions = running_dfa
                        .transitions()
                        .iter()
                        .map(|(sym, state)| format!("({}, {})", sym, state))
                        .collect::<Vec<String>>()
                        .join(" ->\n");
                    (DfaResult::Accepted(accepts), transitions)
                }
                Err(e) => (DfaResult::Error(e), "".to_string()),
            };
            model.dfa_state.result = result;
            model.dfa_state.transitions.select_all();
            model.dfa_state.transitions.cut();
            model.dfa_state.transitions.insert_str(transitions);
            model.dfa_state.transitions.move_cursor(CursorMove::Top);
            model.dfa_state.transitions.move_cursor(CursorMove::End);
        }
        Msg::T3GrammarMsg(T3GrammarMsg::Eval) => {
            let def = model.t3grammar_state.definition_textarea.lines().join("\n");
            let g = type3grammar::parser::parse_t3grammar_definition(def.as_str());

            let (result, transitions) = match g {
                Ok(g) => {
                    let word = model.t3grammar_state.input_word_textarea.lines();
                    let word = word.first().map(|w| w.deref()).unwrap_or("");
                    let productions = g.try_find_productions(word);
                    let chain = productions
                        .iter()
                        .map(|chain| {
                            chain
                                .iter()
                                .map(|(lhs, rhs)| format!("{lhs} -> '{rhs}'"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .collect::<Vec<String>>()
                        .join("\n-----\n");
                    (T3GrammarResult::Produced(productions.len()), chain)
                }
                Err(e) => (T3GrammarResult::Error(e), "".into()),
            };
            model.t3grammar_state.result = result;
            model.t3grammar_state.productions.select_all();
            model.t3grammar_state.productions.cut();
            model.t3grammar_state.productions.insert_str(transitions);
            model
                .t3grammar_state
                .productions
                .move_cursor(CursorMove::Top);
            model
                .t3grammar_state
                .productions
                .move_cursor(CursorMove::End);
        }
        Msg::T2GrammarMsg(T2GrammarMsg::Eval) => {
            let def = model.t2grammar_state.definition_textarea.lines().join("\n");
            let g = type2grammar::parser::parse_t2grammar_definition(def.as_str());

            let (result, transitions, cnf) = match g {
                Ok(g) => {
                    let word = model.t2grammar_state.input_word_textarea.lines();
                    let word = word.first().map(|w| w.deref()).unwrap_or("");
                    let productions = g.try_find_productions(word);
                    let has_produced = productions.is_some();
                    let chain = productions
                        .map(|chain: Vec<(String, Vec<String>)>| {
                            chain
                                .iter()
                                .map(|(lhs, rhs)| {
                                    let rhs = rhs.join("");
                                    format!("{lhs} -> '{rhs}'")
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or("".into());
                    let cnf = g.to_cnf().to_string();
                    (T2GrammarResult::Produced(has_produced), chain, cnf)
                }
                Err(e) => (T2GrammarResult::Error(e), "".into(), "".into()),
            };
            model.t2grammar_state.result = result;
            set_textarea(&mut model.t2grammar_state.productions, transitions);
            set_textarea(&mut model.t2grammar_state.cnf, cnf);
        }
    }
}
