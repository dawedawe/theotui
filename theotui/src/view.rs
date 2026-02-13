use crate::model::Model;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table,
    },
};
use tui_input::Input;

pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    fn center_horizontal(area: Rect, width: u16) -> Rect {
        let [area] = Layout::horizontal([Constraint::Length(width)])
            .flex(Flex::Center)
            .areas(area);
        area
    }

    let default_style = Style::default().fg(Color::Green);

    let chunks = Layout::default()
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
        .split(frame.area());

    let formula_rect = center_horizontal(chunks[0], 100);
    let classification_rect = center_horizontal(chunks[1], 100);
    let result_rect = center_horizontal(chunks[2], 100);

    // render formula input
    let formula_input = Input::new(model.formula_input_state.value.clone())
        .with_cursor(model.formula_input_state.cursor);
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
    match &model.output {
        crate::model::PropLogicOutput::None => (),
        crate::model::PropLogicOutput::Error(e) => {
            let result_paragraph = Paragraph::new(e.clone())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(result_paragraph, result_rect);
        }
        crate::model::PropLogicOutput::Literal(eval_result) => {
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
        crate::model::PropLogicOutput::Table(table) if table.rows.is_empty() => {
            panic!("should not happen")
        }
        crate::model::PropLogicOutput::Table(table) => {
            // render formula classification
            let classification = {
                let mut c = "".to_string();
                if table.is_sat() {
                    c.push_str("φ ∈ SAT");
                    if table.is_tautology() {
                        c.push_str(", ⊢ φ");
                    }
                } else {
                    c.push_str("φ ∉ SAT");
                    if table.is_contradiction() {
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
            let vars = table.vars();
            let widths = [Constraint::Length(10)].repeat(vars.len() + 1);
            let header = {
                let mut header_names = vars.clone();
                header_names.push("result".to_string());
                header_names
                    .into_iter()
                    .map(Cell::from)
                    .collect::<Row>()
                    .style(default_style)
                    .height(1)
            };
            let rows: Vec<Row> = table
                .rows
                .iter()
                .enumerate()
                .map(|(idx, (assignment, result))| {
                    let row_style = match idx % 2 {
                        0 => default_style,
                        _ => default_style.bg(Color::Indexed(236u8)),
                    };
                    let mut bools = vec![];
                    vars.iter()
                        .for_each(|var| bools.push(assignment[var].to_string()));
                    bools.push(result.to_string());
                    bools
                        .into_iter()
                        .map(Cell::from)
                        .collect::<Row>()
                        .style(row_style)
                })
                .collect();

            let t = Table::new(rows, widths)
                .header(header)
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_stateful_widget(t, result_rect, &mut model.truth_table_state);

            render_scrollbar(frame, result_rect, &mut model.truth_table_scroll_state);
        }
    };
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
