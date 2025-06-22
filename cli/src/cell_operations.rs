///////////////// ONLY COMPLETED TOKENS FOR NUMERAL CELL FUNCS; STRING CELL FUNCS NOT DONE
///////////////// HAVE TO MAKE LEXER BY OWN 😢  FOR COMPLEX FUNCTIONS AS PROPOSED       // ban gaya yay
use crate::ast::{Expr, Addr};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ops::IndexMut;
use std::ops::Index;
// #[allow(unused_imports)]
use std::rc::Rc;



#[derive(Clone)]
pub struct CellFunc 
{
    // pub dependency_list: Vec<Weak<RefCell<Cell>>>,
    pub expression: Expr,
    // pub destination: Weak<RefCell<Cell>>, // USE OF Weak<T> is DOUBTFUL // @viraaz11: kyu chahiye      // @Pianissimo3115: HATA SKTE AS WELL
    // pub value: i32,           // HATA SKTE  //Hata diya
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
        writeln!(f, "CellFunc{{expression: {:?}}}", self.expression)
    }
}
impl CellFunc
{
    pub fn new(expression: Expr) -> Self 
    {
        CellFunc 
        {
            expression,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cell 
{
    pub addr: Addr,
    pub value: i32,
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
            value: 0,
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
    pub col_number: usize
}
impl IndexMut<usize> for Column //NOTE IMPORTANT: Make sure no mutable reference is used whn, for example, printing the contents as it will crete unnencesary cells. Use this carefully.
{

    fn index_mut(&mut self, ind: usize) -> &mut Rc<RefCell<Cell>> { //NOTE: Rc ko as mut bhejna hota hai kya
        if self.cells.len() <= ind
        {
            let mut p = self.cells.len() as u32;
            self.cells.resize_with(ind+1, || {p += 1; Rc::new(RefCell::new(Cell::new(Addr{row: (p - 1) as usize, col: self.col_number})))});
        }
        &mut self.cells[ind]   //NOTE: Ye mut, & mut waherah dekh lena theek se koi please. (┬┬﹏┬┬)
    }   
}


impl Index<usize> for Column   //NOTE: This is not needed for my part, I guessed this would be needed in other part so left this here. 
{
    type Output = Rc<RefCell<Cell>>;

    fn index(&self, ind: usize) -> &Self::Output {
        // if self.cells.len() <= ind
        // {
        //     return &Rc::new(RefCell::new(Cell::new(Addr{sheet: self.sheet_number, row: ind as usize, col: self.col_number}))); //NOTE: Ye mut, & mut waherah dekh lena theek se koi please. (┬┬﹏┬┬)}
        // }
        &self.cells[ind]
    }   
}

impl Column
{
    fn new(col_number: usize) -> Self {
        Column{cells: vec![], col_number}
    }

    // fn val_at(&self, row: usize) -> i32 { //NOTE: usize?
    //     if row >= self.cells.len() {
    //         return 0;
    //     }
    //     return self.cells[row].borrow().value //NOTE: Doing clone here cause bohot koshish ke baad mujhse references nahi bheja gaya. Chota struct hai to farak nahi padna chahiye.
    // }
}


#[derive(Debug, Clone)]
pub struct Sheet
{
    pub data: Vec<RefCell<Column>>,
    pub rows: usize,
    pub columns: usize,
}
impl Sheet
{
    pub fn new(cols: usize, rows: usize) -> Self 
    {
        let mut s = Sheet 
        {
            data: vec![], //NOTE: Empty vector new se banana chahiye?
            rows: 0, //Number of rows in each column
            columns: 0, //Number of columns
        };
        s.resize(rows , cols );  //NOTE: Ye usize and usize wali cheez sort kar lena please
        s
    }
    pub fn resize(&mut self, row_num: usize, col_num: usize)   
    {
        {
            let mut p = self.columns;  //Assuming self.columns == self.data.len() //NOTE: Hope to god that this does not cause issues??
            self.data.resize_with(col_num, || { p += 1; RefCell::new(Column::new(p-1))}); //NOTE!!! : Defaulting sheet number to 0 for now. Must be changed.
        }
        self.columns = col_num ;  //NOTE: self.rows and (neeche,) self.columns usize hai to unko "as usize" use karna pada. Har jaga otherwise usize lagega. If possible, sheet struct mai usize kar dena inko.

        self.rows = row_num;
    }

    
    // pub fn val_at(&self, col: usize, row: usize) -> i32 {  
    //     self.data[col].borrow().val_at(row)
    // }

}