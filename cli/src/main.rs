pub mod ast;
pub mod tokens;
pub mod cell_operations;
pub mod evaluate_operations;
use lalrpop_util::lalrpop_mod;
use lalrpop_util::ParseError;
use logos::Logos;
use crate::cell_operations::{Cell, CellFunc, Sheet};
use crate::evaluate_operations::evaluate;
use crate::tokens::LexicalError;
use std::io::{self, Write, BufRead, BufReader};
use std::rc::Rc;
use std::cell::RefCell;
use std::cmp;
use std::time::Instant;
lalrpop_mod!(pub grammar);

struct Settings{
    cell_width: u32,
}
impl Settings {
    fn new() -> Self {
        Settings{
            cell_width: 9,
        }
    }
}

fn display_sheet(col: usize, row: usize, sheet: &Sheet, settings: &Settings)
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

                let colref = sheet.data[j].borrow();
                if i >= colref.cells.len()
                {
                    print!("{:>width$}", "0");
                    continue
                } 
                else
                {
                    let cell = colref.cells[i].borrow();
                    if cell.valid {
                        let val =  cell.value;
                        print!("{:>width$}", val);
                    }
                    else {
                        print!("{:>width$}", "err")
                    }
                }  
        }
        println!()
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>
{

    let r: u32 = std::env::args().nth(1)
        .expect("Row number not entered (First arg missing)")
        .parse().expect("Invalid input for Row number (First arg)");
    
    let c: u32 = std::env::args().nth(2)
        .expect("Column number not entered (Second arg missing)")
        .parse().expect("Invalid input for Column number (Second arg)");

    let mut in_file: Box<dyn BufRead>  = match std::env::args().nth(3) {
        Some(x) => {
            let file = std::fs::File::open(x)?;
            Box::new(std::io::BufReader::new(file)) as Box<dyn BufRead>
        },
        None => Box::new(BufReader::new(io::stdin())),
    };
        

    let mut sheet: Sheet = Sheet::new(c as usize, r as usize);

    let mut exit : bool = false;

    let mut curr_col: usize= 0;
    let mut curr_row: usize = 0;
    let mut show_window: bool = true;
    let mut last_err_msg = String::from("ok");
    let settings = Settings::new();
    let mut last_time = 0;
    'mainloop: while !exit 
    {
        let mut inp = String::new();
        let mut start = Instant::now();
        if show_window 
        {
            display_sheet(curr_col, curr_row, &sheet,  &settings);
        }
        print!("[{}.0] ", last_time);
        // print!("({}) > ", last_err_msg.strip_suffix("\n").unwrap_or(&last_err_msg));
        print!("({}) > ", {if last_err_msg == "ok" {last_err_msg} else {"err".to_string()}});
        io::stdout().flush().unwrap();

        in_file
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
                let curr_sheet = &sheet;
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
                        curr_row = cmp::min(addr.row, curr_sheet.rows.saturating_sub(1));
                        curr_col = cmp::min(addr.col, curr_sheet.columns.saturating_sub(1)); 
                    },

                    // ast::DisplayCommand::MoveUp => curr_row = cmp::max(10, cmp::min(curr_row , curr_sheet.rows)) - 10,
                    ast::DisplayCommand::MoveUp => curr_row = curr_row.saturating_sub(10),
                    ast::DisplayCommand::MoveDown => curr_row = cmp::min(curr_row.saturating_add(10) , curr_sheet.rows.saturating_sub(10)),
                    ast::DisplayCommand::MoveRight => curr_col = cmp::min(curr_col.saturating_add(10) , curr_sheet.columns.saturating_sub(10)),
                    ast::DisplayCommand::MoveLeft => curr_col = curr_col.saturating_sub(10),
                };
                last_time = 0;
                last_err_msg = String::from("ok");
                // last_err_msg = inp.clone();

                continue;
                },
            ast::Command::Quit => exit = true,
            ast::Command::AssignCmd(a, b_ex) => {  //NOTE: All validity checks for addresses will be more complicated when we implement multiple sheets.
                let old_func: Option<CellFunc>;
                {
                let curr_sheet = &sheet;
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
                let mut col = curr_sheet.data[a.col].borrow_mut();
                if col.cells.len() <= a.row
                {
                    let mut p = col.cells.len() as u32;
                    col.cells.resize_with(a.row + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{row: (p-1 )as usize, col: a.col})))});
                }
                drop(col);
                // println!("{:?}", dep_vec);
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
                            let mut col = curr_sheet.data[a_1.col].borrow_mut();
                            if col.cells.len() <= a_1.row 
                            {
                                let mut p = col.cells.len() as u32;
                                col.cells.resize_with(a_1.row + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{row: (p-1) as usize, col: a_1.col})))});
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
                                let mut col = curr_sheet.data[i].borrow_mut();
                                if col.cells.len() <= a_2.row
                                {
                                    let mut p = col.cells.len() as u32;
                                    col.cells.resize_with(a_2.row + 1, || {p += 1; Rc::new(RefCell::new(Cell::new(ast::Addr{row: (p-1) as usize, col: i})))});
                                }
                                drop(col);
                            }
                        },
                    }
                }

                let target_cell_rc = Rc::clone(& (curr_sheet.data[a.col].borrow_mut()[a.row]));
                let mut target_cell_ref = target_cell_rc.borrow_mut();
                old_func = (target_cell_ref).cell_func.clone();
                (target_cell_ref).cell_func = Some(CellFunc{expression: b_ex});
                // println!("{}", target_cell_rc.try_borrow_mut().is_ok());
                drop(target_cell_ref);

            }
            start = Instant::now();
                // println!("{}", Rc::clone(& (&sheets[0].borrow().data[a.col].borrow_mut()[a.row])).try_borrow_mut().is_ok());
                if let Err(n) = evaluate(&mut sheet, &a, &old_func) {
                    if n != -1 {
                        last_time = start.elapsed().as_secs();
                        last_err_msg = "err".to_string();
                        continue 'mainloop;
                    }
                }
                        
            }

        }
        last_time = start.elapsed().as_secs();
        // last_err_msg = inp.clone();
        last_err_msg = String::from("ok");
    }

    Ok(())
}