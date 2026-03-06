use crate::model::{Model, PropLogicResultFilter, SelectedTopic};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{
        Block, Borders, Cell, List, ListState, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
};
use strum::IntoEnumIterator;
use tui_input::Input;

fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn default_style() -> Style {
    Style::default().fg(Color::Green)
}
pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    let default_style = default_style();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints(
            [
                Constraint::Length(50), // tabs input
                Constraint::Min(1),     // tab content
            ]
            .as_ref(),
        )
        .split(frame.area());

    let tabs_rect = chunks[0];
    let tab_content_rect = chunks[1];

    // render topic list
    let items = SelectedTopic::iter().map(|t| t.to_string());
    let highlight_style = default_style.bold();
    let selected_tab_index = model.selected_topic as usize;
    let topic_list = List::new(items)
        .style(default_style)
        .highlight_style(highlight_style.bold())
        .highlight_symbol("> ")
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
    let mut topic_list_state = ListState::default().with_selected(Some(selected_tab_index));

    frame.render_stateful_widget(topic_list, tabs_rect, &mut topic_list_state);

    match model.selected_topic {
        SelectedTopic::PropositionalLogic => render_proplogic(frame, tab_content_rect, model),
        SelectedTopic::SetTheory => render_settheory(frame, tab_content_rect, model),
    }
}

fn render_proplogic(frame: &mut Frame, rect: Rect, model: &mut Model) {
    let default_style = default_style();

    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),  // formula input
                Constraint::Length(3),  // classification
                Constraint::Length(20), // result / truth table
                Constraint::Min(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(rect);

    let formula_rect = center_horizontal(tab_content_chunks[0], 100);
    let classification_rect = center_horizontal(tab_content_chunks[1], 100);
    let result_rect = center_horizontal(tab_content_chunks[2], 100);

    // render formula input
    let formula_input = Input::new(model.proplogic_state.formula_input_state.value.clone())
        .with_cursor(model.proplogic_state.formula_input_state.cursor);
    let formula_width = formula_rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let formula_scroll = formula_input.visual_scroll(formula_width as usize);
    let formula_paragraph = Paragraph::new(formula_input.value())
        .style(default_style)
        .scroll((0, formula_scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(" Formula φ "));
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
            let result_paragraph = Paragraph::new(e.clone())
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
        }
    };
}

fn render_settheory(frame: &mut Frame, rect: Rect, model: &mut Model) {
    let default_style = default_style();

    let tab_content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(7),  // term input
                Constraint::Length(20), // result
                Constraint::Min(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(rect);

    let term_rect = center_horizontal(tab_content_chunks[0], 100);
    let result_rect = center_horizontal(tab_content_chunks[1], 100);

    let editor_block = Block::default()
        .borders(Borders::ALL)
        .title(" Term ")
        .style(default_style);

    frame.render_widget(editor_block, term_rect);
    let editor_rect = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(term_rect);
    model
        .settheory_state
        .term_textarea
        .set_cursor_line_style(default_style);
    frame.render_widget(&model.settheory_state.term_textarea, editor_rect[0]);

    // render eval result
    match &model.settheory_state.result {
        &crate::model::SetTheoryResult::None => (),
        crate::model::SetTheoryResult::Error(e) => {
            let result_paragraph = Paragraph::new(e.clone())
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
