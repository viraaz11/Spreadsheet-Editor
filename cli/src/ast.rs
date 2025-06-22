#![allow(unused)]
use std::cmp::{PartialEq, Eq, Ordering, PartialOrd, Ord};

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

#[derive(Debug)]
pub enum Command {
    DisplayCmd(DisplayCommand),  //Note: IS Box<DisplayCommand> better? Display Command is a finite data type, but expr was not.
    AssignCmd(Addr, Expr),
    Quit,
}

#[derive(Debug)]
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
pub enum Expr {
    Atom(AtomicExpr),
    MonoOp(MonoFunction, AtomicExpr),
    RangeOp{op: RangeFunction, start: Addr, end: Addr}, //Note: Should addr be under Box<>?
    BinOp(AtomicExpr, BinaryFunction, AtomicExpr),
}


#[derive(Debug, Clone)]
pub enum AtomicExpr {
    Integer(i32),
    Cell(Addr),
}
#[derive(PartialEq, Eq)]
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
            Expr::Atom(AtomicExpr::Integer(_)) => vec![],
            Expr::Atom(AtomicExpr::Cell(addr)) => vec![ParentType::Single(addr.clone())],
            Expr::MonoOp(_, AtomicExpr::Integer(_)) => vec![],
            Expr::MonoOp(_, AtomicExpr::Cell(addr)) => vec![ParentType::Single(addr.clone())],
  
            Expr::RangeOp{start, end, ..} => vec![ParentType::Range(start.clone(), end.clone())],
            Expr::BinOp(left, _, right) => {
                let mut deps = 
                match left
                {
                    AtomicExpr::Cell(addr) => vec![ParentType::Single(addr.clone())],
                    AtomicExpr::Integer(_) => vec![]
                };
                let mut temp = 
                match right 
                {
                    AtomicExpr::Cell(addr) => vec![ParentType::Single(addr.clone())],
                    AtomicExpr::Integer(_) => vec![]
                };
                deps.append(&mut temp);
                deps
            }
        }
    }

}

// pub enum Addr {
//     Local { row: u32, col: u32}, //NOTE: check all different int datatypes used a at different places and verify.
//     Global {sheet: u32, row: u32, col: u32} //NOTE: sheet should be String or str or &str or something else?!?.
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addr {
    pub row: usize,
    pub col: usize,
}

impl PartialOrd for Addr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Addr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.row.cmp(&other.row)
            .then(self.col.cmp(&other.col))
    }
}


// #[derive(Debug)]
// pub enum Range {

// }

#[derive(Debug, Clone)]
pub enum MonoFunction {
    Sleep,
}

#[derive(Debug, Clone)]
pub enum RangeFunction {
    Sum,
    Avg,
    Max,
    Min,
    Stdev,
}

#[derive(Debug, Clone)]
pub enum BinaryFunction {
    Mul,
    Div,
    Add,
    Sub,
}