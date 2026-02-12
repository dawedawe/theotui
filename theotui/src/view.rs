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
                Constraint::Length(2),
                Constraint::Length(3),  // input
                Constraint::Length(20), // output
                Constraint::Min(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(frame.area());

    let formula_rect = chunks[1];
    let formula_rect = center_horizontal(formula_rect, 100);
    let _main_rect = chunks[2];

    let formula_input = Input::new(model.formula_input_state.value.clone())
        .with_cursor(model.formula_input_state.cursor);
    let formula_width = formula_rect.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let formula_scroll = formula_input.visual_scroll(formula_width as usize);
    let formula_paragraph = Paragraph::new(formula_input.value())
        .style(default_style)
        .scroll((0, formula_scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(" Formula "));
    frame.render_widget(formula_paragraph, formula_rect);

    frame.set_cursor_position((
        // Put cursor past the end of the input text
        formula_rect.x
            + ((formula_input.visual_cursor()).max(formula_scroll) - formula_scroll) as u16
            + 1,
        // Move one line down, from the border to the input line
        formula_rect.y + 1,
    ));

    match &model.output {
        crate::model::PropLogicOutput::None => (),
        crate::model::PropLogicOutput::Literal(output) => {
            let output_rect = chunks[2];
            let output_rect = center_horizontal(output_rect, 100);
            let output_paragraph = Paragraph::new(output.clone())
                .style(default_style)
                .block(Block::default().borders(Borders::ALL).title(" Result "));
            frame.render_widget(output_paragraph, output_rect);
        }
        crate::model::PropLogicOutput::Table(table) if table.is_empty() => {
            panic!("should not happen")
        }
        crate::model::PropLogicOutput::Table(table) => {
            let keys = theoinf::propositional_logic::formula_vars(&table[0].0);
            let widths = [Constraint::Length(10)].repeat(keys.len() + 1);
            let header = {
                let mut header_names = keys.clone();
                header_names.push("result".to_string());
                header_names
                    .into_iter()
                    .map(Cell::from)
                    .collect::<Row>()
                    .style(default_style)
                    .height(1)
            };
            let rows: Vec<Row> = table
                .iter()
                .enumerate()
                .map(|(idx, (assignment, result))| {
                    let row_style = match idx % 2 {
                        0 => default_style,
                        _ => default_style.bg(Color::Indexed(236u8)),
                    };
                    let mut bools = vec![];
                    keys.iter()
                        .for_each(|key| bools.push(assignment[key].to_string()));
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
            let table_rect = chunks[2];
            let table_rect = center_horizontal(table_rect, 100);
            frame.render_stateful_widget(t, table_rect, &mut model.truth_table_state);

            render_scrollbar(frame, table_rect, &mut model.truth_table_scroll_state);
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
