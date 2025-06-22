use spreadsheet::cell_operations::{
    Cell, CellFunc, Sheet,
};
use spreadsheet::ast::{
    Expr, AtomicExpr, Addr, RangeFunction, BinaryFunction, MonoFunction
};
use spreadsheet::evaluate_operations::evaluate;
use spreadsheet::tokens;
use spreadsheet::grammar;
#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use logos::Logos;


    // Helper function to create a new Sheet with cells
    fn create_test_sheet() -> Sheet {
        let sheet = Sheet::new(4, 4 );
        let cell1 = Rc::new(RefCell::new(Cell::new(Addr{row:0,col:0})));
        let cell2 = Rc::new(RefCell::new(Cell::new(Addr{row:1,col:0})));
        let cell3 = Rc::new(RefCell::new(Cell::new(Addr{row:0,col:1})));
        let cell4 = Rc::new(RefCell::new(Cell::new(Addr{row:1,col:1})));
        let cell5 = Rc::new(RefCell::new(Cell::new(Addr{row:2,col:0})));
        let cell6 = Rc::new(RefCell::new(Cell::new(Addr{row:0,col:2})));
        let cell7 = Rc::new(RefCell::new(Cell::new(Addr{row:2,col:1})));
        let cell8 = Rc::new(RefCell::new(Cell::new(Addr{row:1,col:2})));
        let cell9 = Rc::new(RefCell::new(Cell::new(Addr{row:2,col:2})));
        

        cell1.borrow_mut().cell_func = Some(CellFunc::new(
            Expr::Atom(AtomicExpr::Integer(5))
        ));

        
        cell2.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::Atom(AtomicExpr::Cell(Addr { row: 0, col: 0 })),
        });
        cell3.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::Atom(AtomicExpr::Integer(7)),
        });
        cell4.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::Atom(AtomicExpr::Integer(2)),
        });

        cell5.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::MonoOp(MonoFunction::Sleep, AtomicExpr::Cell(Addr { row: 0, col: 0 })),
        });
        cell6.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::BinOp(
                AtomicExpr::Cell(Addr { row: 0, col: 0 }),
                BinaryFunction::Add,
                AtomicExpr::Cell(Addr { row: 1, col: 1 }),
            ),
        });
        cell7.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::RangeOp {
                op: RangeFunction::Sum,
                start: Addr { row: 0, col: 0 },
                end: Addr { row: 1, col: 1 },
            },
        });
        cell8.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::RangeOp {
                op: RangeFunction::Min,
                start: Addr { row: 0, col: 0 },
                end: Addr { row: 1, col: 1 },
            },
        });
        cell9.borrow_mut().cell_func = Some(CellFunc{
            expression: Expr::RangeOp {
                op: RangeFunction::Stdev,
                start: Addr { row: 0, col: 0 },
                end: Addr { row: 1, col: 1 },
            },
        });
        
        sheet.data[0].borrow_mut()[0] = cell1;
        sheet.data[1].borrow_mut()[0] = cell3; 
        sheet.data[0].borrow_mut()[1] = cell2;
        sheet.data[1].borrow_mut()[1] = cell4;
        sheet.data[2].borrow_mut()[0] = cell6;
        sheet.data[0].borrow_mut()[2] = cell5;
        sheet.data[2].borrow_mut()[1] = cell8;
        sheet.data[1].borrow_mut()[2] = cell7;
        sheet.data[2].borrow_mut()[2] = cell9;


        sheet
    }

    // Test evaluating atomic expressions (simple integer and cell)
    #[test]
    fn test_eval_atomic_exp() {
        let mut sheet = create_test_sheet();
        if evaluate(&mut sheet, &Addr { row: 0, col: 0 }, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[0].borrow_mut()[0]));
        let target_cell_ref = target_cell_rc.borrow();
        assert!(target_cell_ref.value == 5);      
        drop(target_cell_ref);
        
        if evaluate(&mut sheet, &Addr { row: 1, col: 0 }, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[0].borrow_mut()[1]));
        let target_cell_ref = target_cell_rc.borrow();
        assert!(target_cell_ref.value == 5);      

    }

    // Test evaluating basic binary operations
    #[test]
    fn test_mono_op_sleep() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        if evaluate(&mut sheet, &Addr { row: 2, col: 0 }, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[0].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert!(target_cell_ref.value == 5);    
    }

    #[test]
    fn test_neg_sleep() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5
        sheet.data[0].borrow_mut()[0].borrow_mut().value = -5;
        if let Err(n) = evaluate(&mut sheet, &Addr { row: 2, col: 0 }, &None) {
            assert_eq!(n,-2);
        }
        else  
        {
            panic!("Negative value in sleep did not return error as expected");
        }  
    }

    #[test]
    fn test_eval_binop_add() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;

        if evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[2].borrow_mut()[0]));
        let target_cell_ref = target_cell_rc.borrow();
        assert!(target_cell_ref.value == 8); 
    }
    #[test]
    fn test_eval_binop_add_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Add,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc::new(expr));

        if let Err(n) = evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None) {
            assert_eq!(n,-1);
        }
        else  
        {
            panic!("Error not returned as expected");
        }
    }

    #[test]
    fn test_eval_binop_sub() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Sub,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[2].borrow_mut()[0]));
        let target_cell_ref = target_cell_rc.borrow();
        assert!(target_cell_ref.value == 2); 
    }

    #[test]
    fn test_eval_binop_sub_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Sub,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        let temp = evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None);
        if let Err(n) = temp {
            assert_eq!(n,-1);
        }
        else  
        {
            let target_cell_rc = Rc::clone(& (sheet.data[2].borrow_mut()[0]));
            let target_cell_ref = target_cell_rc.borrow();
            println!("\n\n {}\n\n", target_cell_ref.value);
            panic!("Error not returned as expected");
        }
    }


    #[test]
    fn test_eval_binop_mul() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Mul,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );

        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[2].borrow_mut()[0]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value,15); 
    }

    #[test]
    fn test_eval_binop_mul_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;
        
        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Mul,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if let Err(n) = evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None) {
            assert_eq!(n,-1);
        }
        else  
        {
            panic!("Error not returned as expected");
        }
    }

    #[test]
    fn test_eval_binop_div() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 6 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 6;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Div,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );

        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None).is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[2].borrow_mut()[0]));
        let target_cell_ref = target_cell_rc.borrow();
        // println!("Value: {}", target_cell_ref.value);
        assert_eq!(target_cell_ref.value, 2); 
    }

    #[test]
    fn test_eval_binop_div_by_zero() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 6 and cell (1,1) to 0 (division by zero)
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 6;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 0;

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Div,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        ); // Division by zero should return an error
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if let Err(n) = evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None) {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Division by zero did not return error as expected"); 
        }
    }

    #[test]
    fn test_eval_binop_div_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set cell (0,0) to 5 and cell (1,1) to 3
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 3;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;
        
        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Div,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[2].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });

        if let Err(n) = evaluate(&mut sheet, &Addr { row: 0, col: 2}, &None) {
            assert_eq!(n,-1);
        }
        else  
        {
            panic!("Error not returned as expected");
        }
    }

    // Test evaluating range functions (Min, Max, etc.)
    #[test]
    fn test_eval_range_min() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;

        let expr = Expr::RangeOp {
            op: RangeFunction::Min,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if result.is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[1].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value, 2); // Min of values in range (0,0) to (1,1) is 2
    }

    #[test]
    fn test_eval_range_min_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::RangeOp {
            op: RangeFunction::Min,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if let Err(n) = result {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Error not returned as expected"); 
        }
    }

    #[test]
    fn test_eval_range_max() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;

        let expr = Expr::RangeOp {
            op: RangeFunction::Max,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if result.is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[1].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value, 7); // Min of values in range (0,0) to (1,1) is 2
    }

    #[test]
    fn test_eval_range_max_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::RangeOp {
            op: RangeFunction::Max,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if let Err(n) = result {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Error not returned as expected");
        }
    }


    #[test]
    fn test_eval_range_avg() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;

        let expr = Expr::RangeOp {
            op: RangeFunction::Avg,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if result.is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[1].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value, 4); // Min of values in range (0,0) to (1,1) is 2
    }

    #[test]
    fn test_eval_range_avg_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::RangeOp {
            op: RangeFunction::Avg,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if let Err(n) = result {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Error not returned as expected");
        }
    }
    #[test]
    fn test_eval_range_sum() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;

        let expr = Expr::RangeOp {
            op: RangeFunction::Sum,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if result.is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[1].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value, 17); // Min of values in range (0,0) to (1,1) is 2
    }

    #[test]
    fn test_eval_range_sum_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::RangeOp {
            op: RangeFunction::Sum,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if let Err(n) = result {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Error not returned as expected");
        }
    }

    #[test]
    fn test_eval_range_stdev() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;

        let expr = Expr::RangeOp {
            op: RangeFunction::Stdev,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if result.is_err() {
            panic!("Error evaluating cell");
        }
        let target_cell_rc = Rc::clone(& (sheet.data[1].borrow_mut()[2]));
        let target_cell_ref = target_cell_rc.borrow();
        assert_eq!(target_cell_ref.value, 2); // Min of values in range (0,0) to (1,1) is 2
    }

    #[test]
    fn test_eval_range_stdev_one_cell_with_err() {
        let mut sheet = create_test_sheet();
        // Set some cells in a range to specific values
        sheet.data[0].borrow_mut()[0].borrow_mut().value = 5;
        sheet.data[1].borrow_mut()[0].borrow_mut().value = 3;
        sheet.data[0].borrow_mut()[1].borrow_mut().value = 7;
        sheet.data[1].borrow_mut()[1].borrow_mut().value = 2;
        sheet.data[1].borrow_mut()[1].borrow_mut().valid = false;

        let expr = Expr::RangeOp {
            op: RangeFunction::Stdev,
            start: Addr { row: 0, col: 0 },
            end: Addr { row: 1, col: 1 },
        };
        sheet.data[1].borrow_mut()[2].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 2, col: 1}, &None);
        if let Err(n) = result {
            assert_eq!(n,-1);
        }
        else 
        {
            panic!("Error not returned as expected"); 
        }
    }

    // Test circular dependencies
    #[test]
    fn test_circular_dependency() {
        let mut sheet = create_test_sheet();

        // Set up a circular dependency: cell (0,0) -> cell (1,1) -> cell (0,0)
        sheet.data[0].borrow_mut()[0].borrow_mut().children.insert(Addr { row: 1, col: 1 });
        sheet.data[1].borrow_mut()[1].borrow_mut().children.insert(Addr { row: 0, col: 0 });

        let expr = Expr::BinOp(
            AtomicExpr::Cell(Addr { row: 0, col: 0 }),
            BinaryFunction::Add,
            AtomicExpr::Cell(Addr { row: 1, col: 1 }),
        );
        sheet.data[0].borrow_mut()[0].borrow_mut().cell_func = Some(CellFunc{
            expression: expr,
        });
        let result = evaluate(&mut sheet, &Addr { row: 0, col: 0}, &None);
        if let Err(n) = result {
            assert_eq!(n,-3) // Circular dependency should return an error
        }
        else 
        {
            panic!("Error not returned as expected");
        }
    }

    #[test]
    fn test_parser() {
        let file = File::open("./tests/IOtestcases.txt").expect("Error opening test_cases file.");
        let reader = BufReader::new(file);
        let parser = grammar::CommandParser::new();
        
        for line_result in reader.lines() {
            let line = line_result.expect("Issue in running test.");
            let trimmed = line.trim();
    
            if trimmed.is_empty() {
                continue; // Ignore empty lines
            } else if trimmed.starts_with('#') {
                continue; // Ignore comment lines
            } else if trimmed.starts_with('!') {
                let lexer = tokens::Token::lexer(&trimmed[1..].trim()).spanned()
                .map(|(token_result, span)| {
                    let token = token_result?; // Propagate LexicalError
                    Ok((span.start, token, span.end)) // (usize, Token, usize)
                });
               match parser.parse(0, lexer) {
                Ok(_) => panic!("{}", &trimmed[1..].trim()),
                Err(_) => ()
               }
            } else {
                let lexer = tokens::Token::lexer(&trimmed.trim()).spanned()
                .map(|(token_result, span)| {
                    let token = token_result?; // Propagate LexicalError
                    Ok((span.start, token, span.end)) // (usize, Token, usize)
                });
               parser.parse(0, lexer).expect("Parsing failed");
            }
        }

    }

    #[test]
    fn print_test(){
        let sheet = create_test_sheet();
        let cell_rc = Rc::clone(& (sheet.data[0].borrow_mut()[0]));
        let cell_ref = cell_rc.borrow();
        let a = format!("{:?}", cell_ref.cell_func);
        let b = format!("{}", cell_ref.cell_func.clone().unwrap_or(CellFunc::new(Expr::Atom(AtomicExpr::Integer(0)))));

        let c = "Some(CellFunc{expression: Atom(Integer(5))}\n)".to_string();
        let d = "Atom(Integer(5))".to_string();

        let e = format!("{}", tokens::Token::Assign);
        let f = "Assign".to_string();
        assert_eq!(a,c);
        assert_eq!(b,d);
        assert_eq!(e,f);
    }


}
