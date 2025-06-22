use std::thread;
use std::time::Duration;
use crate::ast::{Addr, BinaryFunction, Expr, AtomicExpr, ParentType, RangeFunction};
use crate::cell_operations::{Sheet,Cell,CellFunc,Column};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
fn min_eval(data: &[RefCell<Column>], range: ((usize,usize),(usize,usize))) -> Result<i32,i32> 
{

    let cell1: (usize, usize) = range.0;
    let cell2: (usize, usize) = range.1;
    let mut mini = i32::MAX;
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            if cell_ref.value < mini {
                mini = cell_ref.value;
            }
        }
    }
    
    Ok(mini)
   
}
fn max_eval(data: &[RefCell<Column>], range: ((usize,usize),(usize,usize))) -> Result<i32, i32> 
{
    
    let cell1: (usize, usize) = range.0;
    let cell2: (usize, usize) = range.1;
    let mut maxi = i32::MIN;
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            if cell_ref.value > maxi {
                maxi = cell_ref.value;
            }
        }
    }
    
    Ok(maxi)
}

fn sum_eval(data: &[RefCell<Column>], range: ((usize,usize),(usize,usize))) -> Result<i32, i32> 
{
    
    let cell1: (usize, usize) = range.0;
    let cell2: (usize, usize) = range.1;
    let mut summ = 0;
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            summ += cell_ref.value;
        }
    }
    
    Ok(summ)
}

fn avg_eval(data: &[RefCell<Column>], range: ((usize,usize),(usize,usize))) -> Result<i32, i32> 
{
    
    let cell1: (usize, usize) = range.0;
    let cell2: (usize, usize) = range.1;
    let mut summ = 0;
    let mut count = 0;  
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            summ += cell_ref.value;
            count += 1;
        }
    }
    
    Ok(summ/count)
}


fn stdev_eval(data: &[RefCell<Column>], range: ((usize,usize),(usize,usize))) -> Result<i32, i32> 
{
    let cell1: (usize, usize) = range.0;
    let cell2: (usize, usize) = range.1;
    let mut summ = 0 ;
    let mut count = 0;
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            summ += cell_ref.value;
            count += 1;
        }
    }

    let mean = summ / count;
    let mut sum_squared_diff = 0;
    for col_rc in data.iter().skip(cell1.1).take(cell2.1 - cell1.1 + 1) {
        let col_ref = col_rc.borrow();
        for row in cell1.0..=cell2.0 {
            let cell_rc = Rc::clone(&col_ref[row]);
            let cell_ref = cell_rc.borrow();
            if !cell_ref.valid {
                return Err(-1);
            }
            sum_squared_diff += (cell_ref.value - mean).pow(2);
        }
    }

    let stdev = (sum_squared_diff as f64/ count as f64).sqrt().round();
    Ok(stdev as i32)
}

fn sleep(seconds: i32)
{
    thread::sleep(Duration::from_secs_f32(seconds as f32));
}

fn remove_old_dependencies(cell: &Addr,sheet:&mut Sheet, dependencies: Vec<ParentType>)     // DEPENDENCIES OF OLD_FUNC
{
    for i in dependencies
    {
        match i 
        {
            ParentType::Single(addr) => 
            {
               
                let Addr { row, col } = addr;

                let column_ref = &sheet.data[col];
                let column = column_ref.borrow_mut();

                let cell_rc = Rc::clone(&column[row]);
                drop(column);
                let mut parent_cell = cell_rc.borrow_mut();

                parent_cell.children.remove(cell);
                drop(parent_cell); 
            },
            ParentType::Range(start, end) => 
            {
                let Addr{row:r1, col:c1} = start;
                let Addr{row:r2, col:c2} = end;
                for i in c1..=c2 
                {
                    for j in r1..=r2 
                    {
                        let column_ref = &sheet.data[i];
                        let column = column_ref.borrow_mut();

                        let cell_rc = Rc::clone(& column[j]);
                        drop(column);
                        let mut parent_cell = cell_rc.borrow_mut();
                        parent_cell.children.remove(cell);
                        drop(parent_cell); 
                    }
                }
            },
        }
    }
} 

fn eval_atomic_exp(atomic_expr: &AtomicExpr, sheet: &mut Sheet) -> Result<i32,i32> 
{
    match atomic_expr 
    {
        AtomicExpr::Integer(n) =>
        {
            Ok(*n)
        }
        AtomicExpr::Cell(addr) =>
        {
            let Addr {row, col } = addr;
            let col = *col;
            let row = *row;
            let column_ref = &sheet.data[col];
            let column = column_ref.borrow_mut();
            let cell_rc = Rc::clone(&column[row]);
            drop(column);
            let parent_cell = cell_rc.borrow();
            if !parent_cell.valid 
            {
                Err(-1)
            }
            else 
            {
                Ok(parent_cell.value)
            }
        }
    }
}

fn eval(expr: &Expr, sheet: &mut Sheet) -> Result<i32,i32> 
{
    match expr 
    {
        Expr::Atom(atomic_expr) => 
        {
            eval_atomic_exp(atomic_expr, sheet)
        },
        Expr::MonoOp(_, exp) =>
        {
            let sleep_val = eval_atomic_exp(exp, sheet)?;
            if sleep_val < 0
            {
                Err(-2)
            }
            else 
            {
                sleep(sleep_val);
                Ok(sleep_val)
            }
        }
        Expr::RangeOp{op,start, end} =>
        {
            match op 
            {
                RangeFunction::Min => 
                {
                    let range = ((start.row, start.col), (end.row,end.col));
                    min_eval(&sheet.data, range)
                },
                RangeFunction::Max => 
                {
                    let range = ((start.row, start.col), (end.row,end.col));
                    max_eval(&sheet.data, range)
                },
                RangeFunction::Sum => 
                {
                    let range = ((start.row, start.col), (end.row,end.col));
                    sum_eval(&sheet.data, range)
                },
                RangeFunction::Avg => 
                {
                    let range = ((start.row, start.col), (end.row,end.col));
                    avg_eval(&sheet.data, range)
                },
                RangeFunction::Stdev => 
                {
                    let range = ((start.row, start.col), (end.row,end.col));
                    stdev_eval(&sheet.data, range)
                },
            }
        }
        Expr:: BinOp(exp1,func , exp2 ) =>
        {
            match func 
            {
                BinaryFunction:: Mul =>
                {
                    let left = eval_atomic_exp(exp1, sheet)?;
                    let right = eval_atomic_exp(exp2, sheet)?;
                    Ok(left*right)
                },
                BinaryFunction::Add =>
                {
                    let left = eval_atomic_exp(exp1, sheet)?;
                    let right = eval_atomic_exp(exp2, sheet)?;
                    Ok(left+right)
                },
                BinaryFunction::Sub =>
                {
                    let left = eval_atomic_exp(exp1, sheet)?;
                    let right = eval_atomic_exp(exp2, sheet)?;
                    Ok(left-right)
                },
                BinaryFunction::Div =>
                {
                    let left = eval_atomic_exp(exp1, sheet)?;
                    let right = eval_atomic_exp(exp2, sheet)?;
                    if right == 0
                    {
                        Err(-1)
                    }
                    else
                    {
                        Ok(left/right)
                    }
                }
            }
        }
    }
}

fn calculate(cell_rc:Rc<RefCell<Cell>>, sheet: &mut Sheet) -> Result<(),i32>
{
    let temp: Rc<RefCell<Cell>> = Rc::clone(&cell_rc);
    let mut cell: std::cell::RefMut<'_, Cell> = temp.borrow_mut();
    let cell_func: &Option<CellFunc> = &cell.cell_func;
    match cell_func
    {
        Some(func) =>
        {   
            let expr = &func.expression;
            let temp = eval(expr, sheet);
            if let Err(err) = temp 
            {
                cell.valid = false;
                drop(cell);
                return Err(err);
            }
            else {
                let temp = temp.unwrap();
                cell.value = temp;
                cell.valid = true;
                drop(cell);
            }
            Ok(())
        }
        None => 
        {
            // print!("Cell is empty\n");
            cell.valid = true;
            drop(cell);
            Ok(())
        }
    }  
}

fn update_parent_avls(cell:&Addr, sheet: &mut Sheet, dependencies: Vec<ParentType>) 
{
    for i in dependencies
    {
        match i 
        {
            ParentType::Single(addr) => 
            { 
                let column_ref = &sheet.data[addr.col];
                let column = column_ref.borrow_mut();

                let cell_rc = Rc::clone(&column[addr.row]);
                drop(column);
                let mut parent_cell = cell_rc.borrow_mut();
                parent_cell.children.insert((cell).clone());
                drop(parent_cell); 
            },
            ParentType::Range(start, end) => 
            {
                let Addr{row:r1, col:c1} = start;
                let Addr{row:r2, col:c2} = end;
                for i in c1..=c2 
                {
                    for j in r1..=r2 
                    {
                        let column_ref = &sheet.data[i];
                        let column = column_ref.borrow_mut();

                        let cell_rc = Rc::clone(&column[j]);
                        drop(column);
                        let mut parent_cell = cell_rc.borrow_mut();
                        parent_cell.children.insert((cell).clone());
                        drop(parent_cell);
                    }
                }
            },
        }
        
    }
}


fn dfs(sheet: &mut Sheet,current_cell: &Addr, visited: &mut HashMap<Addr,bool>, rec_stack: &mut HashMap<Addr,bool>, stack: &mut Vec<Addr>) -> Result<(),i32>     
{
    rec_stack.insert(current_cell.clone(), true); 

    let column_ref = &sheet.data[current_cell.col];
    let column = column_ref.borrow_mut();

    let cell_rc = Rc::clone(&column[current_cell.row]);
    drop(column);
    let curr_cell = cell_rc.borrow();

    let ordered_set = curr_cell.children.clone();
    for i in &ordered_set
    {
        if rec_stack.contains_key(i) 
        {
            return Err(-3);
        }
        else if visited.contains_key(i) 
        {
            continue;
        }
        else 
        {
            dfs(sheet, i, visited, rec_stack, stack)?;
        }
    }
    visited.insert(current_cell.clone(), true); 
    rec_stack.remove(current_cell);
    stack.push(current_cell.clone()); 
    Ok(())
}

fn topological_sort(sheet: &mut Sheet, addr:&Addr) -> Result<Vec<Addr>,i32> 
{
    let mut visited: HashMap<Addr, bool> = HashMap::new();
    let mut rec_stack: HashMap<Addr, bool> = HashMap::new();
    let mut stack: Vec<Addr> = Vec::new();
    dfs(sheet, addr, &mut visited, &mut rec_stack, &mut stack)?;
    Ok(stack)
}


fn update_children(sheet: &mut Sheet, cell: &Addr) -> Result<(), i32> 
{
    let ret = topological_sort(sheet, cell)?;
    let mut error : Result<(),i32> = Ok(());
    for i in ret.iter().rev()
    {
        let column_ref = &sheet.data[i.col];
        let column = column_ref.borrow_mut();

        let cell_rc = Rc::clone(& column[i.row]);
        drop(column);
        let curr_cell = cell_rc.borrow();
        // let checker = curr_cell.cell_func.is_some();
        drop(curr_cell);
        // if checker
        // {
            let temp = calculate(cell_rc, sheet);
            if temp.is_err()
            { 
                error = temp;
            }
        // }
    }
    error?;

    Ok(())
    
}

pub fn evaluate(sheet: &mut Sheet, cell: &Addr, old_func: &Option<CellFunc>) -> Result<(), i32>   
{
    let cell_rc = {
        let column_ref = &sheet.data[cell.col];
        let column = column_ref.borrow_mut();
        Rc::clone(&column[cell.row])
    };
    let curr_cell = cell_rc.borrow();
    let cell_funcc = curr_cell.cell_func.clone();
    drop(curr_cell);
    let old_dependencies =  match old_func {
        Some(x) => x.expression.get_dependency_list(),
        None => vec![]
    };   
    let dependencies = match &cell_funcc {
        Some(func) => func.expression.get_dependency_list(),
        None => {
            vec![]
        }
    };
    remove_old_dependencies(cell, sheet, old_dependencies);
    update_parent_avls(cell, sheet, dependencies);
    let temp = update_children(sheet, cell);
    if let Err(i) = temp 
    {
        if i != -1{
            let mut curr_cell = cell_rc.borrow_mut();
            curr_cell.cell_func = old_func.clone();
            drop(curr_cell);
            evaluate(sheet,cell, &cell_funcc)?;
        }
        return Err(i);
    }    
    Ok(())
}