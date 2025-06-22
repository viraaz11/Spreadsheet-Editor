///////////////// ONLY COMPLETED TOKENS FOR NUMERAL CELL FUNCS; STRING CELL FUNCS NOT DONE
///////////////// HAVE TO MAKE LEXER BY OWN 😢  FOR COMPLEX FUNCTIONS AS PROPOSED       // ban gaya yay
use crate::ast::{Expr, Addr};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ops::Index;
// #[allow(unused_imports)]
use std::rc::Rc;
use std::vec;



#[derive(Debug, Clone)]
pub enum ValueType 
{
    BoolValue(bool),
    IntegerValue(i32),
    FloatValue(f64),
    String(String),
}
impl std::fmt::Display for ValueType 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self 
        {
            ValueType::BoolValue(b) => write!(f, "{}", b),
            ValueType::IntegerValue(n) => write!(f, "{}", n),
            ValueType::FloatValue(n) => write!(f, "{}", n),
            ValueType::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone)]
pub struct CellFunc 
{
    // pub dependency_list: Vec<Weak<RefCell<Cell>>>,
    pub expression: Expr,
    // pub destination: Weak<RefCell<Cell>>, // USE OF Weak<T> is DOUBTFUL // @viraaz11: kyu chahiye      // @Pianissimo3115: HATA SKTE AS WELL
    // pub value: ValueType,           // HATA SKTE  //Hata diya
}

impl std::fmt::Display for CellFunc 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        write!(f, "{:?}", self.expression)
    }
}

impl std::fmt::Debug for CellFunc 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        writeln!(f, "CellFunc {{")?;

        writeln!(f, "\texpression: {:?},", self.expression)?;
        // writeln!(f, "\tvalue: {:?},", self.value)?;

        // write!(f, "\tdestination: ")?;
        // if let Some(dest_rc) = self.destination.upgrade() 
        // {
        //     let dest = dest_rc.borrow();
        //     writeln!(f, "Cell({}, {})", dest.addr.row, dest.addr.col)?;
        // } 
        // else 
        // {
        //     writeln!(f, "None (dropped)")?;
        // }

        writeln!(f, "}}")
    }
}
impl CellFunc
{
    pub fn new(expression: Expr) -> Self 
    {
        CellFunc 
        {
            expression,
            // destination,
            // value: ValueType::IntegerValue(0), // Default value; will be updated later //NOTE: Removed cause upar likha tha ki nahi chahiye
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell 
{
    pub addr: Addr,
    pub formula: String, 
    pub value: ValueType,
    pub cell_func: Option<CellFunc>,
    pub children: BTreeSet<Addr>, // USE OF Weak<T> is DOUBTFUL
    pub valid: bool,
}

impl Cell 
{
    pub fn new(addr: Addr) -> Self 
    {
        Cell 
        {
            addr,
            formula: "0".to_string(), 
            value: ValueType::IntegerValue(0),
            cell_func: None,
            valid: true,
            children: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column
{
    pub cells: Vec<Rc<RefCell<Cell>>>,
    pub sheet_number: u32,
    pub col_number: u32
}

// impl IndexMut<usize> for Column //NOTE IMPORTANT: Make sure no mutable reference is used whn, for example, printing the contents as it will crete unnencesary cells. Use this carefully.
// {

//     fn index_mut(&mut self, ind: usize) -> &mut Rc<RefCell<Cell>> { //NOTE: Rc ko as mut bhejna hota hai kya
//         if self.cells.len() <= ind
//         {
//             let mut p = self.cells.len() as u32;
//             self.cells.resize_with(ind+1, || {p += 1; Rc::new(RefCell::new(Cell::new(Addr{sheet: self.sheet_number, row: p, col: self.col_number})))});
//         }
//         &mut self.cells[ind]   //NOTE: Ye mut, & mut waherah dekh lena theek se koi please. (┬┬﹏┬┬)
//     }   
// }

impl Index<usize> for Column   //NOTE: This is not needed for my part, I guessed this would be needed in other part so left this here. 
{
    type Output = Rc<RefCell<Cell>>;

    fn index(&self, ind: usize) -> &Self::Output {
        // if self.cells.len() <= ind
        // {
        //     return &Rc::new(RefCell::new(Cell::new(Addr{sheet: self.sheet_number, row: ind as u32, col: self.col_number}))); //NOTE: Ye mut, & mut waherah dekh lena theek se koi please. (┬┬﹏┬┬)}
        // }
        &self.cells[ind]
    }   
}

impl Column
{
    fn new(col_number: u32, sheet_number: u32) -> Self {
        Column{cells: vec![], col_number, sheet_number}
    }

    fn truncate(&mut self, new_len: usize) {
        self.cells.truncate(new_len);
    }

    fn len(&self) -> usize {  
        self.cells.len()
    }

    fn val_at(&self, row: usize) -> ValueType { //NOTE: usize?
        if row >= self.cells.len() {
            return ValueType::IntegerValue(0);
        }
        return self.cells[row].borrow().value.clone() //NOTE: Doing clone here cause bohot koshish ke baad mujhse references nahi bheja gaya. Chota struct hai to farak nahi padna chahiye.
    }

    // fn expr_at(&self,row: usize, formula_width: usize)
    // {
    //     if row >= self.cells.len()
    //     {
    //         print!("");
    //     }
    //     if let Some(cell_func) = &self.cells[row].borrow().cell_func 
    //     {
    //         print!("{:>formula_width$}", cell_func.expression);
    //     } 
    //     else 
    //     {
    //         print!(""); 
    //     }
    // }
}


#[derive(Debug, Clone)]
pub struct Sheet
{
    pub data: Vec<RefCell<Column>>,
    pub rows: u32,
    pub columns: u32,
    pub sheet_idx: u32,
}
impl Sheet
{
    pub fn new(sheet_idx: u32, cols: u32, rows: u32) -> Self 
    {
        let mut s = Sheet 
        {
            data: vec![], //NOTE: Empty vector new se banana chahiye?
            rows: 0, //Number of rows in each column
            columns: 0, //Number of columns
            sheet_idx
        };
        s.resize(rows as usize, cols as usize);  //NOTE: Ye u32 and usize wali cheez sort kar lena please
        s
    }
    pub fn resize(&mut self, row_num: usize, col_num: usize)   
    {
        {
            let mut p = self.columns;  //Assuming self.columns == self.data.len() //NOTE: Hope to god that this does not cause issues??
            self.data.resize_with(col_num, || { p += 1; RefCell::new(Column::new(p-1, self.sheet_idx))}); //NOTE!!! : Defaulting sheet number to 0 for now. Must be changed.
        }
        self.columns = col_num as u32;  //NOTE: self.rows and (neeche,) self.columns u32 hai to unko "as usize" use karna pada. Har jaga otherwise usize lagega. If possible, sheet struct mai usize kar dena inko.

        if row_num < (self.rows as usize) {
            for i in 0..col_num
            {
                let mut col = self.data[i].borrow_mut();   
                if col.len() > row_num {              //NOTE: This check is redundant, as truncate does not do anything if given length is greater than current length. Left this for readability. Needed???
                    col.truncate(row_num) 
                }
            }
        }
        self.rows = row_num as u32;
    }

    
    pub fn val_at(&self, col: usize, row: usize) -> ValueType {  
        self.data[col].borrow().val_at(row)
    }

    // pub fn expr_at(&self, col: usize, row: usize, formula_width : usize)
    // {
    //     self.data[col].borrow().expr_at(row, formula_width);
    // }

}