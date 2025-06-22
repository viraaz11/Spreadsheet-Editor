use spreadsheet::ast::{Expr, AtomicExpr, Addr, MonoFunction, RangeFunction, BinaryFunction, ParentType};
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_integer_has_no_dependencies() {
        let expr = Expr::Atom(AtomicExpr::Integer(42));
        let deps = expr.get_dependency_list();
        assert!(deps.is_empty());  // AtomicExpr::Integer has no dependencies
    }

    #[test]
    fn test_atom_cell_has_single_dependency() {
        let addr = Addr { row: 2, col: 3 };
        let expr = Expr::Atom(AtomicExpr::Cell(addr.clone()));
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Single(addr)]);  // AtomicExpr::Cell has a single dependency
    }

    #[test]
    fn test_monoop_integer_has_no_dependencies() {
        let expr = Expr::MonoOp(MonoFunction::Sleep, AtomicExpr::Integer(10));
        let deps = expr.get_dependency_list();
        assert!(deps.is_empty());  // MonoOp with Integer has no dependencies
    }

    #[test]
    fn test_monoop_cell_has_dependency() {
        let addr = Addr { row: 0, col: 1 };
        let expr = Expr::MonoOp(MonoFunction::Sleep, AtomicExpr::Cell(addr.clone()));
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Single(addr)]);  // MonoOp with Cell has a single dependency
    }

    #[test]
    fn test_rangeop_has_range_dependency() {
        let start = Addr { row: 0, col: 0 };
        let end = Addr { row: 2, col: 2 };
        let expr = Expr::RangeOp {
            op: RangeFunction::Sum,
            start: start.clone(),
            end: end.clone(),
        };
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Range(start, end)]);  // RangeOp has a range dependency
    }

    #[test]
    fn test_binop_integer_integer_has_no_dependencies() {
        let expr = Expr::BinOp(
            AtomicExpr::Integer(4),
            BinaryFunction::Add,
            AtomicExpr::Integer(5),
        );
        let deps = expr.get_dependency_list();
        assert!(deps.is_empty());  // BinOp with Integers has no dependencies
    }

    #[test]
    fn test_binop_cell_integer_has_one_dependency() {
        let addr = Addr { row: 1, col: 1 };
        let expr = Expr::BinOp(
            AtomicExpr::Cell(addr.clone()),
            BinaryFunction::Sub,
            AtomicExpr::Integer(7),
        );
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Single(addr)]);  // BinOp with Cell and Integer has one dependency
    }

    #[test]
    fn test_binop_integer_cell_has_one_dependency() {
        let addr = Addr { row: 3, col: 2 };
        let expr = Expr::BinOp(
            AtomicExpr::Integer(9),
            BinaryFunction::Mul,
            AtomicExpr::Cell(addr.clone()),
        );
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Single(addr)]);  // BinOp with Integer and Cell has one dependency
    }

    #[test]
    fn test_binop_cell_cell_has_two_dependencies() {
        let addr1 = Addr { row: 1, col: 1 };
        let addr2 = Addr { row: 2, col: 2 };
        let expr = Expr::BinOp(
            AtomicExpr::Cell(addr1.clone()),
            BinaryFunction::Div,
            AtomicExpr::Cell(addr2.clone()),
        );
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![
            ParentType::Single(addr1),
            ParentType::Single(addr2),
        ]);  // BinOp with Cell and Cell has two dependencies
    }

    // Additional test case for ParentType Range (edge case)
    #[test]
    fn test_parent_type_range() {
        let start = Addr { row: 0, col: 0 };
        let end = Addr { row: 10, col: 10 };
        let expr = Expr::RangeOp {
            op: RangeFunction::Max,
            start: start.clone(),
            end: end.clone(),
        };
        let deps = expr.get_dependency_list();
        assert_eq!(deps, vec![ParentType::Range(start, end)]);  // Range with large range
    }

    // Additional tests for `Addr` ordering
    #[test]
    fn test_addr_ordering() {
        let addr1 = Addr { row: 1, col: 1 };
        let addr2 = Addr { row: 2, col: 2 };
        assert!(addr1 < addr2);  // Testing PartialOrd for Addr
    }

    #[test]
    fn test_addr_eq() {
        let addr1 = Addr { row: 2, col: 2 };
        let addr2 = Addr { row: 2, col: 2 };
        assert_eq!(addr1, addr2);  // Testing Eq for Addr
    }

    #[test]
    fn test_mono_function_sleep() {
        let sleep_func = MonoFunction::Sleep;
        assert_eq!(format!("{:?}", sleep_func), "Sleep");  // Test Debug for MonoFunction
    }

    #[test]
    fn test_range_function_sum() {
        let sum_func = RangeFunction::Sum;
        assert_eq!(format!("{:?}", sum_func), "Sum");  // Test Debug for RangeFunction
    }

    #[test]
    fn test_binary_function_add() {
        let add_func = BinaryFunction::Add;
        assert_eq!(format!("{:?}", add_func), "Add");  // Test Debug for BinaryFunction
    }

    #[test]
    fn test_parent_type_debug_single() {
        let addr = Addr { row: 2, col: 3 };
        let parent = ParentType::Single(addr.clone());
        let result = format!("{:?}", parent);
        assert_eq!(result, format!("Single({:?})", addr));  // Test fmt::Debug for ParentType::Single
    }

    #[test]
    fn test_parent_type_debug_range() {
        let start = Addr { row: 0, col: 0 };
        let end = Addr { row: 2, col: 2 };
        let parent = ParentType::Range(start.clone(), end.clone());
        let result = format!("{:?}", parent);
        assert_eq!(result, format!("Range({:?}, {:?})", start, end));  // Test fmt::Debug for ParentType::Range
    }

    #[test]
    fn test_monoop_integer_dependency_list() {
        let expr = Expr::MonoOp(MonoFunction::Sleep, AtomicExpr::Integer(10));
        let deps = expr.get_dependency_list();
        assert!(deps.is_empty());  // MonoOp with Integer has no dependencies (already tested but let's ensure full coverage)
    }

    #[test]
    fn test_binop_integer_integer_dependency_list() {
        let expr = Expr::BinOp(
            AtomicExpr::Integer(5),
            BinaryFunction::Add,
            AtomicExpr::Integer(10),
        );
        let deps = expr.get_dependency_list();
        assert!(deps.is_empty());  // BinOp with Integer and Integer has no dependencies (already tested but let's ensure full coverage)
    }

    #[test]
    fn test_addr_ordering_reverse() {
        let addr1 = Addr { row: 2, col: 2 };
        let addr2 = Addr { row: 1, col: 1 };
        assert!(addr2 < addr1);  // Ensure that ordering works in reverse as well
    }
}
