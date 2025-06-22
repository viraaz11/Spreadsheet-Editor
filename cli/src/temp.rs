pub mod ast;
pub mod tokens;
pub mod cell_operations;
pub mod evaluate_operations;
use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;
use logos::Logos;
use crate::cell_operations::{Cell, CellFunc, Sheet, ValueType};
use crate::evaluate_operations::evaluate;
use crate::tokens::LexicalError;
use std::io::{self, Write, BufWriter};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use std::time::Instant;
use std::fs::File;
// use serde::Serialize;
use csv::Reader;

//NOTE: PLEASE HAR JAGA usize KAR DO, bohot zyada conversions karne pad rahe hai


lalrpop_mod!(pub grammar); // include the generated parser

/* STUFF TO DO:
0 - initialise
0 - ask input
0 - If error, report respective error, restart
O - If not error:
    0 - Check required cells:
        0 - If out of range, report error (In extension: Suggest resizing)
        0 - Initialise if not done yet (should now happen automatically)
    0 - Give new function to - cell
    O - Give old and new function to - evaluate()
    O - If result is error (loop) then report error.
restart
*/

// struct SheetNames {
//     map: Vec<(String, u32)>
// }

// impl SheetNames {
//     fn numFromName(&self, name: str) -> u32 {
//         for i in 0..self
//     }
// }

struct Settings{
    cell_width: u32,
    formula_width: u32
}
impl Settings {
    fn new() -> Self {
        Settings{
            cell_width: 9,
            formula_width: 15
        }
    }
}

fn import_csv(csv_name: &str) -> Result<Sheet, String>
{
    if let Ok(rdr) = Reader::from_path(csv_name)
    {
        todo!()
    }
    else 
    {
        return Err("Error reading csv".to_string());
    }

}

fn export_csv(sheet: &Sheet) -> Result<(), String> 
{
    if let Ok(file) = File::create(sheet.sheet_name.clone() + ".csv")
    {
        let mut writer = BufWriter::new(file);
        let mut csv_data : Vec<Vec<String>> = vec![];
        for col in &sheet.data
        {
            csv_data.push(vec![]);
            if col.borrow().cells.len() == 0
            {
                for _i in 0..sheet.rows
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
                for i in 0..curr_rows
                {
                    let value = Rc::clone(&row[i as usize]).borrow().value.clone();
                    if let Some(last) = csv_data.last_mut()
                    {
                        (*last).push(value.to_string());
                    }

                }
            }
        }
        for row in 0..csv_data[0].len()
        {
            for col in 0..csv_data.len()
            {
                if let Ok(()) = write!(writer, "{}", csv_data[col][row])
                {}
                else 
                {
                    return Err("Error in writing csv".to_string());
                }
                if row != csv_data[0].len()-1
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
            {}
            else 
            {
                return Err("Error in writing csv".to_string());
            }
        }

        Ok(())
    }
    else 
    {
        return Err("Error in creating csv".to_string());
    }
}



fn display_sheet(col: u32, row: u32, sheet: &Sheet, settings: &Settings, showformulas: bool)
{
    let row_max = cmp::min(row+10, sheet.rows);
    let col_max = cmp::min(col+10, sheet.columns);
    let width = settings.cell_width as usize;
    
    print!("      ");
    for i in col..col_max {
        let mut curr = String::new();
        let mut curr_col = i + 1;
        while curr_col > 0
        {

            curr.push(((b'A') + ((curr_col-1) % 26) as u8) as char);
            
            curr_col -= 1;
            curr_col /= 26;
        }
        print!("{:>width$}", curr.chars().rev().collect::<String>());
    }
    println!();
    for i in row..row_max {
        print!("{:>width$}", i+1);
        for j in col..col_max {

            if showformulas
            {
                sheet.expr_at(j as usize, i as usize, settings.formula_width as usize);
            }
            else
            {
                let colref = sheet.data[j as usize].borrow();
                if i as usize >= colref.cells.len()
                {
                    print!("{:>width$}", "~");
                    continue
                } 
                else
                {
                    let cell = colref.cells[i as usize].borrow();
                    if cell.valid {
                        let val =  &cell.value;
                        match val {
                            ValueType::BoolValue(b) => print!("{:>width$}", b),
                            ValueType::IntegerValue(x) => print!("{:>width$}", x),
                            ValueType::FloatValue(n) => print!("{:>width$.2}", n, width = width),
                            ValueType::String(s) => print!("{:>width$}", s),
                        }
                    }
                    else {
                        print!("{:>width$}", "~ERR")
                    }
                }  
            }
        }
        println!()
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>{

    let r: u32 = std::env::args().nth(1)
        .expect("Row number not entered (First arg missing)")
        .parse().expect("Invalid input for Row number (First arg)");
    
    let c: u32 = std::env::args().nth(2)
        .expect("Column number not entered (Second arg missing)")
        .parse().expect("Invalid input for Column number (Second arg)");

    // let r: u32 = 3; ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////NOTE: For testing, remove later
    // let c: u32 = 3; ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////NOTE: For testing, remove later
//sheets: &Vec<Rc<RefCell<Sheet>>>

    let mut sheets: Vec<Rc<RefCell<Sheet>>> = vec![Rc::new(RefCell::new(Sheet::new(0, String::from("sheet0"), c, r)))];

    let mut exit : bool = false;

    let mut curr_col: usize= 0;
    let mut curr_row: usize = 0;
    let mut show_window: bool = true;
    let mut last_err_msg = String::from("ok");
    let settings = Settings::new();
    let mut last_time = 0;
    'mainloop: while !exit {
        let mut start = Instant::now();
        if show_window {
            // let curr_sheet = ;
            display_sheet(curr_col as u32, curr_row as u32, &sheets[0].borrow(),  &settings, false);
        }
        let mut inp = String::new();
        print!("[{}.0] ", last_time);
        print!("({}) >> ", last_err_msg);
        io::stdout().flush().unwrap();

        io::stdin()
        .read_line(&mut inp)
        .expect("Failed to read line"); //NOTE (to self): Better error message

        let lexer = tokens::Token::lexer(&inp).spanned()
        .map(|(token_result, span)| {
            let token = token_result?; // Propagate LexicalError
            Ok((span.start, token, span.end)) // (usize, Token, usize)
        });
        
        let parser = grammar::CommandParser::new();

        let (ast, dep_vec) = match parser.parse(0, lexer) {  //NOTE: Error messages are temporary.
            Ok(x) => x,
            Err(ParseError::User{error: LexicalError::InvalidToken}) => 
            {
                last_err_msg = String::from("Invalid Token"); 
                last_time = 0;
                continue
            },
            Err(ParseError::User{error: LexicalError::InvalidInteger(x)}) => 
            {   
                last_err_msg = format!("Invalid Integer {:?}", x); 
                last_time = 0;
                continue
            }, 
            Err(e) => 
            {
                last_err_msg = format!("This error: {:?}", e); 
                last_time = 0;
                continue
            }
        };
        // println!("{:?}", dep_vec);
        // println!("{:?}", ast);


        match ast {
            ast::Command::DisplayCmd(d_cmd) => {
                let curr_sheet = &sheets[0].borrow();
                match d_cmd {
                    ast::DisplayCommand::EnableOut => show_window = true,
                    ast::DisplayCommand::DisableOut => show_window = false,
                    ast::DisplayCommand::ScrollTo(addr) => 
                    {
                        if (addr.row >= curr_sheet.rows) | (addr.col >= curr_sheet.columns) {
                            last_time = 0;
                            last_err_msg = String::from("Address out of bounds");
                            continue
                        }
                        curr_col = cmp::max(0, cmp::min(addr.col as i64, curr_sheet.columns as i64 - 10)) as usize;
                        curr_row = cmp::max(0, cmp::min(addr.row as i64, curr_sheet.rows as i64 - 10)) as usize; 
                    },

                    ast::DisplayCommand::MoveUp => curr_row = cmp::max(0, cmp::min(curr_row as i64 -1 , curr_sheet.rows as i64 - 10)) as usize,
                    ast::DisplayCommand::MoveDown => curr_row = cmp::max(0, cmp::min(curr_row as i64 +1 , curr_sheet.rows as i64 - 10)) as usize,
                    ast::DisplayCommand::MoveRight => curr_col = cmp::max(0, cmp::min(curr_col as i64 +1 , curr_sheet.columns as i64 - 10)) as usize,
                    ast::DisplayCommand::MoveLeft => curr_col = cmp::max(0, cmp::min(curr_col as i64 -1 , curr_sheet.columns as i64 - 10)) as usize,
                }},
            ast::Command::Quit => exit = true,
            ast::Command::AssignCmd(a, b_ex) => {  //NOTE: All validity checks for addresses will be more complicated when we implement multiple sheets.
                let old_func: Option<CellFunc>;
                {
                let curr_sheet = &sheets[0].borrow();
                if a.row >= curr_sheet.rows {
                    last_time = 0;
                    last_err_msg = String::from("Target address row out of range"); //NOTE: Error messages are temporary.
                    continue 'mainloop;
                }
                if a.col >= curr_sheet.columns {
                    last_time = 0;
                    last_err_msg = String::from("Target address column out of range"); //NOTE: Error messages are temporary.
                    continue 'mainloop;
                }
                let mut col = curr_sheet.data[a.col as usize].borrow_mut();
                if col.cells.len() <= a.row as usize
                {
                    let mut p = col.cells.len() as u32;
                    col.cells.resize_with(a.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: curr_sheet.sheet_idx, row: p, col: a.col})))});
                }
                drop(col);

                for dep in &dep_vec {
                    match dep {
                        ast::ParentType::Single(a_1) => {
                            if a_1.row >= curr_sheet.rows {
                                last_time = 0;
                                last_err_msg = String::from("Address row out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_1.col >= curr_sheet.columns {
                                last_time = 0;
                                last_err_msg = String::from("Address column out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            let mut col = curr_sheet.data[a_1.col as usize].borrow_mut();
                            if col.cells.len() <= a_1.row  as usize
                            {
                                let mut p = col.cells.len() as u32;
                                col.cells.resize_with(a_1.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: curr_sheet.sheet_idx, row: p, col: a_1.col})))});
                            }
                            drop(col);
                        },
                        ast::ParentType::Range(a_1, a_2) => {
                            if a_1.row >= curr_sheet.rows {
                                last_time = 0;
                                last_err_msg = String::from("Range start address row out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_1.col >= curr_sheet.columns {
                                last_time = 0;
                                last_err_msg = String::from("Range start address column out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_2.row >= curr_sheet.rows {
                                last_time = 0;
                                last_err_msg = String::from("Range end address row out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_2.col >= curr_sheet.columns {
                                last_time = 0;
                                last_err_msg = String::from("Range end address column out of range"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_1.col > a_2.col {
                                last_time = 0;
                                last_err_msg = String::from("Range start column higher than end column"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            if a_1.row > a_2.row {
                                last_time = 0;
                                last_err_msg = String::from("Range start row higher than end row"); //NOTE: Error messages are temporary.
                                continue 'mainloop;
                            }
                            for i in a_1.col..=a_2.col {
                                let mut col = curr_sheet.data[i as usize].borrow_mut();
                                if col.cells.len() <= a_2.row as usize
                                {
                                    let mut p = col.cells.len() as u32;
                                    col.cells.resize_with(a_2.row as usize + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{sheet: curr_sheet.sheet_idx, row: p, col: i})))});
                                }
                                drop(col);
                            }
                        },
                    }
                }

                let target_cell_rc = Rc::clone(& (curr_sheet.data[a.col as usize].borrow_mut()[a.row as usize]));
                let mut target_cell_ref = target_cell_rc.borrow_mut();
                old_func = (target_cell_ref).cell_func.clone();
                (target_cell_ref).cell_func = Some(CellFunc{expression: *b_ex});
                // println!("{}", target_cell_rc.try_borrow_mut().is_ok());
                drop(target_cell_ref);

            }
            start = Instant::now();
                // println!("{}", Rc::clone(& (&sheets[0].borrow().data[a.col as usize].borrow_mut()[a.row as usize])).try_borrow_mut().is_ok());
                if let Err(strr) = evaluate(&mut sheets, &a, &old_func)
                {
                    last_time = start.elapsed().as_secs();
                    last_err_msg = strr;
                    continue 'mainloop;
                    
                }              
            }

        }
        last_time = start.elapsed().as_secs();
        last_err_msg = String::from("ok");
    }

    Ok(())
}