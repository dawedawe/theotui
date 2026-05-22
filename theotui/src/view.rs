use crate::model::{
    DfaFocus, DfaResult, Focus, Model, PropLogicResultFilter, SelectedTopic, T2GrammarFocus,
    T2GrammarResult, T3GrammarFocus, T3GrammarResult,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, List, ListState, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
};
use strum::IntoEnumIterator;
use tui_input::Input;

fn default_style() -> Style {
    Style::default().fg(Color::Green)
}
pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    let default_style = default_style();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Length(25), // topic selection
                Constraint::Min(1),     // topic content
            ]
            .as_ref(),
        )
        .split(frame.area());

    let topics_rect = chunks[0];
    let topics_content_rect = chunks[1];

    // render topic list
    let items = SelectedTopic::iter().map(|t| t.to_string());
    let highlight_style = if model.focus == Focus::TopicList {
        default_style.bold().bg(Color::Green).fg(Color::Black)
    } else {
        default_style.bold()
    };
    let selected_tab_index = model.selected_topic as usize;
    let topic_list = List::new(items)
        .style(default_style)
        .highlight_style(highlight_style.bold())
        .highlight_symbol("> ")
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
    let mut topic_list_state = ListState::default().with_selected(Some(selected_tab_index));

    frame.render_stateful_widget(topic_list, topics_rect, &mut topic_list_state);

    match model.selected_topic {
        SelectedTopic::SetTheory => render_settheory(frame, topics_content_rect, model),
        SelectedTopic::PropositionalLogic => render_proplogic(frame, topics_content_rect, model),
        SelectedTopic::Dfa => render_dfa(frame, topics_content_rect, model),
        SelectedTopic::T3Grammar => render_t3grammar(frame, topics_content_rect, model),
        SelectedTopic::T2Grammar => render_t2grammar(frame, topics_content_rect, model),
    }
}

fn render_settheory(frame: &mut Frame, rect: Rect, model: &mut Model) {
    let default_style = default_style();

    let main_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // term, result
                Constraint::Length(1), // key bindings
            ]
            .as_ref(),
        )
        .split(rect);

    let key_bindings_rect = main_vert_split[1];
    let (non_help_rect, help_rect) = if model.show_help {
        let halfs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_vert_split[0]);
        (halfs[0], halfs[1])
    } else {
        (main_vert_split[0], Rect::default())
    };

    let sub_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(3),    // term input
                Constraint::Length(3), // result
            ]
            .as_ref(),
        )
        .split(non_help_rect);

    let term_rect = sub_vert_split[0];
    let result_rect = sub_vert_split[1];

    model
        .settheory_state
        .term_textarea
        .set_cursor_line_style(default_style);

    let editor_block = Block::default().borders(Borders::ALL).style(default_style);
    let editor_block = if model.focus == Focus::TopicContent {
        editor_block
            .title(" Term* ")
            .title_style(default_style.bold())
    } else {
        editor_block.title(" Term ")
    };
    model.settheory_state.term_textarea.set_block(editor_block);
    frame.render_widget(&model.settheory_state.term_textarea, term_rect);

    // render eval result
    match &model.settheory_state.result {
        &crate::model::SetTheoryResult::None => (),
        crate::model::SetTheoryResult::Error(e) => {
            let result_paragraph = Paragraph::new(e.as_str())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(result_paragraph, result_rect);
        }
        crate::model::SetTheoryResult::Expr(eval_result) => {
            let result_paragraph = Paragraph::new(eval_result.to_string())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(result_paragraph, result_rect);
        }
    }

    // render help if toggled
    if model.show_help {
        let help = "A = {1,2,3}       // declare a set
UNI = {1,2,3,4,5} // declare the UNIVERSE set
A u B             // union
A n B             // intersection
A \\ B             // difference
A x B             // cartesian product
A c B             // strict subset
A c= B            // subset
A == B            // equality
!A                // complement, needs UNI
|A|               // cardinality";
        let help_paragraph = Paragraph::new(help)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_paragraph, help_rect);
    }

    // render key bindings
    let key_bindings = vec![
        Span::raw("Switch focus: "),
        Span::styled("Tab | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Evaluate: "),
        Span::styled("F5 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Help: "),
        Span::styled("F1 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Exit: "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    ];
    let key_bindings_text = Text::from(Line::from(key_bindings)).style(default_style);
    let key_bindings_paragraph = Paragraph::new(key_bindings_text);
    frame.render_widget(key_bindings_paragraph, key_bindings_rect);
}

fn render_proplogic(frame: &mut Frame, rect: Rect, model: &mut Model) {
    let default_style = default_style();

    let main_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // formula, classification, result, help
                Constraint::Length(1), // key bindings
            ]
            .as_ref(),
        )
        .split(rect);

    let key_bindings_rect = main_vert_split[1];
    let (non_help_rect, help_rect) = if model.show_help {
        let halfs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_vert_split[0]);
        (halfs[0], halfs[1])
    } else {
        (main_vert_split[0], Rect::default())
    };

    let sub_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // formula input
                Constraint::Length(3), // classification
                Constraint::Min(7),    // result / truth table
                Constraint::Length(3), // CNF
                Constraint::Length(3), // DNF
            ]
            .as_ref(),
        )
        .split(non_help_rect);

    let formula_rect = sub_vert_split[0];
    let classification_rect = sub_vert_split[1];
    let result_rect = sub_vert_split[2];
    let cnf_rect = sub_vert_split[3];
    let dnf_rect = sub_vert_split[4];

    // render formula input
    let formula_input = Input::new(model.proplogic_state.formula_input_state.value.clone())
        .with_cursor(model.proplogic_state.formula_input_state.cursor);
    let formula_width = formula_rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let formula_scroll = formula_input.visual_scroll(formula_width as usize);
    let formula_block = Block::default().borders(Borders::ALL);
    let formula_block = if model.focus == Focus::TopicContent {
        formula_block
            .title(" Formula φ* ")
            .title_style(default_style.bold())
    } else {
        formula_block.title(" Formula φ ")
    };
    let formula_paragraph = Paragraph::new(formula_input.value())
        .style(default_style)
        .scroll((0, formula_scroll as u16))
        .block(formula_block);
    frame.render_widget(formula_paragraph, formula_rect);

    frame.set_cursor_position((
        // Put cursor past the end of the input text
        formula_rect.x
            + ((formula_input.visual_cursor()).max(formula_scroll) - formula_scroll) as u16
            + 1,
        // Move one line down, from the border to the input line
        formula_rect.y + 1,
    ));

    // render eval result
    match &model.proplogic_state.result {
        crate::model::PropLogicResult::None => (),
        crate::model::PropLogicResult::Error(e) => {
            let result_paragraph = Paragraph::new(e.as_str())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(result_paragraph, result_rect);
        }
        crate::model::PropLogicResult::Literal(eval_result) => {
            // render formula classification
            let classification = if *eval_result {
                "φ ∈ SAT, ⊢ φ"
            } else {
                "φ ∉ SAT, φ ⊢ ⊥"
            };
            let classification_paragraph =
                Paragraph::new(classification).style(default_style).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Classification "),
                );
            frame.render_widget(classification_paragraph, classification_rect);

            // render formula result
            let result_paragraph = Paragraph::new(eval_result.to_string())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(result_paragraph, result_rect);
        }
        crate::model::PropLogicResult::Table(table) if table.rows.is_empty() => {
            panic!("should not happen")
        }
        crate::model::PropLogicResult::Table(result_table) => {
            // render formula classification
            let classification = {
                let mut c = "".to_string();
                if result_table.is_sat() {
                    c.push_str("φ ∈ SAT");
                    if result_table.is_tautology() {
                        c.push_str(", ⊨ φ");
                    }
                } else {
                    c.push_str("φ ∉ SAT");
                    if result_table.is_contradiction() {
                        c.push_str(", φ ⊢ ⊥");
                    }
                }
                c
            };
            let classification_paragraph =
                Paragraph::new(classification).style(default_style).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Classification "),
                );
            frame.render_widget(classification_paragraph, classification_rect);

            // render truth table
            let vars = result_table.vars();
            let widths = [Constraint::Length(10)].repeat(vars.len() + 2);
            let header = {
                let mut header_names = vars.clone();
                header_names.insert(0, "#".into());
                header_names.push("result".to_string());
                header_names
                    .into_iter()
                    .map(Cell::from)
                    .collect::<Row>()
                    .style(default_style)
                    .height(1)
            };
            let rows: Vec<Row> = result_table
                .rows
                .iter()
                .enumerate()
                .filter_map(|(idx, (assignment, result))| {
                    let show_row = match model.proplogic_state.result_filter {
                        Some(PropLogicResultFilter::OnlyFalse) => !*result,
                        Some(PropLogicResultFilter::OnlyTrue) => *result,
                        _ => true,
                    };
                    if show_row {
                        let mut bools = vec![];
                        bools.push((idx + 1).to_string());
                        vars.iter()
                            .for_each(|var| bools.push(assignment[var].to_string()));
                        bools.push(result.to_string());
                        Some(
                            bools.into_iter().map(Cell::from).collect::<Row>(), // .style(row_style),
                        )
                    } else {
                        None
                    }
                })
                .enumerate()
                .map(|(idx, row)| {
                    let row_style = match idx % 2 {
                        0 => default_style,
                        _ => default_style.bg(Color::Indexed(236u8)),
                    };
                    row.style(row_style)
                })
                .collect();

            let table = {
                let vars_count = result_table.vars().len();
                let rows_count = result_table.rows.len();
                let true_rows_count = result_table.rows.iter().filter(|r| r.1).count();
                let false_rows_count = rows_count - true_rows_count;
                let title = format!(
                    " Result: {vars_c} vars, {rows_c} rows ({true_c} true, {false_c} false){filter} ",
                    vars_c = vars_count,
                    rows_c = rows_count,
                    true_c = true_rows_count,
                    false_c = false_rows_count,
                    filter = match model.proplogic_state.result_filter {
                        Some(PropLogicResultFilter::OnlyFalse) => ", filter: only false",
                        Some(PropLogicResultFilter::OnlyTrue) => ", filter: only true",
                        None => "",
                    }
                );
                Table::new(rows, widths)
                    .header(header)
                    .style(default_style)
                    .block(Block::default().borders(Borders::ALL).title(title))
            };
            frame.render_stateful_widget(
                table,
                result_rect,
                &mut model.proplogic_state.truth_table_state,
            );

            render_scrollbar(
                frame,
                result_rect,
                &mut model.proplogic_state.truth_table_scroll_state,
            );

            // render cnf
            let cnf = match result_table.cnf() {
                Some(e) => e.to_string(),
                None => "".to_string(),
            };
            let cnf_paragraph = Paragraph::new(cnf)
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" CNF "));
            frame.render_widget(cnf_paragraph, cnf_rect);

            // render dnf
            let dnf = match result_table.dnf() {
                Some(e) => e.to_string(),
                None => "".to_string(),
            };
            let dnf_paragraph = Paragraph::new(dnf)
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" DNF "));
            frame.render_widget(dnf_paragraph, dnf_rect);
        }
    };

    // render help if toggled
    if model.show_help {
        let help = "true    // boolean literal true
false   // boolean literal false
p       // a propositional variable
!p      // not, negation
p & q   // and, conjunction
p | q   // or, disjunction
p ^ q   // exclusive or
p <=> q // equivalence
p -> q  // implication";
        let help_paragraph = Paragraph::new(help)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_paragraph, help_rect);
    }

    // render key bindings
    let key_bindings = vec![
        Span::raw("Switch focus: "),
        Span::styled("Tab | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Evaluate: "),
        Span::styled("F5,Enter | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Filter true: "),
        Span::styled("Ctrl-t | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Filter false: "),
        Span::styled("Ctrl-f | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Scroll: "),
        Span::styled("↑/↓ | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Help: "),
        Span::styled("F1 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Exit: "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    ];
    let key_bindings_text = Text::from(Line::from(key_bindings)).style(default_style);
    let key_bindings_paragraph = Paragraph::new(key_bindings_text);
    frame.render_widget(key_bindings_paragraph, key_bindings_rect);
}

fn render_dfa(frame: &mut Frame, rect: Rect, model: &mut Model) {
    let default_style = default_style();

    let main_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // definition, input word
                Constraint::Length(1), // key bindings
            ]
            .as_ref(),
        )
        .split(rect);

    let key_bindings_rect = main_vert_split[1];
    let (non_help_rect, help_rect) = if model.show_help {
        let halfs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_vert_split[0]);
        (halfs[0], halfs[1])
    } else {
        (main_vert_split[0], Rect::default())
    };

    let sub_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(7),    // definition
                Constraint::Length(3), // input word
                Constraint::Min(3),    // transitions
                Constraint::Length(3), // result
            ]
            .as_ref(),
        )
        .split(non_help_rect);

    let definition_rect = sub_vert_split[0];
    let word_input_rect = sub_vert_split[1];
    let transitions_rect = sub_vert_split[2];
    let result_rect = sub_vert_split[3];

    model
        .dfa_state
        .definition_textarea
        .set_cursor_line_style(default_style);
    model
        .dfa_state
        .input_word_textarea
        .set_cursor_line_style(default_style);
    model
        .dfa_state
        .transitions
        .set_line_number_style(default_style);

    // render definition textarea
    let definition_block = Block::default().borders(Borders::ALL).style(default_style);
    let definition_block =
        if model.focus == Focus::TopicContent && model.dfa_state.focus == DfaFocus::Definition {
            definition_block
                .title(" Definition* ")
                .title_style(default_style.bold())
        } else {
            definition_block.title(" Definition ")
        };
    model
        .dfa_state
        .definition_textarea
        .set_block(definition_block);
    frame.render_widget(&model.dfa_state.definition_textarea, definition_rect);

    // render word input textarea
    let word_input_block = Block::default().borders(Borders::ALL).style(default_style);
    let word_input_block =
        if model.focus == Focus::TopicContent && model.dfa_state.focus == DfaFocus::WordInput {
            word_input_block
                .title(" Word* ")
                .title_style(default_style.bold())
        } else {
            word_input_block.title(" Word ")
        };
    model
        .dfa_state
        .input_word_textarea
        .set_block(word_input_block);
    frame.render_widget(&model.dfa_state.input_word_textarea, word_input_rect);

    let transitions_block = Block::default().borders(Borders::ALL).style(default_style);
    let transitions_block = {
        let count = model.dfa_state.transitions.lines().len();
        let show_count = matches!(model.dfa_state.result, DfaResult::Accepted(_));
        let has_focus =
            model.focus == Focus::TopicContent && model.dfa_state.focus == DfaFocus::Transitions;
        let mut title = " Transitions".to_string();
        if show_count {
            title.push_str(format!(" ({count})").as_str());
        }
        if has_focus {
            title.push_str("* ");
            transitions_block
                .title(title)
                .set_style(default_style.bold())
        } else {
            title.push(' ');
            transitions_block.title(title)
        }
    };
    model.dfa_state.transitions.set_block(transitions_block);
    frame.render_widget(&model.dfa_state.transitions, transitions_rect);

    // render result
    let result_paragraph = {
        let result = match &model.dfa_state.result {
            DfaResult::None => "".to_string(),
            DfaResult::Error(e) => e.clone(),
            DfaResult::Accepted(b) => b.to_string(),
        };

        Paragraph::new(result).style(default_style).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Acceptor Result "),
        )
    };
    frame.render_widget(result_paragraph, result_rect);

    // render help if toggled
    if model.show_help {
        let help = "Sigma = { 'a', 'b' }                     // the set of the alphabet symbols
S = { s0, s1, s2 }                       // the set of states
start = s0                               // the start state
F = { s2  }                              // the set of final states
delta = { (s0, 'a', s1), (s1, 'b', s2) } // the set of state transitions of the delta function";
        let help_paragraph = Paragraph::new(help)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_paragraph, help_rect);
    }

    // render key bindings
    let key_bindings = vec![
        Span::raw("Switch focus: "),
        Span::styled("Tab | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Evaluate: "),
        Span::styled("F5 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Help: "),
        Span::styled("F1 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Exit: "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    ];
    let key_bindings_text = Text::from(Line::from(key_bindings)).style(default_style);
    let key_bindings_paragraph = Paragraph::new(key_bindings_text);
    frame.render_widget(key_bindings_paragraph, key_bindings_rect);
}

fn render_t3grammar(frame: &mut Frame<'_>, rect: Rect, model: &mut Model<'_>) {
    let default_style = default_style();

    let main_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // definition, input word
                Constraint::Length(1), // key bindings
            ]
            .as_ref(),
        )
        .split(rect);

    let key_bindings_rect = main_vert_split[1];
    let (non_help_rect, help_rect) = if model.show_help {
        let halfs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_vert_split[0]);
        (halfs[0], halfs[1])
    } else {
        (main_vert_split[0], Rect::default())
    };

    let sub_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(7),    // definition
                Constraint::Length(3), // input word
                Constraint::Min(3),    // transitions
                Constraint::Length(3), // result
            ]
            .as_ref(),
        )
        .split(non_help_rect);

    let definition_rect = sub_vert_split[0];
    let word_input_rect = sub_vert_split[1];
    let productions_rect = sub_vert_split[2];
    let result_rect = sub_vert_split[3];

    model
        .t3grammar_state
        .definition_textarea
        .set_cursor_line_style(default_style);
    model
        .t3grammar_state
        .input_word_textarea
        .set_cursor_line_style(default_style);
    model
        .t3grammar_state
        .productions
        .set_line_number_style(default_style);

    // render definition textarea
    let definition_block = Block::default().borders(Borders::ALL).style(default_style);
    let definition_block = if model.focus == Focus::TopicContent
        && model.t3grammar_state.focus == T3GrammarFocus::Definition
    {
        definition_block
            .title(" Definition G* ")
            .title_style(default_style.bold())
    } else {
        definition_block.title(" Definition G ")
    };
    model
        .t3grammar_state
        .definition_textarea
        .set_block(definition_block);
    frame.render_widget(&model.t3grammar_state.definition_textarea, definition_rect);

    // render word input textarea
    let word_input_block = Block::default().borders(Borders::ALL).style(default_style);
    let word_input_block = if model.focus == Focus::TopicContent
        && model.t3grammar_state.focus == T3GrammarFocus::WordInput
    {
        word_input_block
            .title(" Word w* ")
            .title_style(default_style.bold())
    } else {
        word_input_block.title(" Word w ")
    };
    model
        .t3grammar_state
        .input_word_textarea
        .set_block(word_input_block);
    frame.render_widget(&model.t3grammar_state.input_word_textarea, word_input_rect);

    let productions_block = Block::default().borders(Borders::ALL).style(default_style);
    let productions_block = {
        let count = if let T3GrammarResult::Produced(c) = model.t3grammar_state.result {
            c
        } else {
            0
        };
        let has_focus = model.focus == Focus::TopicContent
            && model.t3grammar_state.focus == T3GrammarFocus::Productions;
        let mut title = " Productions".to_string();
        if count > 0 {
            title.push_str(format!(" ({count})").as_str());
        }
        if has_focus {
            title.push_str("* ");
            productions_block
                .title(title)
                .set_style(default_style.bold())
        } else {
            title.push(' ');
            productions_block.title(title)
        }
    };
    model
        .t3grammar_state
        .productions
        .set_block(productions_block);
    frame.render_widget(&model.t3grammar_state.productions, productions_rect);

    // render result
    let result_paragraph = {
        let result = match &model.t3grammar_state.result {
            T3GrammarResult::None => "".to_string(),
            T3GrammarResult::Error(e) => e.clone(),
            T3GrammarResult::Produced(b) if *b == 0 => "w ∉ L(G)".to_string(),
            T3GrammarResult::Produced(_) => "w ∈ L(G)".to_string(),
        };

        Paragraph::new(result)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Result "))
    };
    frame.render_widget(result_paragraph, result_rect);

    // render help if toggled
    if model.show_help {
        let help = "V = { S, T }                                    // the set of non-terminals
Sigma = { 'a', 'b' }                            // the set of the terminal symbols
P = { S -> 'aT', T -> 'b', T -> 'bT', T -> '' } // the set of production rules
S = S                                           // the start non-terminal";
        let help_paragraph = Paragraph::new(help)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_paragraph, help_rect);
    }

    // render key bindings
    let key_bindings = vec![
        Span::raw("Switch focus: "),
        Span::styled("Tab | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Try productions: "),
        Span::styled("F5 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Help: "),
        Span::styled("F1 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Exit: "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    ];
    let key_bindings_text = Text::from(Line::from(key_bindings)).style(default_style);
    let key_bindings_paragraph = Paragraph::new(key_bindings_text);
    frame.render_widget(key_bindings_paragraph, key_bindings_rect);
}

fn render_t2grammar(frame: &mut Frame<'_>, rect: Rect, model: &mut Model<'_>) {
    let default_style = default_style();

    let main_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),    // definition, input word
                Constraint::Length(1), // key bindings
            ]
            .as_ref(),
        )
        .split(rect);

    let key_bindings_rect = main_vert_split[1];
    let (non_help_rect, help_rect) = if model.show_help {
        let halfs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_vert_split[0]);
        (halfs[0], halfs[1])
    } else {
        (main_vert_split[0], Rect::default())
    };

    let sub_vert_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(7),    // definition
                Constraint::Length(3), // input word
                Constraint::Min(3),    // transitions
                Constraint::Length(3), // result
            ]
            .as_ref(),
        )
        .split(non_help_rect);

    let definition_rect = sub_vert_split[0];
    let word_input_rect = sub_vert_split[1];
    let productions_rect = sub_vert_split[2];
    let result_rect = sub_vert_split[3];

    model
        .t2grammar_state
        .definition_textarea
        .set_cursor_line_style(default_style);
    model
        .t2grammar_state
        .input_word_textarea
        .set_cursor_line_style(default_style);
    model
        .t2grammar_state
        .productions
        .set_line_number_style(default_style);

    // render definition textarea
    let definition_block = Block::default().borders(Borders::ALL).style(default_style);
    let definition_block = if model.focus == Focus::TopicContent
        && model.t2grammar_state.focus == T2GrammarFocus::Definition
    {
        definition_block
            .title(" Definition G* ")
            .title_style(default_style.bold())
    } else {
        definition_block.title(" Definition G ")
    };
    model
        .t2grammar_state
        .definition_textarea
        .set_block(definition_block);
    frame.render_widget(&model.t2grammar_state.definition_textarea, definition_rect);

    // render word input textarea
    let word_input_block = Block::default().borders(Borders::ALL).style(default_style);
    let word_input_block = if model.focus == Focus::TopicContent
        && model.t2grammar_state.focus == T2GrammarFocus::WordInput
    {
        word_input_block
            .title(" Word w* ")
            .title_style(default_style.bold())
    } else {
        word_input_block.title(" Word w ")
    };
    model
        .t2grammar_state
        .input_word_textarea
        .set_block(word_input_block);
    frame.render_widget(&model.t2grammar_state.input_word_textarea, word_input_rect);

    let productions_block = Block::default().borders(Borders::ALL).style(default_style);
    let productions_block = {
        let has_focus = model.focus == Focus::TopicContent
            && model.t2grammar_state.focus == T2GrammarFocus::Productions;
        let mut title = " Productions".to_string();
        if has_focus {
            title.push_str("* ");
            productions_block
                .title(title)
                .set_style(default_style.bold())
        } else {
            title.push(' ');
            productions_block.title(title)
        }
    };
    model
        .t2grammar_state
        .productions
        .set_block(productions_block);
    frame.render_widget(&model.t2grammar_state.productions, productions_rect);

    // render result
    let result_paragraph = {
        let result = match &model.t2grammar_state.result {
            T2GrammarResult::None => "".to_string(),
            T2GrammarResult::Error(e) => e.clone(),
            T2GrammarResult::Produced(false) => "w ∉ L(G)".to_string(),
            T2GrammarResult::Produced(true) => "w ∈ L(G)".to_string(),
        };

        Paragraph::new(result)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Result "))
    };
    frame.render_widget(result_paragraph, result_rect);

    // render help if toggled
    if model.show_help {
        let help = "V = { S }                     // the set of non-terminals
Sigma = { '(', ')' }          // the set of the terminal symbols
P = { S -> '(S)', S -> '()' } // the set of production rules
S = S                         // the start non-terminal";
        let help_paragraph = Paragraph::new(help)
            .style(default_style)
            .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help_paragraph, help_rect);
    }

    // render key bindings
    let key_bindings = vec![
        Span::raw("Switch focus: "),
        Span::styled("Tab | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Try productions: "),
        Span::styled("F5 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Help: "),
        Span::styled("F1 | ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("Exit: "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    ];
    let key_bindings_text = Text::from(Line::from(key_bindings)).style(default_style);
    let key_bindings_paragraph = Paragraph::new(key_bindings_text);
    frame.render_widget(key_bindings_paragraph, key_bindings_rect);
}

fn render_scrollbar(frame: &mut Frame, area: Rect, scroll_state: &mut ScrollbarState) {
    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        scroll_state,
    );
}
