#![allow(unused)]
use std::cmp::{PartialEq, Eq, Ordering, PartialOrd, Ord};
use std::fmt::Display;
use crate::cell_operations::ValueType;
pub enum ParserError{
    NumberTooLargeAt(String, u32, u32),
}


// pub enum CellRef {
//     /// Reference within the current sheet
//     Local {
//         row: usize,
//         col: usize,
//     },
//     /// Reference to a different sheet
//     Global {
//         sheet: usize,
//         row: usize,
//         col: usize,
//     },
// }

#[derive(Debug, Clone)]
pub enum Command {
    DisplayCmd(DisplayCommand),  //Note: IS Box<DisplayCommand> better? Display Command is a finite data type, but expr was not.
    OtherCmd(OtherCommand),
    AssignCmd(Addr, Box<Expr>),
    Quit,
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    EnableOut,
    DisableOut,
    ScrollTo(Addr),
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

#[derive(Debug, Clone)]
pub enum OtherCommand {
    AddSheet(String, usize, usize),
    RemoveSheet(String),
    RenameSheet(String, String),
    DuplicateSheet(String, Option<String>),
    Undo,
    Redo,
    // Help(String) //Display help for the command
    // List //Display list of all commands

    // AddRow(i32),
    // AddCol(i32),
    // RemoveRow(i32),
    // RemoveCol(i32),

    MakeChart(Addr, Addr,Addr, Addr),

    CopyCellVals(Addr, Addr),
    CopyRangeVals(Addr, Addr, Addr),
    CopyCellFormulae(Addr, Addr),
    CopyRangeFormulae(Addr, Addr, Addr),

    AutofillAp(Addr, Addr),
    AutofillGp(Addr, Addr),

    ExportCsv(String),
    LoadCsv(String, Option<String>), //File, SheetName
    Resize(String, usize, usize)
    //Graph ke commands daal dena @ExactHarmony917
    //ML wale commands daal dena @ExactHarmony917
    

}



#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool),
    String(String),
    Integer(i32),
    Float(f64),
    Cell(Addr),
    Wildcard,
    MonoOp(MonoFunction, Box<Expr>),
    RangeOp{op: RangeFunction, start: Addr, end: Addr, cond: Box<Expr>}, //Note: Should addr be under Box<>?
    InfixOp(Box<Expr>, InfixFunction, Box<Expr>),
    BinOp(BinaryFunction, Box<Expr>, Box<Expr>),
    TernaryOp(TernaryFunction, Box<Expr>, Box<Expr>, Box<Expr>)
}

pub enum ParentType {
    Single(Addr),
    Range(Addr, Addr),
}
impl std::fmt::Debug for ParentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParentType:: Single(addr) => write!(f, "Single({:?})", addr),
            ParentType::Range(start, end) => write!(f, "Range({:?}, {:?})", start, end),
        }
    }
}
impl Expr
{
    // pub enum ParentType {
    //     Single(Addr),
    //     Range(Addr, Addr),
    // }


    pub fn get_dependency_list (&self) -> Vec<ParentType> 
    {
        match self 
        {
            Expr::Integer(_) => vec![],
            Expr::String(_) => vec![],
            Expr::Bool(_) => vec![],
            Expr::Float(_) => vec![],
            Expr::Cell(addr) => vec![ParentType::Single(addr.clone())],
            Expr::MonoOp(_, expr) => expr.get_dependency_list(),
            Expr::RangeOp{start, end, ..} => vec![ParentType::Range(start.clone(), end.clone())],
            Expr::InfixOp(left, _, right) => {
                let mut deps = left.get_dependency_list();
                deps.append(&mut right.get_dependency_list());
                deps
            }
            Expr::BinOp(_, left, right) => {
                let mut deps = left.get_dependency_list();
                deps.append(&mut right.get_dependency_list());
                deps
            }
            Expr::TernaryOp(_, cond, true_expr, false_expr) => {
                let mut deps = cond.get_dependency_list();
                deps.append(&mut true_expr.get_dependency_list());
                deps.append(&mut false_expr.get_dependency_list());
                deps
            }
            Expr::Wildcard => vec![], 

        }
    }

}

// pub enum Addr {
//     Local { row: u32, col: u32}, //NOTE: check all different int datatypes used a at different places and verify.
//     Global {sheet: u32, row: u32, col: u32} //NOTE: sheet should be String or str or &str or something else?!?.
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addr {
    pub sheet: u32,
    pub row: u32,
    pub col: u32,
}

impl PartialOrd for Addr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Addr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sheet
            .cmp(&other.sheet)
            .then(self.row.cmp(&other.row))
            .then(self.col.cmp(&other.col))
    }
}


// #[derive(Debug)]
// pub enum Range {

// }

#[derive(Debug, Clone)]
pub enum MonoFunction {
    Sleep,
    Not,
}

#[derive(Debug, Clone)]
pub enum RangeFunction {
    Sum,
    Avg,
    Max,
    Min,
    Stdev,
    Count
}

#[derive(Debug, Clone)]
pub enum BinaryFunction {
    Round,
    IsSubstr
}

#[derive(Debug, Clone)]
pub enum TernaryFunction {
    IfThenElse
}

#[derive(Debug, Clone)]
pub enum InfixFunction {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    FloorDiv,
    Mod,

    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    And,
    Or,
    // Not,
    
    Concat,
    
}