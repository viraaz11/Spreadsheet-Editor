pub mod ast;
pub mod tokensexpr;
pub mod tokenscmds;
pub mod cell_operations;
pub mod evaluate_operations;
pub mod graphic_interface;
use graphic_interface::CellDetailsWidget;
use graphic_interface::HistoryWidget;
use graphic_interface::OutputsWidget;
use graphic_interface::TabsWidget;
use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;
use logos::Logos;
// use ratatui::style::Style;
use crate::ast::{Expr, Addr, ParentType};
use csv::ReaderBuilder;
use crate::cell_operations::{Cell, CellFunc, Sheet, ValueType};
use crate::evaluate_operations::evaluate;
use crate::graphic_interface::{draw_table, StyleGuide, TextInputWidget, InputMode};
// use crate::tokenscmds;
// use crate::tokensexpr;
use std::io::{self, Write, BufWriter};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
// use std::time::Instant;
use std::fs::File;
// use serde::Serialize;
// use csv::Reader;

// --For Graphics--
use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    backend::CrosstermBackend,
    Terminal,
    layout::{
        Constraint::{Min, Percentage},
        Layout,
    },
    // widgets::{Table, Row, Cell, Block, Borders},
    // layout::{Constraint, Direction, Layout, Rect},
    // style::{Style, Color, palette::tailwind},
};
// -x-






//NOTE: PLEASE HAR JAGA usize KAR DO, bohot zyada conversions karne pad rahe hai


lalrpop_mod!(pub grammarexpr); // include the generated parser
lalrpop_mod!(pub grammarcmds); // include the generated parser

/// A simple function to demonstrate Markdown syntax.
///
/// # Details
/// This function adds two numbers together and returns the result.
///
/// **Arguments:**
/// - `a`: The first number.
/// - `b`: The second number.
///
/// **Returns:**
/// - The sum of `a` and `b`.
///
/// # Examples
/// ```
/// let sum = add(5, 3);
/// assert_eq!(sum, 8);
/// ```
/// A struct for storing all the sheets created and managing sheet operations.
pub struct SheetStorage {
    pub map: Vec<(String, usize)>,
    pub data: Vec<Rc<RefCell<Sheet>>>   //NOTE: This should be made int Option<Rc<...>>
}

impl Default for SheetStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetStorage {

    /// # Details
    /// Function to create a new SheetStorage
    ///
    /// **Returns:**
    /// - new `SheetStorage` instance
    pub fn new() -> Self {
        SheetStorage{
            map: vec![],
            data: vec![]
        }
    }

    /// # Details
    /// Method to list names of all the sheets
    ///
    /// **Returns:**
    /// - `Vec<String>` containing the names of all the active sheets.
    pub fn list_names(&self) -> Vec<String> {
        self.map.iter().map(|x| x.0.clone()).collect::<Vec<String>>()
    }

    /// # Details
    /// Method to get global sheet index of a sheet number.
    /// 
    /// **Arguments:**
    /// - `num`: The sheet number.
    /// 
    /// **Returns:**
    /// - `Some(usize)` containing global index if num is valid, else `None`.
    pub fn list_index_from_num(&self, num: usize) -> Option<usize> {
        for i in 0..self.map.len() {
            if self.map[i].1 == num {
                return Some(i)
            }
        };
        None
    }

    /// # Details
    /// Method to get sheet number for a sheet name.
    /// 
    /// **Arguments:**
    /// - `name`: The sheet name.
    ///
    /// **Returns:**
    /// - `Some(usize)` containing num if name exists, else `None`.
    pub fn num_from_name(&self, name: &str) -> Option<usize> {
        for (curr_name, num) in &self.map {
            if curr_name == name {
                return Some(*num);
            }
        };
        None
    }

    /// # Details
    /// Method to get sheet name for a sheet number.
    /// 
    /// **Arguments:**
    /// - `num`: The sheet number.
    /// 
    /// **Returns:**
    /// - `Some(&String)` containing the name if num is valid, else `None`.
    pub fn name_from_num(&self, num: usize) -> Option<&String> {
        for (curr_name, number) in &self.map {
            if number == &num {
                return Some(curr_name);
            }
        };
        None
    }

    /// # Details
    /// Method to create a new sheet in the store
    /// 
    /// **Arguments:**
    /// - `name`: Name for the new sheet.
    /// - `cols`: Number of initial columns
    /// - `rows`: Number of initial rows
    /// 
    /// **Returns:**
    /// - `Some(usize)` containing the num of the new sheet if name does not conflict. If name is repeated then `None`.
    pub fn new_sheet(&mut self, name: &str, cols: usize, rows: usize) -> Option<usize> {
        for (curr_name, _num) in &self.map {
            if curr_name == name {
                return None;
            }
        };
        let new_num = self.data.len();
        let new_sheet_ref = RefCell::new(Sheet::new(new_num as u32, cols as u32, rows as u32));
        self.data.push(Rc::new(new_sheet_ref));
        self.map.push((String::from(name), new_num));
        Some(new_num)
    }

    /// # Details
    /// Method to add an existing sheet object to the store.
    /// 
    /// **Arguments:**
    /// - `name`: Name of the new sheet.
    /// - `sheet`: The sheet object to add.
    /// 
    /// **Returns:**
    /// - `Some(usize)` containing the num of the added sheet if name does not conflict. If name is repeated then `None`.
    pub fn add_sheet(&mut self, name: &str, sheet: Sheet) -> Option<usize> { //Assumes that sheet_idx would be same as data.len()
        for (curr_name, _num) in &self.map {
            if curr_name == name {
                return None;
            }
        };
        let new_num = self.data.len();
        let new_sheet_ref = RefCell::new(sheet);
        self.data.push(Rc::new(new_sheet_ref));
        self.map.push((String::from(name), new_num));
        Some(new_num)
    }

    /// # Details
    /// Method to remove a sheet from the store.
    /// 
    /// **Arguments:**
    /// - `name`: Name of the sheet to remove.
    /// 
    /// **Returns:**
    /// - `Some(usize)` containing the num of the removed sheet if a sheet by that name exists. If a sheet by the name does not exist then `None`.
    pub fn remove_sheet(&mut self, name: &str) -> Option<usize> {
        
        for i in 0..self.map.len() {
            if self.map[i].0 == name {
                let removed_num = self.map[i].1;
                self.data[removed_num] = Rc::new(RefCell::new(Sheet::new(999, 0, 0))); //NOTE: This very bad bad fix later
                self.map.remove(i);
                return Some(removed_num)
            }
        };
        None
    }
    pub fn rename_sheet(&mut self, name: &str, name_new: &str) -> Option<usize> {

        for i in 0..self.map.len() {
            if self.map[i].0 == name {
                let renamed_num = self.map[i].1;
                if self.num_from_name(name_new).is_none() {
                    self.map[i] = (String::from(name_new), renamed_num);
                    return Some(renamed_num)
                } else { return None }
            }
        };
        None
    }
}


struct Settings{
    // cell_width: u32,
    // formula_width: u32,
    undo_history_limit: u32
}
impl Settings {
    fn new() -> Self {
        Settings{
            // cell_width: 9,
            // formula_width: 15,
            undo_history_limit: 10
        }
    }
}
/// Imports a CSV file into a `Sheet`.
///
/// **Arguments:**
/// - `csv_name`: The name of the CSV file.
/// - `sheet_idx`: The index of the sheet.
///
/// **Returns:**
/// - `Ok(Sheet)` if the import is successful.
/// - `Err(String)` if an error occurs.
fn import_csv(csv_name: &str, sheet_idx: u32) -> Result<Sheet, String>
{

    let mut csv_data: Vec<Vec<String>> = vec![];
    if let Ok(mut rdr) = ReaderBuilder::new()
        .has_headers(false)
        .from_path(csv_name)
    {
        for result in rdr.records()
        {
            if let Ok(record) = result
            {
                let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                csv_data.push(row);
            }
            else
            {
                return Err("Error reading csv".to_string());
            }
        }
        let sheet: Sheet = Sheet::new(sheet_idx, csv_data[0].len() as u32, csv_data.len() as u32);
        for row in 0..csv_data.len()
        {
            for col in 0..csv_data[0].len()
            {
                if csv_data[row][col].is_empty()
                {
                    continue;
                }
                let mut cell = cell_operations::Cell::new(Addr{sheet: sheet_idx, row: row as u32, col: col as u32});
                let raw_val = csv_data[row][col].clone();

                if let Ok(val) = raw_val.parse::<i32>()
                {
                    cell.cell_func = Some(cell_operations::CellFunc::new(Expr::Integer(val)));
                    cell.valid = true;
                    cell.value = cell_operations::ValueType::IntegerValue(val);
                    cell.formula = raw_val;
                }
                else if let Ok(val) = raw_val.parse::<f64>()
                {
                    cell.cell_func = Some(cell_operations::CellFunc::new(Expr::Float(val)));
                    cell.valid = true;
                    cell.value = cell_operations::ValueType::FloatValue(val);
                    cell.formula = raw_val;

                }
                else if let Ok(val) = raw_val.parse::<bool>() 
                {
                    cell.cell_func = Some(cell_operations::CellFunc::new(Expr::Bool(val)));
                    cell.valid = true;
                    cell.value = cell_operations::ValueType::BoolValue(val);
                    cell.formula = raw_val;

                } 
                else 
                {
                    cell.valid = true;
                    cell.value = cell_operations::ValueType::String(raw_val.clone());
                    cell.cell_func = Some(cell_operations::CellFunc::new(Expr::String(raw_val.clone())));
                    cell.formula = raw_val;
                }
                
                sheet.data[col].borrow_mut().cells.push(Rc::new(RefCell::new(cell)));
            }
        }
        Ok(sheet)
    }
    else
    {
        Err("Error reading csv".to_string())
    }
}
/// Exports a `Sheet` to a CSV file.
///
/// **Arguments:**
/// - `sheet`: The sheet to export.
/// - `name`: The name of the output file.
///
/// **Returns:**
/// - `Ok(())` if the export is successful.
/// - `Err(String)` if an error occurs.
fn export_csv(sheet: &Sheet, name: &str) -> Result<(), String> 
{
    if let Ok(file) = File::create(String::from(name) + ".csv")
    {
        let mut writer = BufWriter::new(file);
        let mut csv_data : Vec<Vec<String>> = vec![];
        for col in &sheet.data
        {
            csv_data.push(vec![]);
            if col.borrow().cells.is_empty()
            {
                for _ in 0..sheet.rows
                {
                    if let Some(last) = csv_data.last_mut()
                    {
                        (*last).push("<EMPTY>".to_string());
                    }
                }
            }
            else
            {
                let curr_rows: usize = col.borrow().cells.len();
                let row: &Vec<Rc<RefCell<Cell>>> = &col.borrow().cells;
                if let Some(last) = csv_data.last_mut() {
                    row.iter()
                        .take(curr_rows)
                        .map(|cell_rc| cell_rc.borrow().value.clone().to_string())
                        .for_each(|value| last.push(value));
                }
                for _i in curr_rows..sheet.rows as usize
                {
                    if let Some(last) = csv_data.last_mut()
                    {
                        (*last).push("<EMPTY>".to_string());
                    }
                }
            }
        }
        for row in 0..csv_data[0].len()
        {
            for col in 0..csv_data.len()
            {
                // if csv_data[col][row] == "<EMPTY>"
                // {
                //     if let Ok(()) = write!(writer, "{}", "")
                //     {}
                //     else 
                //     {
                //         return Err("Error in writing csv".to_string());
                //     }
                // }
                // if let Ok(()) = write!(writer, "{}", csv_data[col][row])
                // {}
                // else 
                // {
                //     return Err("Error in writing csv".to_string());
                // }
                // if row != csv_data[0].len()-1
                // {
                //     if let Ok(()) = write!(writer, ",")
                //     {}
                //     else 
                //     {
                //         return Err("Error in writing csv".to_string());
                //     }
                // }
                if csv_data[col][row] != "<EMPTY>"
                {

                    if let Ok(()) = write!(writer, "{}", csv_data[col][row])
                    {}
                    else 
                    {
                        return Err("Error in writing csv".to_string());
                    }
                }
                if col != csv_data.len()-1
                {
                    if let Ok(()) = write!(writer, ",")
                    {}
                    else 
                    {
                        return Err("Error in writing csv".to_string());
                    }
                }

            }
            if let Ok(()) = writeln!(writer)
            {
            }
            else 
            {
                return Err("Error in writing csv".to_string());
            }
        }

        Ok(())
    }
    else 
    {
        Err("Error in creating csv".to_string())
    }
}

/// Copies the value of one cell to another.
///
/// **Arguments:**
/// - `addr1`: The address of the source cell.
/// - `addr2`: The address of the destination cell.
/// - `sheets`: A reference to the list of sheets.
fn copy_cell_value(addr1:Addr, addr2:Addr, sheets: &[Rc<RefCell<Sheet>>])
{
    let sheet_ref = &sheets[addr1.sheet as usize];
    let sheet = sheet_ref.borrow();
    let column_ref = &sheet.data[addr1.col as usize];
    let column = column_ref.borrow();
    let cell_rc = Rc::clone(&column[addr1.row as usize]);
    // drop(column);
    let cell = cell_rc.borrow();
    let value = cell.value.clone();
    
    let sheet_ref2 = &sheets[addr2.sheet as usize];
    let sheet2 = sheet_ref2.borrow();
    let column_ref2 = &sheet2.data[addr2.col as usize];
    let column2 = column_ref2.borrow();
    let cell_rc2 = Rc::clone(&column2[addr2.row as usize]);
    // drop(column);
    let mut cell2 = cell_rc2.borrow_mut();
    
    
    match value.clone() 
    {
        ValueType::IntegerValue(val) => {
            cell2.cell_func = Some(cell_operations::CellFunc::new(Expr::Integer(val)));
            cell2.formula = val.to_string();
        }
        ValueType::FloatValue(val) => {
            cell2.cell_func = Some(cell_operations::CellFunc::new(Expr::Float(val)));
            cell2.formula = val.to_string();
        }
        ValueType::BoolValue(val) => {
            cell2.cell_func = Some(cell_operations::CellFunc::new(Expr::Bool(val)));
            cell2.formula = val.to_string();
        }
        ValueType::String(val) => {
            cell2.cell_func = Some(cell_operations::CellFunc::new(Expr::String(val.clone())));
            cell2.formula = val.clone();
        }
    }
    cell2.value = value;
}

/// Copies the values of a range of cells to another range.
///
/// **Arguments:**
/// - `addr1`: The top-left address of the source range.
/// - `addr2`: The bottom-right address of the source range.
/// - `addr3`: The top-left address of the destination range.
/// - `sheets`: A reference to the list of sheets.
fn copy_range_value(addr1:Addr, addr2:Addr, addr3: Addr, sheets: &[Rc<RefCell<Sheet>>])
{
    let mut m = 0;
    for (n,i) in (addr1.row..=addr2.row).enumerate()
    {
        for j in addr1.col..=addr2.col
        {
            copy_cell_value(Addr{sheet: addr1.sheet, row: i, col: j}, Addr{sheet: addr3.sheet, row: addr3.row + n as u32, col: addr3.col + m}, sheets);
            m += 1;
        }

        m = 0;
    }
}
/// Copies the formula of one cell to another.
///
/// **Arguments:**
/// - `addr1`: The address of the source cell.
/// - `addr2`: The address of the destination cell.
/// - `sheets`: A mutable reference to the list of sheets.
///
/// **Returns:**
/// - `Ok(())` if the operation is successful.
/// - `Err(String)` if an error occurs.
fn copy_cell_function(addr1:Addr, addr2:Addr, sheets: &mut [Rc<RefCell<Sheet>>]) -> Result<(),String>
{
    let sheet_ref = Rc::clone(&sheets[addr1.sheet as usize]);
    let sheet = sheet_ref.borrow();
    let column_ref = &sheet.data[addr1.col as usize];
    let column = column_ref.borrow();
    let cell_rc = Rc::clone(&column[addr1.row as usize]);
    // drop(column);
    let cell = cell_rc.borrow();
    let func = cell.cell_func.clone();
    let formula = cell.formula.clone();
    drop(cell);
    let sheet_ref2 = Rc::clone(&sheets[addr2.sheet as usize]);
    let sheet2 = sheet_ref2.borrow();
    let column_ref2 = &sheet2.data[addr2.col as usize];
    let column2 = column_ref2.borrow();
    let cell_rc2 = Rc::clone(&column2[addr2.row as usize]);
    // drop(column);
    let mut cell2 = cell_rc2.borrow_mut();
    let old_func = cell2.cell_func.clone();
    cell2.cell_func = func.clone();
    if let Some(func) = func.clone()
    {
        let exp = update_cell_func(func.expression, addr1.sheet, addr2.sheet);
        cell2.cell_func = Some(CellFunc::new(exp));
    }
    cell2.formula = formula;
    drop(cell2);
    evaluate(sheets, &addr2, &old_func)

}
/// Copies the formulas of a range of cells to another range.
///
/// **Arguments:**
/// - `addr1`: The top-left address of the source range.
/// - `addr2`: The bottom-right address of the source range.
/// - `addr3`: The top-left address of the destination range.
/// - `sheets`: A mutable reference to the list of sheets.
///
/// **Returns:**
/// - `Ok(())` if the operation is successful.
/// - `Err(String)` if an error occurs.
fn copy_range_function(addr1:Addr, addr2:Addr, addr3: Addr, sheets: &mut [Rc<RefCell<Sheet>>]) -> Result<(),String>
{
    let mut m = 0;
    for (n,i) in (addr1.row..=addr2.row).enumerate()
    {
        for j in addr1.col..=addr2.col
        {
            copy_cell_function(Addr{sheet: addr1.sheet, row: i, col: j}, Addr{sheet: addr3.sheet, row: addr3.row + n as u32, col: addr3.col + m}, sheets)?;
            m += 1;
        }
        m=0;
    }
    Ok(())
    
}

/// Autofills a range of cells in an arithmetic progression (AP).
///
/// **Arguments:**
/// - `start_addr`: The starting address of the range.
/// - `end_addr`: The ending address of the range.
/// - `sheets`: A mutable reference to the list of sheets.
///
/// **Returns:**
/// - `Ok(())` if the operation is successful.
/// - `Err(String)` if an error occurs.
fn autofill_ap(start_addr: Addr, end_addr: Addr, sheets: &mut [Rc<RefCell<Sheet>>]) -> Result<(), String> {
    let sheet_ref: &Rc<RefCell<Sheet>> = &sheets[start_addr.sheet as usize];
    let sheet: std::cell::Ref<'_, Sheet> = sheet_ref.borrow();
    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[start_addr.col as usize];
    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
    let cell_rc: Rc<RefCell<Cell>> = Rc::clone(&column[start_addr.row as usize]);
    let cell1: std::cell::Ref<'_, Cell> = cell_rc.borrow();

    if start_addr.col == end_addr.col {
        // Autofill in the same column
        let cell2_rc = Rc::clone(&column[(start_addr.row + 1) as usize]);
        let cell2 = cell2_rc.borrow();
        match (cell1.value.clone(), cell2.value.clone()) {
            (ValueType::IntegerValue(val1), ValueType::IntegerValue(val2)) => {
                let common_diff = val2 - val1;
                for row in start_addr.row + 2..=end_addr.row {
                    let cell_rc = Rc::clone(&column[row as usize]);
                    let mut cell = cell_rc.borrow_mut();
                    let val = val1 + common_diff * (row - start_addr.row) as i32;
                    cell.value = ValueType::IntegerValue(val);
                    cell.cell_func = Some(CellFunc::new(Expr::Integer(val)));
                    cell.formula = val.to_string();
                }
            }
            (ValueType::FloatValue(val1), ValueType::FloatValue(val2)) => {
                let common_diff = val2 - val1;
                for row in start_addr.row + 2..=end_addr.row {
                    let cell_rc = Rc::clone(&column[row as usize]);
                    let mut cell = cell_rc.borrow_mut();
                    let val = val1 + common_diff * (row - start_addr.row) as f64;
                    cell.value = ValueType::FloatValue(val);
                    cell.cell_func = Some(CellFunc::new(Expr::Float(val)));
                    cell.formula = val.to_string();
                }
            }
            (_, _) => {
                return Err("AP autofill cannot be used with Booleans or Strings".to_string());
            }
        }
    } else if start_addr.row == end_addr.row {
        // Autofill in the same row
        let column_ref: &RefCell<cell_operations::Column> = &sheet.data[(start_addr.col + 1) as usize];
        let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
        let cell_rc: Rc<RefCell<Cell>> = Rc::clone(&column[start_addr.row as usize]);
        let cell2: std::cell::Ref<'_, Cell> = cell_rc.borrow();
        match (cell1.value.clone(), cell2.value.clone()) {
            (ValueType::IntegerValue(val1), ValueType::IntegerValue(val2)) => {
                let common_diff = val2 - val1;
                for col in start_addr.col + 2..=end_addr.col {
                    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[col as usize];
                    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
                    let cell_rc = Rc::clone(&column[start_addr.row as usize]);
                    let mut cell3 = cell_rc.borrow_mut();
                    let val = val1 + common_diff * (col - start_addr.col) as i32;
                    cell3.value = ValueType::IntegerValue(val);
                    cell3.cell_func = Some(CellFunc::new(Expr::Integer(val)));
                    cell3.formula = val.to_string();
                }
            }
            (ValueType::FloatValue(val1), ValueType::FloatValue(val2)) => {
                let common_diff = val2 - val1;
                for col in start_addr.col + 2..=end_addr.col {
                    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[col as usize];
                    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
                    let cell_rc = Rc::clone(&column[start_addr.row as usize]);
                    let mut cell3 = cell_rc.borrow_mut();
                    let val = val1 + common_diff * (col - start_addr.col) as f64;
                    cell3.value = ValueType::FloatValue(val);
                    cell3.cell_func = Some(CellFunc::new(Expr::Float(val)));
                    cell3.formula = val.to_string();
                }
            }
            (_, _) => {
                return Err("AP autofill cannot be used with Booleans or Strings".to_string());
            }
        }
    } else {
        return Err("Cannot autofill sequence in 2-D range".to_string());
    }
    Ok(())
}

/// Autofills a range of cells in a geometric progression (GP).
///
/// **Arguments:**
/// - `start_addr`: The starting address of the range.
/// - `end_addr`: The ending address of the range.
/// - `sheets`: A mutable reference to the list of sheets.
///
/// **Returns:**
/// - `Ok(())` if the operation is successful.
/// - `Err(String)` if an error occurs.
fn autofill_gp(start_addr: Addr, end_addr: Addr, sheets: &mut [Rc<RefCell<Sheet>>]) -> Result<(), String> {
    let sheet_ref: &Rc<RefCell<Sheet>> = &sheets[start_addr.sheet as usize];
    let sheet: std::cell::Ref<'_, Sheet> = sheet_ref.borrow();
    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[start_addr.col as usize];
    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
    let cell_rc: Rc<RefCell<Cell>> = Rc::clone(&column[start_addr.row as usize]);
    let cell1: std::cell::Ref<'_, Cell> = cell_rc.borrow();

    if let ValueType::IntegerValue(0) = cell1.value {
        return Err("GP cannot start with Integral Value 0".to_string());
    }
    if let ValueType::FloatValue(0.0) = cell1.value {
        return Err("GP cannot start with Float Value 0".to_string());
    }

    if start_addr.col == end_addr.col {
        // Autofill in the same column
        let cell2_rc = Rc::clone(&column[(start_addr.row + 1) as usize]);
        let cell2 = cell2_rc.borrow();
        match (cell1.value.clone(), cell2.value.clone()) {
            (ValueType::IntegerValue(val1), ValueType::IntegerValue(val2)) => {
                let common_ratio = val2 as f64 / val1 as f64;
                for row in start_addr.row + 2..=end_addr.row {
                    let cell_rc = Rc::clone(&column[row as usize]);
                    let mut cell = cell_rc.borrow_mut();
                    let val = val1 as f64 * common_ratio.powf((row - start_addr.row) as f64);
                    cell.value = ValueType::IntegerValue(val as i32);
                    cell.cell_func = Some(CellFunc::new(Expr::Integer(val as i32)));
                    cell.formula = val.to_string();
                }
            }
            (ValueType::FloatValue(val1), ValueType::FloatValue(val2)) => {
                let common_ratio = val2 / val1;
                for row in start_addr.row + 2..=end_addr.row {
                    let cell_rc = Rc::clone(&column[row as usize]);
                    let mut cell = cell_rc.borrow_mut();
                    let val = val1 * common_ratio.powf((row - start_addr.row) as f64);
                    cell.value = ValueType::FloatValue(val);
                    cell.cell_func = Some(CellFunc::new(Expr::Float(val)));
                    cell.formula = val.to_string();
                }
            }
            (_, _) => {
                return Err("GP autofill cannot be used with Booleans or Strings".to_string());
            }
        }
    } else if start_addr.row == end_addr.row {
        // Autofill in the same row
        let column_ref: &RefCell<cell_operations::Column> = &sheet.data[(start_addr.col + 1) as usize];
        let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
        let cell_rc: Rc<RefCell<Cell>> = Rc::clone(&column[start_addr.row as usize]);
        let cell2: std::cell::Ref<'_, Cell> = cell_rc.borrow();
        match (cell1.value.clone(), cell2.value.clone()) {
            (ValueType::IntegerValue(val1), ValueType::IntegerValue(val2)) => {
                let common_ratio = val2 as f64 / val1 as f64;
                for col in start_addr.col + 2..=end_addr.col {
                    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[col as usize];
                    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
                    let cell_rc = Rc::clone(&column[start_addr.row as usize]);
                    let mut cell3 = cell_rc.borrow_mut();
                    let val = val1 as f64 * common_ratio.powf((col - start_addr.col) as f64);
                    cell3.value = ValueType::IntegerValue(val as i32);
                    cell3.cell_func = Some(CellFunc::new(Expr::Integer(val as i32)));
                    cell3.formula = val.to_string();
                }
            }
            (ValueType::FloatValue(val1), ValueType::FloatValue(val2)) => {
                let common_ratio = val2 / val1;
                for col in start_addr.col + 2..=end_addr.col {
                    let column_ref: &RefCell<cell_operations::Column> = &sheet.data[col as usize];
                    let column: std::cell::Ref<'_, cell_operations::Column> = column_ref.borrow();
                    let cell_rc = Rc::clone(&column[start_addr.row as usize]);
                    let mut cell3 = cell_rc.borrow_mut();
                    let val = val1 * common_ratio.powf((col - start_addr.col) as f64);
                    cell3.value = ValueType::FloatValue(val);
                    cell3.cell_func = Some(CellFunc::new(Expr::Float(val)));
                    cell3.formula = val.to_string();
                }
            }
            (_, _) => {
                return Err("GP autofill cannot be used with Booleans or Strings".to_string());
            }
        }
    } else {
        return Err("Cannot autofill sequence in 2-D range".to_string());
    }
    Ok(())
}
/// Invalidates the children of a given cell by marking them as invalid when we remove a spreadsheet.
///
/// **Arguments:**
/// - `sheets`: A mutable reference to the list of sheets.
/// - `cell_addr`: The address of the cell whose children need to be invalidated.
fn invalidate_children(sheets: &mut [Rc<RefCell<Sheet>>], cell_addr: Addr) 
{
    let Some(sheet_ref) = sheets.get(cell_addr.sheet as usize) else {
        return;
    };
    let sheet = sheet_ref.borrow();
    let Some(column_ref) = sheet.data.get(cell_addr.col as usize) else {
        return;
    };
    let column = column_ref.borrow();
    let Some(cell_ref) = column.cells.get(cell_addr.row as usize) else {
        return;
    };
    let cell = cell_ref.borrow();

    for child_addr in cell.children.iter() {
        let Some(new_sheet_ref) = sheets.get(child_addr.sheet as usize) else {
            continue;
        };
        let new_sheet = new_sheet_ref.borrow();
        let Some(new_column_ref) = new_sheet.data.get(child_addr.col as usize) else {
            continue;
        };
        let new_column = new_column_ref.borrow();
        let Some(new_cell_ref) = new_column.cells.get(child_addr.row as usize) else {
            continue;
        };
        let mut new_cell = new_cell_ref.borrow_mut();
        new_cell.valid = false;
    }
}

/// Undoes the last operation performed on the spreadsheet.
///
/// **Arguments:**
/// - `sheets`: A mutable reference to the list of sheets.
/// - `undo_history`: A mutable reference to the undo history stack.
/// - `redo_history`: A mutable reference to the redo history stack.
/// - `settings`: A reference to the application settings.
///
/// **Returns:**
/// - `Ok((Addr, Option<CellFunc>))` if the undo operation is successful.
/// - `Err(String)` if there is no operation to undo or an error occurs.
type UndoEntry = (
    bool,               // something like "is_dirty"
    Addr,               // address
    Option<CellFunc>,   // old function
    Option<String>,     // old formula (optional)
    bool,               // is_overwritten
    Option<CellFunc>,   // new function
    Option<String>,     // new formula (optional)
    bool                // recompute_needed?
);
type RedoEntry = (
    Addr,               // address
    Option<CellFunc>,   // old function
    Option<String>,     // old formula
    bool,               // is_overwritten
    Option<CellFunc>,   // new function
    Option<String>,     // new formula
    bool                // recompute_needed
);
fn undo(sheets: &mut [Rc<RefCell<Sheet>>],undo_history: &mut Vec<UndoEntry>, redo_history: &mut Vec<RedoEntry>, settings: &Settings) -> Result<(Addr,Option<CellFunc>),String>
{
    let temp = undo_history.last();
    if temp.is_none()
    {
        return Err("Already at the earliest change".to_string());
    }
    let (undoable, addr, old_func, old_formula, old_valid, new_func, new_formula, new_valid) = temp.unwrap().clone();
    let sheet_ref = &sheets[addr.sheet as usize];
    let sheet = sheet_ref.borrow();
    let column_ref = &sheet.data[addr.col as usize];
    let column = column_ref.borrow();
    let cell_rc = Rc::clone(&column[addr.row as usize]);
    // drop(column);
    let mut cell = cell_rc.borrow_mut();
    let old_function = new_func.clone();
    if !undoable
    {
        return Err("Cannot undo non-assignment operation".to_string());
    }
    if let Some(func) = old_func.clone()
    {
        cell.cell_func = Some(func);
    }
    else
    {
        cell.cell_func = Some(CellFunc::new(Expr::Integer(0)));
        cell.formula = "0".to_string();
        cell.value = ValueType::IntegerValue(0);
        
    }
    if let Some(formula) = old_formula.clone()
    {
        cell.formula = formula.clone();
    }
    else
    {
        cell.formula = "~".to_string();
    }
    cell.valid = old_valid;
    redo_history.push((addr.clone(),old_func.clone(), old_formula.clone(), old_valid, new_func.clone(), new_formula.clone(), new_valid));
    undo_history.pop();
    if redo_history.len() > settings.undo_history_limit as usize
    {
        redo_history.remove(0);
    }

    Ok((cell.addr.clone(),old_function))
}

/// Redoes the last undone operation on the spreadsheet.
///
/// **Arguments:**
/// - `sheets`: A mutable reference to the list of sheets.
/// - `undo_history`: A mutable reference to the undo history stack.
/// - `redo_history`: A mutable reference to the redo history stack.
/// - `settings`: A reference to the application settings.
///
/// **Returns:**
/// - `Ok((Addr, Option<CellFunc>))` if the redo operation is successful.
/// - `Err(String)` if there is no operation to redo or an error occurs.
fn redo(sheets: &mut [Rc<RefCell<Sheet>>], undo_history: &mut Vec<UndoEntry>, redo_history: &mut Vec<RedoEntry>, settings: &Settings) -> Result<(Addr,Option<CellFunc>),String>
{
    let temp = redo_history.last();
    if temp.is_none()
    {
        return Err("Already at the latest change".to_string());
    }
    // assert!(index < history.len() as i32);
    let (addr, old_func, old_formula, old_valid, new_func, new_formula, new_valid) = temp.unwrap().clone();
    let sheet_ref = &sheets[addr.sheet as usize];
    let sheet = sheet_ref.borrow();
    let column_ref = &sheet.data[addr.col as usize];
    let column = column_ref.borrow();
    let cell_rc: Rc<RefCell<Cell>> = Rc::clone(&column[addr.row as usize]);
    // drop(column);
    let mut cell = cell_rc.borrow_mut();
    let old_function = old_func.clone();
    if let Some(func) = new_func.clone()
    {
        cell.cell_func = Some(func);
    }
    else
    {
        cell.cell_func = None;
    }
    if let Some(formula) = old_formula.clone()
    {
        cell.formula = formula.clone();
    }
    else
    {
        cell.formula = "~".to_string();
    }
    cell.valid = new_valid;
    undo_history.push((true, addr.clone(),old_func.clone(), old_formula.clone(), old_valid, new_func.clone(), new_formula.clone(), new_valid));
    redo_history.pop();
    if undo_history.len() > settings.undo_history_limit as usize
    {
        undo_history.remove(0);
    }
    Ok((cell.addr.clone(),old_function))
}

/// Duplicates a sheet by creating a copy of it.
///
/// **Arguments:**
/// - `sheets`: A mutable reference to the list of sheets.
/// - `sheet_number`: The index of the sheet to duplicate.
///
/// **Returns:**
/// - `Ok(Sheet)` if the duplication is successful.
/// - `Err(String)` if the sheet does not exist or an error occurs.
fn duplicate_sheet(sheets: &mut Vec<Rc<RefCell<Sheet>>>, sheet_number: usize) -> Result<Sheet,String>  // sheet_number and sheet_name correspond to the old sheet that has been copied
{
    if sheet_number >= sheets.len()
    {
        return Err("The sheet you are trying to copy does not exit".to_string());
    }
    let mut new_sheet = sheets[sheet_number].borrow().clone();
    // new_sheet.sheet_name = sheet_name;
    new_sheet.sheet_idx = sheets.len() as u32;
    

    for col in new_sheet.data.iter()
    {
        for row in col.borrow().cells.iter()
        {
            let mut cell: std::cell::RefMut<'_, Cell> = row.borrow_mut();
            cell.addr.sheet = sheet_number as u32;
            let dep_list_ref: Option<&CellFunc> = cell.cell_func.as_ref();
            if let Some(dep_listt) = dep_list_ref
            {
                let dep_list: Vec<ParentType> = dep_listt.expression.get_dependency_list();
                for dep in dep_list.iter()
                {
                    match dep {
                        ParentType::Single(a_1) => {
                            let sheet_ref =&(*sheets)[a_1.sheet as usize];
                            let sheet = sheet_ref.borrow();
                            let column_ref = &sheet.data[a_1.col as usize];
                            let column = column_ref.borrow();
                            let cell_rc = Rc::clone(&column[a_1.row as usize]);
                            // drop(column);
                            let mut parent_cell = cell_rc.borrow_mut();
                            parent_cell.children.insert(cell.addr.clone());
                            if a_1.sheet == sheet_number as u32 
                            {
                                let old_addr = Addr{sheet: sheet_number as u32, row: cell.addr.row, col: cell.addr.col};
                                parent_cell.children.remove(&old_addr);
                            }
                        }
                        ParentType::Range(a_1, a_2) => {
                            for i in a_1.col..=a_2.col
                            {
                                for j in a_1.row..=a_2.row
                                {
                                    let sheet_ref =&(*sheets)[a_1.sheet as usize];
                                    let sheet = sheet_ref.borrow();
                                    let column_ref = &sheet.data[i as usize];
                                    let column = column_ref.borrow();
                                    let cell_rc = Rc::clone(&column[j as usize]);
                                    // drop(column);
                                    let mut parent_cell = cell_rc.borrow_mut();
                                    parent_cell.children.insert(cell.addr.clone());
                                    if a_1.sheet == sheet_number as u32 
                                    {
                                        let old_addr = Addr{sheet: sheet_number as u32, row: j, col: i };
                                        parent_cell.children.remove(&old_addr);
                                    }
                                }
                            }
                        },
                    }
                }

                
            }
            else
            {
                continue;
            }


            // idhar hum bache change karenge
            let mut toinsert = vec![];
            for address in cell.children.iter()
            {
                if address.sheet == sheet_number as u32
                {
                    toinsert.push(address.clone());
                }
            }
            
            cell.children.clear();

            for addr in toinsert.iter()
            {
                let new_addr = Addr{sheet: new_sheet.sheet_idx, row: addr.row, col: addr.col};
                cell.children.insert(new_addr);
            }
            // aur idhar cell_func change karenge
            if let Some(func) = cell.cell_func.clone()
            { 
                let temp = update_cell_func(func.expression,sheet_number as u32, new_sheet.sheet_idx);            
                cell.cell_func = Some(CellFunc::new(temp));
            }
            
        }
    }
    // sheets.push(Rc::new(RefCell::new(new_sheet)));
    Ok(new_sheet)

}

/// Updates the cell function to reflect changes in the sheet index, used when we duplicate a sheet.
///
/// **Arguments:**
/// - `exp`: The expression to update.
/// - `sheet_num`: The original sheet index.
/// - `sheet_idx`: The new sheet index.
///
/// **Returns:**
/// - The updated expression.
fn update_cell_func(exp: Expr, sheet_num: u32, sheet_idx: u32) -> Expr
{
    match exp 
    {
        Expr::Bool(val) => Expr::Bool(val),
        Expr::Float(val) => Expr::Float(val),
        Expr::Integer(val) => Expr::Integer(val),
        Expr::String(val) => Expr::String(val),
        Expr::Wildcard => Expr::Wildcard,
        Expr::MonoOp(a,b) => 
        {
            let expr = update_cell_func(*b, sheet_num, sheet_idx);
            let exprbox = Box::new(expr);
            Expr::MonoOp(a,exprbox)
        },
        Expr::BinOp(a,b,c) =>
        {
            let expr1 = update_cell_func(*b, sheet_num, sheet_idx);
            let expr2 = update_cell_func(*c, sheet_num, sheet_idx);
            let exprbox1 = Box::new(expr1);
            let exprbox2 = Box::new(expr2);
            Expr::BinOp(a,exprbox1,exprbox2)
        },
        Expr::InfixOp(a,b,c) =>
        {
            let expr1 = update_cell_func(*a, sheet_num, sheet_idx);
            let expr2 = update_cell_func(*c, sheet_num, sheet_idx);
            let exprbox1 = Box::new(expr1);
            let exprbox2 = Box::new(expr2);
            Expr::InfixOp(exprbox1, b, exprbox2)
        },
        Expr::TernaryOp(a,b,c,d) =>
        {
            let expr1 = update_cell_func(*b, sheet_num, sheet_idx);
            let expr2 = update_cell_func(*c, sheet_num, sheet_idx);
            let expr3 = update_cell_func(*d, sheet_num, sheet_idx);
            let exprbox1 = Box::new(expr1);
            let exprbox2 = Box::new(expr2);
            let exprbox3 = Box::new(expr3);
            Expr::TernaryOp(a,exprbox1, exprbox2, exprbox3)
        },
        Expr::RangeOp{op, start, end, cond} =>
        {
            if sheet_num == start.sheet
            {
                let new_start = Addr{sheet:sheet_idx, row:start.row,col:start.col};
                let new_end = Addr{sheet:sheet_idx, row:end.row,col:end.col};
                let new_cond = update_cell_func(*cond, sheet_num, sheet_idx);
                Expr::RangeOp{op, start: new_start, end: new_end, cond: Box::new(new_cond)}
            }
            else 
            {
               Expr::RangeOp { op, start, end,cond }
            }
        }
        Expr::Cell(addr) => 
        {
            if addr.sheet == sheet_num
            {
                let new_addr = Addr{sheet:sheet_idx, row: addr.row, col: addr.col};
                Expr::Cell(new_addr)
            }
            else
            {
                Expr::Cell(addr)
            }
        }
        
    }
}

// fn display_sheet(col: u32, row: u32, sheet: &Sheet, settings: &Settings, showformulas: bool)
// {
//     let row_max = cmp::min(row+10, sheet.rows);
//     let col_max = cmp::min(col+10, sheet.columns);
//     let width = settings.cell_width as usize;
//     let formula_width = settings.formula_width as usize;
    
//     print!("      ");
//     for i in col..col_max {
//         let mut curr = String::new();
//         let mut curr_col = i + 1;
//         while curr_col > 0
//         {

//             curr.push(((b'A') + ((curr_col-1) % 26) as u8) as char);
            
//             curr_col -= 1;
//             curr_col /= 26;
//         }
//         print!("{:>width$}", curr.chars().rev().collect::<String>());
//     }
//     println!();
//     for i in row..row_max {
//         print!("{:>width$}", i+1);
//         for j in col..col_max {

//             if showformulas
//             {
//                 let colref = sheet.data[j as usize].borrow();
//                 if i as usize >= colref.cells.len() {
//                     print!("{:>width$}", "~");
//                     continue;
//                 }
//                 else
//                 {
//                     let cell = colref.cells[i as usize].borrow();
//                     print!("{:>formula_width$}", cell.formula);
//                 } 
//             }
//             else
//             {
//                 let colref = sheet.data[j as usize].borrow();
//                 if i as usize >= colref.cells.len()
//                 {
//                     print!("{:>width$}", "~");
//                     continue
//                 } 
//                 else
//                 {
//                     let cell = colref.cells[i as usize].borrow();
//                     if cell.valid {
//                         let val =  &cell.value;
//                         match val {
//                             ValueType::BoolValue(b) => print!("{:>width$}", b),
//                             ValueType::IntegerValue(x) => print!("{:>width$}", x),
//                             ValueType::FloatValue(n) => print!("{:>width$.2}", n, width = width),
//                             ValueType::String(s) => print!("{:>width$}", s),
//                         }
//                     }
//                     else {
//                         print!("{:>width$}", "~ERR")
//                     }
//                 }  
//             }
//         }
//         println!()
//     }
// }


fn main() -> Result<(), Box<dyn std::error::Error>>{

    let r: u32 = std::env::args().nth(1)
        .expect("Row number not entered (First arg missing)")
        .parse().expect("Invalid input for Row number (First arg)");
    
    let c: u32 = std::env::args().nth(2)
        .expect("Column number not entered (Second arg missing)")
        .parse().expect("Invalid input for Column number (Second arg)");

    //Graphics Initialisation
    enable_raw_mode()?; //NOTE: Source of panic.
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?; //NOTE: Source of panic.

    let styleguide = StyleGuide::new();
    let mut input_widget = TextInputWidget::new();
    let mut history_widget = HistoryWidget::new();
    let mut jump_to_last = true;
    let mut tabs_widget = TabsWidget{tabs: vec![], index: 0};
    let mut celldetails_widget = CellDetailsWidget{};
    let mut outputs_widget: OutputsWidget = OutputsWidget::new();
    let mut show_graph: bool = false;


    let mut sheetstore = SheetStorage::new();
    sheetstore.new_sheet("sheet0", c as usize, r as usize);

    // let mut sheets: Vec<Rc<RefCell<Sheet>>> = vec![Rc::new(RefCell::new(Sheet::new(0, String::from("sheet0"), c, r)))];

    let mut exit : bool = false;

    let mut curr_col: usize= 0;
    let mut curr_row: usize = 0;
    let mut curr_sheet_number: usize = 0;
    // let mut show_window: bool = true;
    let mut last_err_msg = String::from("ok");
    let settings = Settings::new();
    // let mut last_time = 0;
    let mut command_history_index = 0;
    let mut undo_history: Vec<UndoEntry> = vec![];
    let mut redo_history: Vec<RedoEntry> = vec![];

    'mainloop: while !exit {
        // let mut start = Instant::now();
        let _ = terminal.draw(|frame| {
            let [tabs_area, table_details_area, history_output_area, input_area] = Layout::vertical([Min(3), Percentage(60), Percentage(40), Min(3)]).areas(frame.area());
            let [table_area, detail_area] = Layout::horizontal([Percentage(75), Percentage(25)]).areas(table_details_area);
            let [history_area, output_area] = Layout::horizontal([Percentage(70), Percentage(30)]).areas(history_output_area);
            tabs_widget.tabs = sheetstore.list_names();
            tabs_widget.index = sheetstore.list_index_from_num(curr_sheet_number).unwrap();  //NOTE: Source of panic.
            tabs_widget.draw(tabs_area, frame, &styleguide);
            celldetails_widget.draw(curr_col, curr_row, &sheetstore.data[curr_sheet_number].borrow(),detail_area, frame, &styleguide);
            draw_table(curr_col, curr_row, &sheetstore.data[curr_sheet_number].borrow(), "Spreadsheet", table_area, frame, &styleguide);

            if jump_to_last {
                history_widget.scroll_amt = history_widget.history.len().saturating_sub(history_area.height.saturating_sub(2) as usize);
            }
            history_widget.draw(history_area, frame, &styleguide);
            input_widget.draw(input_area, frame, &styleguide);

            if show_graph {
                if sheetstore.name_from_num(outputs_widget.sheetnum).is_none() {
                    outputs_widget.draw_text( String::from("Referred sheet no longer valid."), output_area, frame, &styleguide);
                }
                else {
                    outputs_widget.draw_chart( &sheetstore.data[outputs_widget.sheetnum].borrow(), output_area, frame, &styleguide, false);
                }
            }
            else {
                outputs_widget.draw_idle(output_area, frame, &styleguide);
            }
        });
        let mut inp: String = String::new();

        if let Event::Key(key) = event::read()? {
            match input_widget.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        input_widget.input_mode = InputMode::Editing;
                        continue 'mainloop
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('w') if key.kind == KeyEventKind::Press => {
                        // let _curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                        curr_row = curr_row.saturating_sub(1);
                    }
                    KeyCode::Char('s') if key.kind == KeyEventKind::Press => {
                        let curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                        curr_row = cmp::min(curr_row.saturating_add(1) , curr_sheet.rows.saturating_sub(1) as usize)
                    }
                    KeyCode::Char('d') if key.kind == KeyEventKind::Press => {
                        let curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                        curr_col =  cmp::min(curr_col.saturating_add(1) , curr_sheet.columns.saturating_sub(1) as usize);
                    }
                    KeyCode::Char('a') if key.kind == KeyEventKind::Press => {
                        // let _curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                        curr_col = curr_col.saturating_sub(1);
                    }
                    KeyCode::Up if key.kind == KeyEventKind::Press=> {
                        history_widget.scroll_amt = history_widget.scroll_amt.saturating_sub(1);
                        jump_to_last = false;
                    }
                    KeyCode::Down if key.kind == KeyEventKind::Press => {
                        history_widget.scroll_amt = cmp::min(history_widget.scroll_amt.saturating_add(1), history_widget.history.len().saturating_sub(15));
                        jump_to_last = false;
                    }
                    KeyCode::Right if key.kind == KeyEventKind::Press => {
                        curr_sheet_number = sheetstore.map[(sheetstore.list_index_from_num(curr_sheet_number).expect("curr_sheet_number somehow no longer valid").saturating_add(1))%sheetstore.map.len()].1;
                        curr_col = 0;
                        curr_row = 0;
                    }
                    KeyCode::Left if key.kind == KeyEventKind::Press => {
                        curr_sheet_number = sheetstore.map[(sheetstore.list_index_from_num(curr_sheet_number).expect("curr_sheet_number somehow no longer valid").saturating_sub(1))%sheetstore.map.len()].1;
                        curr_col = 0;
                        curr_row = 0;
                    }
                    _ => { continue 'mainloop }
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => { 
                        inp = input_widget.input.clone();
                        input_widget.reset_cursor();
                        input_widget.input.clear();
                        command_history_index = 0;
                        },
                    KeyCode::Char(to_insert) => { input_widget.enter_char(to_insert); continue 'mainloop} ,
                    KeyCode::Backspace => { input_widget.delete_char(); continue 'mainloop }
                    KeyCode::Left => { input_widget.move_cursor_left(); continue 'mainloop }
                    KeyCode::Up => { 
                        command_history_index = cmp::min(command_history_index+1, history_widget.history.len().saturating_sub(1)); //Subtracting 1 because history widget is initialised with good luck
                        if command_history_index != 0 {
                            input_widget.input = history_widget.history[history_widget.history.len() - command_history_index].0.clone();
                            input_widget.reset_cursor();
                        }
                        continue 'mainloop }
                    KeyCode::Down => { 
                        if command_history_index == 1 {
                            command_history_index = 0;
                            input_widget.input = String::new();
                            input_widget.reset_cursor();
                        }
                        else {
                            command_history_index = command_history_index.saturating_sub(1);
                            if command_history_index != 0 {
                                input_widget.input = history_widget.history[history_widget.history.len() - command_history_index].0.clone();
                                input_widget.reset_cursor();
                            }
                        }
                        continue 'mainloop }
                    KeyCode::Right => { input_widget.move_cursor_right(); continue 'mainloop }
                    KeyCode::Esc => { input_widget.input_mode = InputMode::Normal; continue 'mainloop }
                    _ => { continue 'mainloop }
                },
                InputMode::Editing => { continue 'mainloop }
            }
        }
        else { continue 'mainloop};







        // print!("[{}.0] ", last_time);
        // print!("({}) >> ", last_err_msg);
        // io::stdout().flush().unwrap();

        let ast;
        let dep_vec;

        if inp.is_empty() {
            continue 'mainloop
        }
        if inp.starts_with(':') {
            let inp_smol = inp.chars().skip(1).collect::<String>();
            let lexer = tokenscmds::Token::lexer(&inp_smol).spanned()
            .map(|(token_result, span)| {
                let token = token_result?; // Propagate LexicalError
                Ok((span.start, token, span.end)) // (usize, Token, usize)
            });
            let parser = grammarcmds::CommandParser::new();
            (ast, dep_vec) = match parser.parse(curr_sheet_number as u32, &sheetstore, lexer) {  //NOTE: Error messages are temporary.
                Ok(x) => x,
                Err(ParseError::User{error: tokenscmds::LexicalError::InvalidToken}) => 
                {
                    last_err_msg = String::from("Invalid Token"); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                },
                Err(ParseError::User{error: tokenscmds::LexicalError::InvalidInteger(x)}) => 
                {   
                    last_err_msg = format!("Invalid Integer {:?}", x); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                }, 
                Err(e) => 
                {
                    last_err_msg = format!("This error: {:?}", e); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                }
            };
        }
        else
        {
            let lexer = tokensexpr::Token::lexer(&inp).spanned()
            .map(|(token_result, span)| {
                let token = token_result?; // Propagate LexicalError
                Ok((span.start, token, span.end)) // (usize, Token, usize)
            });
            let parser = grammarexpr::AssignParser::new();
            (ast, dep_vec) = match parser.parse(curr_sheet_number as u32, &sheetstore, lexer) {  //NOTE: Error messages are temporary.
                Ok(x) => x,
                Err(ParseError::User{error: tokensexpr::LexicalError::InvalidToken}) => 
                {
                    last_err_msg = String::from("Invalid Token"); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                },
                Err(ParseError::User{error: tokensexpr::LexicalError::InvalidInteger(x)}) => 
                {   
                    last_err_msg = format!("Invalid Integer {:?}", x); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                }, 
                Err(e) => 
                {
                    last_err_msg = format!("This error: {:?}", e); 
                    // last_time = 0;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop
                }
            };
        }


        // println!("{:?}", dep_vec);
        // println!("{:?}", ast);

        let address: Addr;
        let new_function: Option<CellFunc>;
        // start = Instant::now();
        match ast.clone() {
            ast::Command::OtherCmd(cmd) => { 
                for dep in &dep_vec {
                    match dep {
                        ast::ParentType::Single(a_1) => {
                            let cell_sheet = &sheetstore.data[a_1.sheet as usize].borrow();
                            if a_1.row >= cell_sheet.rows {
                                // last_time = 0;
                                last_err_msg = String::from("Address row out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_1.col >= cell_sheet.columns {
                                // last_time = 0;
                                last_err_msg = String::from("Address column out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            let mut col = cell_sheet.data[a_1.col as usize].borrow_mut();
                            if col.cells.len() <= a_1.row  as usize
                            {
                                let mut p = col.cells.len() as u32;
                                col.cells.resize_with(a_1.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: cell_sheet.sheet_idx, row: p-1, col: a_1.col})))});
                            }
                            drop(col);
                        },
                        ast::ParentType::Range(a_1, a_2) => {
                            
                            let cell_sheet = { 
                                if a_1.sheet == a_2.sheet {
                                    &sheetstore.data[a_1.sheet as usize].borrow()
                                }
                                else {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range addresses must belong to the same sheet.");
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop
                                }
                            };

                            if a_1.row >= cell_sheet.rows {
                                // last_time = 0;
                                last_err_msg = String::from("Range start address row out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_1.col >= cell_sheet.columns {
                                // last_time = 0;
                                last_err_msg = String::from("Range start address column out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_2.row >= cell_sheet.rows {
                                // last_time = 0;
                                last_err_msg = String::from("Range end address row out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_2.col >= cell_sheet.columns {
                                // last_time = 0;
                                last_err_msg = String::from("Range end address column out of range"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_1.col > a_2.col {
                                // last_time = 0;
                                last_err_msg = String::from("Range start column higher than end column"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            if a_1.row > a_2.row {
                                // last_time = 0;
                                last_err_msg = String::from("Range start row higher than end row"); //NOTE: Error messages are temporary.
                                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                jump_to_last = true;
                                continue 'mainloop;
                            }
                            for i in a_1.col..=a_2.col {
                                let mut col = cell_sheet.data[i as usize].borrow_mut();
                                if col.cells.len() <= a_2.row as usize
                                {
                                    let mut p = col.cells.len() as u32;
                                    col.cells.resize_with(a_2.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: cell_sheet.sheet_idx, row: p - 1, col: i})))});
                                }
                                drop(col);
                            }
                        },
                    }
                }


                match cmd 
                {
                    ast::OtherCommand::AddSheet(s, c, r) => {
                        if sheetstore.map.len() >= 12 {
                            last_err_msg = String::from("Number of active sheets limit is set to 12")
                        }
                        else if sheetstore.data.len() >= 50 {
                            last_err_msg = String::from("Total sheets (active or removed) created in session has limit set to 50.")
                        }
                        else if c==0 || r==0 {
                            last_err_msg = String::from("Column and row size cannot be zero.")
                        }
                        else if s.chars().count() > 15 {
                            last_err_msg = format!("Sheet name \"{}\" is larger than 15 characters.", s);
                        }
                        else {
                            let res = sheetstore.new_sheet(s.as_str(), c, r);
                            if res.is_none() {
                                last_err_msg = format!("Sheet name \"{}\" already exists.", s);
                            }
                            else { last_err_msg = String::from("ok") }
                        }
                    }
                    ast::OtherCommand::RemoveSheet(s) => {
                        if sheetstore.map.len() == 1 {
                            last_err_msg = String::from("Cannot use remove sheet when only one sheet remains.");
                        }
                        else {
                            if let Some(sheet_num) = sheetstore.num_from_name(s.as_str())
                            {

                                let sheet_rc = Rc::clone(&sheetstore.data[sheet_num]);
                                let sheet = sheet_rc.borrow();
                                let num_rows = sheet.rows;
                                let num_cols = sheet.columns;
                                for i in 0..num_cols
                                {
                                    let column_ref = &sheet.data[i as usize];
                                    let column = column_ref.borrow();
                                    for j in 0..num_rows
                                    {
                                        if j as usize >= column.cells.len() {
                                            break;
                                        }
                                        let cell_rc = Rc::clone(&column.cells[j as usize]);
                                        let cell = cell_rc.borrow();
                                        invalidate_children(&mut sheetstore.data, cell.addr.clone());
                                        drop(cell);
                                        let mut cell = cell_rc.borrow_mut();
                                        cell.children.clear();
                                    }
                                }
                            }

                            let res = sheetstore.remove_sheet(s.as_str());
                            match res {
                                None => {
                                last_err_msg = format!("Sheet name \"{}\" not found.", s);
                                },
                                Some(n) => { 
                                    last_err_msg = String::from("ok");
                                    curr_sheet_number = sheetstore.map[0].1;
                                    curr_col = 0;
                                    curr_row = 0;
                                    if n == outputs_widget.sheetnum {
                                        show_graph = false
                                    }
                                }
                            }
                        }
                    }
                    ast::OtherCommand::RenameSheet(s, snew) => {
                        if snew.chars().count() > 15 {
                            last_err_msg = format!("Sheet name \"{}\" is larger than 15 characters.", snew);
                        }
                        else {
                            let res = sheetstore.rename_sheet(s.as_str(), snew.as_str());
                            if res.is_none() {
                                last_err_msg = format!("Either Sheet name \"{}\" not found OR Sheet name \"{}\" already exists.", s, snew);
                            } else { last_err_msg = String::from("ok") }
                        }
                    }
                    ast::OtherCommand::DuplicateSheet(s, snew_op) => 
                    {
                        let snew = match snew_op {
                            Some(x) => x,
                            None => format!("{}-copy", s)
                        };
                        let res = sheetstore.num_from_name(s.as_str());
                        if sheetstore.map.len() >= 12 {
                            last_err_msg = String::from("Number of active sheets limit is set to 12")
                        }
                        else if sheetstore.data.len() >= 50 {
                            last_err_msg = String::from("Total sheets (active or removed) created in session has limit set to 50.")
                        }
                        else if res.is_none() 
                        {
                            last_err_msg = format!("Sheet name \"{}\" not found.", s);
                        }
                        else if snew.chars().count() > 15 {
                            last_err_msg = format!("Sheet name \"{}\" is larger than 15 characters.", snew);
                        }
                        else 
                        {
                            let sheet_num = res.unwrap();
                            let res2 = duplicate_sheet(&mut sheetstore.data, sheet_num);
                            if let Ok(new_sheet) = res2 
                            {
                                last_err_msg = String::from("ok");
                                // sheetstore.renameSheet(s.as_str(), &snew).unwrap();
                                sheetstore.add_sheet(snew.as_str(), new_sheet); 
                            } 
                            else 
                            {
                                last_err_msg = format!("Error occured during duplication: {}", res2.err().unwrap())
                            }
                        }
                    },
                    ast::OtherCommand::Undo =>
                    {
                        // last_err_msg = format!("undo_history: {:?}", undo_history.last());      // debugging purpose
                        match undo(&mut sheetstore.data, &mut undo_history, &mut redo_history, &settings)
                        {
                            Ok((cell_addr,old_function)) =>
                            {
                                // start = Instant::now();
                                // println!("{}", Rc::clone(& (&sheets[0].borrow().data[a.col as usize].borrow_mut()[a.row as usize])).try_borrow_mut().is_ok());
                                if let Err(strr) = evaluate(&mut sheetstore.data, &cell_addr, &old_function)
                                {
                                    // last_time = start.elapsed().as_secs();
                                    last_err_msg = strr;
                                    continue 'mainloop;
                                } 
                            }
                            Err(errmsg) =>
                            {
                                last_err_msg = errmsg;
                            }
                        }
                    }
                    ast::OtherCommand::Redo =>
                    {
                        // last_err_msg = format!("redo_history: {:?}", redo_history.last());      // debugging purpose
                        match redo(&mut sheetstore.data, &mut undo_history, &mut redo_history, &settings)
                        {
                            Ok((cell_addr,old_function)) =>
                            {
                                // start = Instant::now();
                                // println!("{}", Rc::clone(& (&sheets[0].borrow().data[a.col as usize].borrow_mut()[a.row as usize])).try_borrow_mut().is_ok());
                                if let Err(strr) = evaluate(&mut sheetstore.data, &cell_addr, &old_function)
                                {
                                    // last_time = start.elapsed().as_secs();
                                    last_err_msg = strr;
                                    continue 'mainloop;
                                } 
                            }
                            Err(errmsg) =>
                            {
                                last_err_msg = errmsg;
                            }
                        }
                    }
                    ast::OtherCommand::ExportCsv(s) => {
                        let s_num = sheetstore.num_from_name(s.as_str());
                        match s_num {
                            Some(x) => {

                                let imp_result = export_csv(&sheetstore.data[x].borrow(), s.as_str());
                                match imp_result {
                                    Ok(()) => {//Since we have alreayd verified that name does not exist already, this should happen successfully
                                        last_err_msg = String::from("ok");
                                    },
                                    Err(e) => last_err_msg = format!("Error occured during import: {}", e)
                                }
                            }
                            None => last_err_msg = format!("Sheet name \"{}\" not found.", s)
                        }
                    }
                    ast::OtherCommand::LoadCsv(path, opt_s) => 
                    {                        
                        if sheetstore.map.len() >= 12 {
                            last_err_msg = String::from("Number of active sheets limit is set to 12")
                        }
                        else if sheetstore.data.len() >= 50 {
                            last_err_msg = String::from("Total sheets (active or removed) created in session has limit set to 50.")
                        }
                        else {
                            match opt_s {
                                None => {
                                    let name_opt = path.strip_suffix(".csv");
                                    match name_opt {
                                        Some(name) => {
                                            if name.chars().count() > 15 {
                                                last_err_msg = format!("Sheet name \"{}\" is larger than 15 characters.", name);
                                            }
                                            else if sheetstore.num_from_name(name).is_none() {
                                                let imp_result = import_csv(&path, sheetstore.data.len() as u32);
                                                match imp_result {
                                                    Ok(x) => {
                                                        sheetstore.add_sheet(name, x); //Since we have alreayd verified that name does not exist already, this should happen successfully
                                                        last_err_msg = String::from("ok");
                                                    },
                                                    Err(e) => last_err_msg = format!("Error occured during import: {}", e)
                                                }
                                            }
                                            else {
                                                last_err_msg = format!("Sheet name \"{}\" already exist.", name)
                                            }
                                        },
                                        None => last_err_msg = format!("Invalid filepath (does not end in .csv): \"{}\"", path)
                                    }
                                },
                                Some(name) => {
                                    if sheetstore.num_from_name(name.as_str()).is_none() {
                                        let imp_result = import_csv(&path, sheetstore.data.len() as u32);
                                        match imp_result {
                                            Ok(x) => {
                                                sheetstore.add_sheet(name.as_str(), x); //Since we have alreayd verified that name does not exist already, this should happen successfully
                                                last_err_msg = String::from("ok");
                                            },
                                            Err(e) => last_err_msg = format!("Error occured during import: {}", e)
                                        }
                                    }
                                    else {
                                        last_err_msg = format!("Sheet name \"{}\" already exist.", name)
                                    };
                                }
                            }
                        }
                    },
                    ast::OtherCommand::Resize(s, c, r) => {
                        if c==0 || r==0 {
                            last_err_msg = String::from("Column and row size cannot be zero.")
                        }
                        else {
                            match sheetstore.num_from_name(s.as_str()) {
                                Some(sheet_num) => {
                                    sheetstore.data[sheet_num].borrow_mut().resize(r, c);  //NOTE: r aur c ka order har jag asame kar dena chahiye ajeeb lag raha
                                    last_err_msg = String::from("ok");
                                } 
                                None => last_err_msg = format!("Sheet name \"{}\" not found.", s)
                            }
                        }
                    },
                    ast::OtherCommand::CopyCellVals(addr1, addr2) =>
                    {
                        copy_cell_value(addr1, addr2,&sheetstore.data);
                        last_err_msg = String::from("ok");
                    },
                    ast::OtherCommand::CopyCellFormulae(addr1, addr2) =>
                    {
                        match copy_cell_function(addr1, addr2,&mut sheetstore.data)
                        {
                            Ok(_) => 
                            {
                                last_err_msg = String::from("ok");
                            }
                            Err(e) => 
                            {
                                last_err_msg = format!("Error occured during copy: {}", e);
                            }
                        }
                    },
                    ast::OtherCommand::CopyRangeFormulae(addr1,addr2, addr3 ) =>
                    {
                        match copy_range_function(addr1, addr2, addr3, &mut sheetstore.data)
                        {
                            Ok(_) => last_err_msg = String::from("ok"),
                            Err(e) => last_err_msg = format!("Error occured during copy: {}", e)
                        }
                    },
                    ast::OtherCommand::CopyRangeVals(addr1,addr2, addr3) =>
                    {
                        copy_range_value(addr1, addr2,addr3, &sheetstore.data);
                        last_err_msg = String::from("ok");
                    },
                    ast::OtherCommand::AutofillAp(addr1,addr2) =>
                    {
                        let res = autofill_ap(addr1, addr2, &mut sheetstore.data);
                        match res {
                            Ok(_) => last_err_msg = String::from("ok"),
                            Err(e) => last_err_msg = format!("Error occured during autofill: {}", e)
                        }
                    },
                    ast::OtherCommand::AutofillGp(addr1, addr2) =>
                    {
                        let res = autofill_gp(addr1, addr2, &mut sheetstore.data);
                        match res {
                            Ok(_) => last_err_msg = String::from("ok"),
                            Err(e) => last_err_msg = format!("Error occured during autofill: {}", e)
                        }
                    },
                    ast::OtherCommand::MakeChart(addr1,addr2,addr3,addr4 ) =>
                    {
                        if addr1.sheet == addr2.sheet &&  addr2.sheet == addr3.sheet && addr3.sheet == addr4.sheet 
                        {
                            if addr1.col == addr2.col && addr3.col == addr4.col
                            {
                                if addr2.row - addr1.row == addr4.row - addr3.row 
                                {
                                    outputs_widget.col1=addr1.col as usize;
                                    outputs_widget.col2=addr3.col as usize;
                                    outputs_widget.row_start1=addr1.row as usize;
                                    outputs_widget.row_end1=addr2.row as usize;
                                    outputs_widget.row_start2=addr3.row as usize;
                                    outputs_widget.row_end2=addr4.row as usize;
                                    outputs_widget.sheetnum = addr1.sheet as usize;
                                    show_graph = true;
                                    last_err_msg = String::from("ok");
                                }
                                else 
                                {
                                    last_err_msg = String::from("The given ranges are not of the same length")
                                }
                            }
                            else 
                            {
                                last_err_msg = String::from("The given ranges are not 1 dimensional")
                            }
                        }
                        else
                        {
                            last_err_msg = String::from("The given ranges are not in the same sheet")
                        }
                    }
                
                };
                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                jump_to_last = true;
                // continue 'mainloop
            }
            ast::Command::DisplayCmd(d_cmd) => {
                let curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                match d_cmd {
                    ast::DisplayCommand::EnableOut => {},
                    ast::DisplayCommand::DisableOut => {},
                    ast::DisplayCommand::ScrollTo(addr) => 
                    {
                        if (addr.row >= curr_sheet.rows) | (addr.col >= curr_sheet.columns) {
                            // last_time = 0;
                            last_err_msg = String::from("Address out of bounds");
                            history_widget.history.push((inp.clone(), last_err_msg.clone()));
                            jump_to_last = true;
                            continue 'mainloop
                        }
                        curr_sheet_number = addr.sheet as usize;
                        let curr_sheet = &sheetstore.data[curr_sheet_number].borrow();
                        curr_row = cmp::min(addr.row, curr_sheet.rows.saturating_sub(1)) as usize;
                        curr_col = cmp::min(addr.col, curr_sheet.columns.saturating_sub(1)) as usize; 
                    },

                    ast::DisplayCommand::MoveUp => curr_row = curr_row.saturating_sub(10),
                    ast::DisplayCommand::MoveDown => curr_row = cmp::min(curr_row.saturating_add(10) , curr_sheet.rows.saturating_sub(10) as usize),
                    ast::DisplayCommand::MoveRight => curr_col = cmp::min(curr_col.saturating_add(10) , curr_sheet.columns.saturating_sub(10) as usize),
                    ast::DisplayCommand::MoveLeft => curr_col = curr_col.saturating_sub(10),
                };
                last_err_msg = String::from("ok");
                history_widget.history.push((inp.clone(), last_err_msg.clone()));
                jump_to_last = true;
                continue 'mainloop
            },
            ast::Command::Quit => exit = true,
            ast::Command::AssignCmd(a, b_ex) => {  //NOTE: All validity checks for addresses will be more complicated when we implement multiple sheets.

                let old_func: Option<CellFunc>;                 // mut is doubtful
                let old_valid: bool;                    // mut is doubtful
                let old_formula: String;
                let mut new_formula: String = String::from("~");
                {
                    let cell_sheet = &sheetstore.data[a.sheet as usize].borrow();
                    if a.row >= cell_sheet.rows {
                        // last_time = 0;
                        last_err_msg = String::from("Target address row out of range"); //NOTE: Error messages are temporary.
                        history_widget.history.push((inp.clone(), last_err_msg.clone()));
                        jump_to_last = true;
                        continue 'mainloop;
                    }
                    if a.col >= cell_sheet.columns {
                        // last_time = 0;
                        last_err_msg = String::from("Target address column out of range"); //NOTE: Error messages are temporary.
                        history_widget.history.push((inp.clone(), last_err_msg.clone()));
                        jump_to_last = true;
                        continue 'mainloop;
                    }
                    let mut col = cell_sheet.data[a.col as usize].borrow_mut();
                    if col.cells.len() <= a.row as usize
                    {
                        let mut p = col.cells.len() as u32;
                        col.cells.resize_with(a.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: cell_sheet.sheet_idx, row: p-1, col: a.col})))});
                    }
                    drop(col);

                    for dep in &dep_vec {
                        match dep {
                            ast::ParentType::Single(a_1) => {
                                let cell_sheet = &sheetstore.data[a_1.sheet as usize].borrow();
                                if a_1.row >= cell_sheet.rows {
                                    // last_time = 0;
                                    last_err_msg = String::from("Address row out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_1.col >= cell_sheet.columns {
                                    // last_time = 0;
                                    last_err_msg = String::from("Address column out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                let mut col = cell_sheet.data[a_1.col as usize].borrow_mut();
                                if col.cells.len() <= a_1.row  as usize
                                {
                                    let mut p = col.cells.len() as u32;
                                    col.cells.resize_with(a_1.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: cell_sheet.sheet_idx, row: p-1, col: a_1.col})))});
                                }
                                drop(col);
                            },
                            ast::ParentType::Range(a_1, a_2) => {
                                
                                let cell_sheet = { 
                                    if a_1.sheet == a_2.sheet {
                                        &sheetstore.data[a_1.sheet as usize].borrow()
                                    }
                                    else {
                                        // last_time = 0;
                                        last_err_msg = String::from("Range addresses must belong to the same sheet.");
                                        history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                        jump_to_last = true;
                                        continue 'mainloop
                                    }
                                };

                                if a_1.row >= cell_sheet.rows {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range start address row out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_1.col >= cell_sheet.columns {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range start address column out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_2.row >= cell_sheet.rows {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range end address row out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_2.col >= cell_sheet.columns {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range end address column out of range"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_1.col > a_2.col {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range start column higher than end column"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                if a_1.row > a_2.row {
                                    // last_time = 0;
                                    last_err_msg = String::from("Range start row higher than end row"); //NOTE: Error messages are temporary.
                                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                                    jump_to_last = true;
                                    continue 'mainloop;
                                }
                                for i in a_1.col..=a_2.col {
                                    let mut col = cell_sheet.data[i as usize].borrow_mut();
                                    if col.cells.len() <= a_2.row as usize
                                    {
                                        let mut p = col.cells.len() as u32;
                                        col.cells.resize_with(a_2.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: cell_sheet.sheet_idx, row: p - 1, col: i})))});
                                    }
                                    drop(col);
                                }
                            },
                        }
                    }

                    let target_sheet = &sheetstore.data[a.sheet as usize].borrow();
                    let target_cell_rc = Rc::clone(& (target_sheet.data[a.col as usize].borrow_mut()[a.row as usize]));
                    let mut target_cell_ref = target_cell_rc.borrow_mut();
                    old_func = (target_cell_ref).cell_func.clone();
                    old_valid = target_cell_ref.valid;

                    old_formula = target_cell_ref.formula.clone();
                    (target_cell_ref).cell_func = Some(CellFunc{expression: *b_ex});
                    new_function = target_cell_ref.cell_func.clone();
                    address = target_cell_ref.addr.clone();
                    // println!("{}", target_cell_rc.try_borrow_mut().is_ok());
                    if let Some(eq_index) = inp.find('=') {
                        target_cell_ref.formula = inp[eq_index + 1..].trim().to_string();
                        new_formula = target_cell_ref.formula.clone();
                    }
                    drop(target_cell_ref);

                }
                // start = Instant::now();
                    // println!("{}", Rc::clone(& (&sheets[0].borrow().data[a.col as usize].borrow_mut()[a.row as usize])).try_borrow_mut().is_ok());
                { 
                    let target_sheet = &sheetstore.data[a.sheet as usize].borrow();
                    let target_cell_rc = Rc::clone(& (target_sheet.data[a.col as usize].borrow_mut()[a.row as usize]));
                    let target_cell_ref = target_cell_rc.borrow();

                    undo_history.push((true, address, old_func.clone(), Some(old_formula), old_valid, new_function.clone(), Some(new_formula),target_cell_ref.valid));
                    if undo_history.len() > settings.undo_history_limit as usize
                    {
                        undo_history.remove(0);
                    }
                    redo_history.clear();
                }

                if let Err(strr) = evaluate(&mut sheetstore.data, &a, &old_func)
                {
                    // last_time = start.elapsed().as_secs();
                    last_err_msg = strr;
                    history_widget.history.push((inp.clone(), last_err_msg.clone()));
                    jump_to_last = true;
                    continue 'mainloop;   
                }
            }
        }
        
        if let ast::Command::OtherCmd(cmd) = ast {
            match cmd
            {
                ast::OtherCommand::Undo => {},
                ast::OtherCommand::Redo => {},
                _ =>
                {
                    undo_history.push((false, Addr{sheet: 0, row: 0, col: 0}, None, None, false, None, None, false));
                }
            }
            continue 'mainloop;
            
        }
        // last_time = start.elapsed().as_secs();
        last_err_msg = String::from("ok");
        history_widget.history.push((inp.clone(), last_err_msg.clone()));
        jump_to_last = true;}

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    io::stdout().flush().unwrap();
    Ok(())
}

