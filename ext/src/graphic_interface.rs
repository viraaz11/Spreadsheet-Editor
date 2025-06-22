// use std::f64::INFINITY;
// use std::io;
use std::cmp;

use crate::cell_operations::{Sheet, ValueType};

// use crossterm::{
//     execute,
//     terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };

// use ratatui::text::Text;
use ratatui::widgets::Dataset;
use ratatui::{
    // backend::CrosstermBackend,
    Frame,
    text::{Line, Span},
    widgets::{Table, Row, Cell, Block, Borders, Paragraph, ScrollbarState, Scrollbar, ScrollbarOrientation, Tabs, Axis, Chart, GraphType, Wrap},
    layout::{Constraint, Rect, Position},
    style::{Style, Color, palette::tailwind, Stylize},
    symbols::Marker
};

pub struct StyleGuide {

    pub table_header: Style,
    pub selected_cell: Style,
    pub table_even_row: Style,
    pub table_odd_row: Style,
    pub inp_box_idle_mode: Style,
    pub inp_box_editing_mode: Style,
    pub history_widget: Style,
    pub tabs_unselected: Style,
    pub tabs_selected: Style,
    pub cell_details_header: Style,
    pub cell_details_even_row: Style,
    pub cell_details_odd_row: Style


}


//For color options, either use the Color::Name foramt or you can also use from tailwind [eg: Style::default().fg(tailwind::SLATE.c900) ]
impl Default for StyleGuide {
    fn default() -> Self {
        Self::new()
    }
}
impl StyleGuide {
    pub fn new() -> Self {
        Self {
            table_header: Style::default().fg(Color::White).bg(tailwind::BLUE.c900),
            selected_cell: Style::default(), //NOT YET IMPLEMENTED
            table_even_row: Style::default().bg(tailwind::SLATE.c950),
            table_odd_row: Style::default().bg(tailwind::SLATE.c900),
            inp_box_idle_mode: Style::default(),
            inp_box_editing_mode: Style::default().fg(Color::Yellow),
            history_widget: Style::default(),
            tabs_unselected: Style::default().fg(tailwind::SLATE.c500),
            tabs_selected: Style::default().fg(tailwind::BLUE.c100),
            cell_details_header: Style::default().fg(Color::White).bg(tailwind::BLUE.c900),
            cell_details_even_row: Style::default().bg(tailwind::SLATE.c950),
            cell_details_odd_row: Style::default().bg(tailwind::SLATE.c900),

        }
    }
}





/// App holds the state of the application
pub struct TextInputWidget {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// Current input mode
    pub input_mode: InputMode,
}

pub enum InputMode {
    Normal,
    Editing,
}
impl Default for TextInputWidget {
    fn default() -> Self {
        Self::new()
    }
}
impl TextInputWidget {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn draw(&self, area: Rect, frame: &mut Frame, styleguide: &StyleGuide) {

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => styleguide.inp_box_idle_mode,
                InputMode::Editing => styleguide.inp_box_editing_mode,
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                area.y + 1,
            )),
        };
        // }).map_err(|x| format!("Error during draw_table: {x:?}")).map(|x| ())
    }
}

pub struct HistoryWidget {
    pub history: Vec<(String, String)>, //NOTE: Todo time??
    pub scrollstate: ScrollbarState,
    pub scroll_amt: usize
}

impl Default for HistoryWidget {
    fn default() -> Self {
        Self::new()
    }
}
impl HistoryWidget {
    pub fn new() -> Self {
        HistoryWidget{ history: vec![(String::from("Starting"),String::from(" All The best"))] ,scrollstate: ScrollbarState::new(0), scroll_amt: 0} //NOTE: PUT CONTENT LENGTH IN SETTINGS!!!!!!
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, styleguide: &StyleGuide) {
        if self.history.len() > 50 {
            self.history.remove(0);
        }

        let text = self.history.iter().map(|(x, y)| Line::from(format!("> {} >> {}", x, y))).collect::<Vec<Line>>();

        let create_block = |title: &'static str| Block::bordered().gray().title(title.bold());

        self.scrollstate = self.scrollstate.content_length(self.history.len()).position(self.scroll_amt);

        let paragraph = Paragraph::new(text.clone())
            .style(styleguide.history_widget)
            .block(create_block("History"))
            .scroll((self.scroll_amt as u16, 0));
        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.scrollstate,
        );

    }
}


pub struct TabsWidget {
    pub tabs: Vec<String>,
    pub index: usize
}

impl TabsWidget {

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, styleguide: &StyleGuide) {
        let tabs =self.tabs
        .iter()
        .map(|t| Line::from(Span::styled(t.clone(), styleguide.tabs_unselected)))
        .collect::<Tabs>()
        .block(Block::bordered())    //        .block(Block::bordered().title("here")) If title wanted
        .highlight_style(styleguide.tabs_selected)
        .select(self.index);
        frame.render_widget(tabs, area);

    }
}

pub struct CellDetailsWidget {
    // pub tabs: Vec<String>,
    // pub index: usize
}

impl CellDetailsWidget {

    pub fn draw(&mut self, col: usize, row: usize, sheet: &Sheet,  area: Rect, frame: &mut Frame, styleguide: &StyleGuide) {
        let block = Block::default()
        .title("Cell Details")
        .borders(Borders::ALL);

        let header = Row::new(vec![String::from("Property"), String::from("Value")])
            .style(styleguide.cell_details_header);

        let curr_cell_col = sheet.data[col].borrow();
        let data = if row >= curr_cell_col.cells.len() {
            vec![
                vec!["Column".to_string(), col.to_string()],
                vec!["Row".to_string(), row.to_string()],
                vec!["Value".to_string(), "~".to_string()],
                vec!["Expression".to_string(), "~".to_string()],
            ]
        } else {
            let curr_cell = curr_cell_col.cells[row].borrow();
    
            vec![
                vec!["Column".to_string(), curr_cell.addr.col.to_string()],
                vec!["Row".to_string(), curr_cell.addr.row.to_string()],
                vec!["Value".to_string(),{
                    if curr_cell.valid {
                        match &curr_cell.value {
                            ValueType::BoolValue(b) => b.to_string(),
                            ValueType::IntegerValue(x) => x.to_string(),
                            ValueType::FloatValue(n) => n.to_string(),
                            ValueType::String(s) => s.clone(),
                        }
                    } else {
                        "~".to_string()
                    }
                }],

                vec!["Expression".to_string(),
                    match &curr_cell.cell_func {
                        Some(_) => curr_cell.formula.clone(),
                        None => "~".to_string(),
                    }],
            ]
        };

        // Convert data into styled rows with alternating backgrounds
        let rows: Vec<Row> = data
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let style = if i % 2 == 0 {
                    styleguide.table_even_row
                } else {
                    styleguide.table_odd_row
                };

                let cells = row.into_iter().map(Cell::from);
                Row::new(cells).style(style)
            })
            .collect();

        let widths = vec![Constraint::Length(10), Constraint::Length(10)];
        // Build the table
        let table = Table::new(rows, widths)
            .header(header)
            .block(block)
            .column_spacing(2); // extra spacing between columns
        frame.render_widget(table, area);

    }
}


pub struct OutputsWidget {
    pub col1: usize,
    pub row_start1:usize,
    pub row_end1: usize,
    pub col2: usize,
    pub row_start2:usize,
    pub row_end2: usize,
    pub sheetnum: usize,
    pub xlabel: String,
    pub ylabel: String,

    }
impl Default for OutputsWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputsWidget {

    pub fn new() -> Self {
        OutputsWidget {
        col1: 0,
        row_start1:0,
        row_end1: 0,
        col2: 0,
        row_start2:0,
        row_end2: 0,
        sheetnum: 0,
        xlabel: String::new(),
        ylabel: String::new(),
        }
    }



    pub fn draw_chart(&mut self, sheet: &Sheet, area: Rect, frame: &mut Frame, styleguide: &StyleGuide, invalid: bool) {
        let mut invalid = invalid;
        let mut data: Vec<(f64,f64)> = vec![];
        let mut min_val1: f64 = f64::MAX;
        let mut min_val2: f64 = f64::MAX;
        let mut max_val1: f64 = f64::MIN;
        let mut max_val2: f64 = f64::MIN;
        for i in 0..=self.row_end1-self.row_start1 {
            let val1: f64;
            let val2: f64;

            if self.col1 >= sheet.data.len() {
                invalid = true;
                break
            }
            let colref1 = match sheet.data[self.col1].try_borrow() {
                Err(_) => {
                    invalid = true;
                    break
                },
                Ok(x) => x
            };
            if self.row_start1 + i >= colref1.cells.len() {
                invalid = true;
                break
            }

            let cell1 = match colref1.cells[self.row_start1 + i].try_borrow() {
                Err(_) => {
                    invalid = true;
                    break
                },
                Ok(x) => x
            };

            if cell1.valid {
                let val =  &cell1.value;
                match val {
                    ValueType::BoolValue(_) => val1 = 0.0,
                    ValueType::IntegerValue(x) => val1 = *x as f64,
                    ValueType::FloatValue(n) => val1 = *n,
                    ValueType::String(_) => val1 = 0.0,
                }
            }
            else { val1 = 0.0 };

            if self.col2 >= sheet.data.len() {
                invalid = true;
                break
            }
            let colref2 = match sheet.data[self.col2].try_borrow() {
                Err(_) => {
                    invalid = true;
                    break
                },
                Ok(x) => x
            };
            if self.row_start2 + i >= colref2.cells.len() {
                invalid = true;
                break
            }

            let cell2 = match colref2.cells[self.row_start2 + i].try_borrow() {
                Err(_) => {
                    invalid = true;
                    break
                },
                Ok(x) => x
            };
            if cell2.valid {
                let val =  &cell2.value;
                match val {
                    ValueType::BoolValue(_) => val2 = 0.0,
                    ValueType::IntegerValue(x) => val2 = *x as f64,
                    ValueType::FloatValue(n) => val2 = *n,
                    ValueType::String(_) => val2 = 0.0,
                }
            }
            else { val2 = 0.0 };
            min_val1 = if min_val1 < val1 {min_val1} else {val1};
            min_val2 = if min_val2 < val2 {min_val2} else {val2};
            max_val1 = if max_val1 > val1 {max_val1} else {val1};
            max_val2 = if max_val2 > val2 {max_val2} else {val2};
            data.push((val1, val2));
        }
        
        if invalid {
            self.draw_text(String::from("Data not valid, run make_chart again."), area, frame, styleguide);
            return;
        }



        let dataset = Dataset::default()
            // .name("Values")
            .marker(Marker::Braille)
            .graph_type(GraphType::Scatter)
            .style(Style::new().cyan())
            .data(&data);

        let chart = Chart::new(vec![dataset])
        .block(Block::bordered().title(Line::from("Scatter chart").cyan().bold().centered()))
        .x_axis(
            Axis::default()
                .title("First_Range")
                .bounds([min_val1, max_val1])
                .style(Style::default().fg(Color::Gray))
                .labels([min_val1.to_string(), max_val1.to_string()])
        )
        .y_axis(
            Axis::default()
                .title("Second_Range")
                .bounds([min_val2, max_val2])
                .style(Style::default().fg(Color::Gray))
                .labels([min_val2.to_string(), max_val2.to_string()])
        );
        frame.render_widget(chart, area);
    }

    pub fn draw_idle(&mut self, area: Rect, frame: &mut Frame, _styleguide: &StyleGuide) {
        let paragraph = Paragraph::new(Line::from("Nothing to see.."))
        .block(Block::bordered().title("Output"))
        .centered();
    
        frame.render_widget(paragraph, area);
    }

    pub fn draw_text(&mut self, text: String, area: Rect, frame: &mut Frame, _styleguide: &StyleGuide) {
        let paragraph = Paragraph::new(Line::from(text))
        .block(Block::bordered().title("Output"))
        .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}






pub fn draw_table(col: usize, row: usize, sheet: &Sheet, title: &str, area: Rect, frame: &mut Frame, styleguide: &StyleGuide) {

    let column_width = 5;
    // let mut area = f.area();
    // Table block (outer border + title)
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);

    let row_max = cmp::min(row + area.height.saturating_sub(2) as usize, sheet.rows as usize);
    let col_max = cmp::min(col + area.width.saturating_sub(column_width+2).saturating_div(column_width+2) as usize, sheet.columns as usize);

    // Header row
    let mut row_heads_vec: Vec<String> = vec![String::from("")];
    for i in col..col_max {
        let mut curr = String::new();
        let mut curr_col = i + 1;
        while curr_col > 0
        {

            curr.push(((b'A') + ((curr_col-1) % 26) as u8) as char);
            
            curr_col -= 1;
            curr_col /= 26;
        }
        row_heads_vec.push(curr.chars().rev().collect::<String>());
    }
    let num_cols = row_heads_vec.len();


    let header = Row::new(row_heads_vec)
        .style(styleguide.table_header);


    let mut data: Vec<Vec<String>> = vec![];
    for i in row..row_max {
        let mut curr_row_vec = vec![(i+1).to_string()];
        for j in col..col_max {
            let colref = sheet.data[j].borrow();
            if i >= colref.cells.len()
            {
                curr_row_vec.push(String::from("~"));
                continue
            } 
            else
            {
                let cell = colref.cells[i].borrow();
                if cell.valid {
                    let val =  &cell.value;
                    match val {
                        ValueType::BoolValue(b) => curr_row_vec.push(b.to_string()),
                        ValueType::IntegerValue(x) => curr_row_vec.push(x.to_string()),
                        ValueType::FloatValue(n) => curr_row_vec.push(n.to_string()),
                        ValueType::String(s) => curr_row_vec.push(s.to_string()),
                    }
                }
                else {
                    curr_row_vec.push(String::from("ERR"));
                }
            }
        };
        data.push(curr_row_vec);
    }


    // Convert data into styled rows with alternating backgrounds
    let rows: Vec<Row> = data
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            let style = if i % 2 == 0 {
                styleguide.table_even_row
            } else {
                styleguide.table_odd_row
            };
            let cells = row.into_iter().map(Cell::from);
            Row::new(cells).style(style)
        })
        .collect();

    let widths = vec![Constraint::Length(column_width); num_cols];
    // Build the table
    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(2); // extra spacing between columns
    frame.render_widget(table, area);
    // }).map_err(|x| format!("Error during draw_table: {x:?}")).map(|x| ())
}
