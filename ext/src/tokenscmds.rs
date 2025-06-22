use std::fmt;  // to implement the Display trait later
use std::num::ParseIntError;
use logos::Logos;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    InternalError(String),
    SheetNotFoundErr(String),
    #[default]
    InvalidToken,
    
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}



fn parse_local_cell(s: &str) -> Option<(u32, u32)> {
    let (letters, digits) = s.chars()
        .partition::<String, _>(|c| c.is_ascii_alphabetic());

    let row = digits.parse::<u32>().ok()? - 1;
    let mut col: u32 = 0;
    for c in letters.chars() {
        col = col*26 + (c as u8 - b'A' + 1) as u32; 
    }
    col -=1; //Zero based indexing
    Some((col, row))
}

fn parse_global_cell(s: &str) -> Option<(String, u32, u32)> {

    let (sheet, addr) = s.split_once(".").unwrap(); //NOTE: Source of panic, ensure it is correct.

    let (letters, digits) = addr.chars()
        .partition::<String, _>(|c| c.is_ascii_alphabetic());

    let row = digits.parse::<u32>().ok()? - 1;
    let mut col: u32 = 0;
    for c in letters.chars() {
        col = col*26 + (c as u8 - b'A' + 1) as u32; 
    }
    col -=1; //Zero based indexing
    Some((sheet.to_string(), col, row))
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[\n\r\f]+", error = LexicalError)]  //removed skip r"[ \t\n\r\f]+"
pub enum Token {

    #[regex("[ \t]+")]
    Ws,  //Ws stands for whitespace
    
    #[regex(r"(0|[1-9][0-9]*)\.[0-9]+([eE][-+]?[0-9]+)?", |lex| lex.slice().parse().ok())]
    #[regex("(0|[1-9][0-9]*)[eE][-+]?[0-9]+", |lex| lex.slice().parse().ok())]
    Float(f64),

    #[regex("0|[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i32),

    #[regex("True|False", |lex| Some(lex.slice() == "True"))]
    Bool(bool),

    #[regex("\"[^\"]*\"|\'[^\']*\'", |lex| {let s = lex.slice(); s[1..s.len() - 1].to_string()}) ]
    Str(String),

    #[token("_")]
    Wildcard,
  
    #[regex("[A-Z]{1,3}[1-9][0-9]{0,2}", |lex| parse_local_cell(lex.slice()))]
    LocalCell((u32, u32)),
    #[regex("[a-z0-9_]+\\.[A-Z]{1,3}[1-9][0-9]{0,2}", |lex| parse_global_cell(lex.slice()))] //NOTE: Sheet names must be lower case in this implementation
    GlobalCell((String, u32, u32)),

    #[token("w")]
    MoveUp,
    #[token("a")]
    MoveLeft,
    #[token("s")]
    MoveDown,
    #[token("d")]
    MoveRight,
    #[token("q")]
    Quit,

    #[token("enable_output")]
    EnableOut,
    #[token("disable_output")]
    DisableOut,
    #[token("scroll_to")]
    ScrollTo,

    #[token("load_csv")]
    LoadCsv,
    #[token("export_csv")]
    ExportCsv,
    
    #[token("resize")]
    Resize,
    
    #[token("add_sheet")]
    AddSheet,
    #[token("remove_sheet")]
    RemoveSheet,
    #[token("rename_sheet")]
    RenameSheet,
    #[token("dup_sheet")]
    DuplicateSheet,
    #[token("autofill_ap")]
    AutofillAp,
    #[token("autofill_gp")]
    AutofillGp,
    #[token("undo")]
    Undo,
    #[token("redo")]
    Redo,
    #[token("make_chart")]
    MakeChart,
    #[token("copy_cell_value")]
    CopyCellValue,
    #[token("copy_cell_formula")]
    CopyCellFormula,
    #[token("copy_range_values")]
    CopyRangeValues,
    #[token("copy_range_formulas")]
    CopyRangeFormulas,

    #[token(":")]
    Colon,
}

//Below is copy paste from lalrpop tutorial:
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{:?}", self)
    }
  }

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
