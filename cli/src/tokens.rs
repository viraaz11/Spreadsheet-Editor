use std::fmt;  // to implement the Display trait later
use std::num::ParseIntError;
use logos::Logos;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

fn parse_local_cell(s: &str) -> Option<(usize, usize)> {
    let (letters, digits) = s.chars()
        .partition::<String, _>(|c| c.is_ascii_alphabetic());

    let row = digits.parse::<usize>().ok()? - 1;
    let mut col: usize = 0;
    for c in letters.chars() {
        col = col*26 + (c as u8 - b'A' + 1) as usize; 
    }
    col -=1; //Zero based indexing
    Some((col, row))
}



#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r\f]+", error = LexicalError)]  //removed skip r"[ \t\n\r\f]+"
pub enum Token {

    // #[regex("[ \t]+")]
    // Ws,  //Ws stands for whitespace
    #[regex("0|[1-9][0-9]*", |lex| lex.slice().parse())]
    Integer(i32),
    #[regex("[A-Z]{1,3}[1-9][0-9]{0,2}", |lex| parse_local_cell(lex.slice()))]
    LocalCell((usize, usize)),
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
    #[token("SUM")]
    Sum,
    #[token("AVG")]
    Avg,
    #[token("MIN")]
    Min,
    #[token("MAX")]
    Max,
    #[token("STDEV")]
    Stdev,
    #[token("SLEEP")]
    Sleep,
    #[token("enable_output")]
    EnableOut,
    #[token("disable_output")]
    DisableOut,
    #[token("scroll_to")]
    ScrollTo,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("=")]
    Assign,
  
    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,

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
