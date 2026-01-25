use crate::ast::*;
use crate::symbol_table::SymbolTable;
use std::collections::HashMap;
use std::thread::scope;

fn check_scope_expr(expr: &Expr, scope_id: u32, symbol_table: &SymbolTable) -> Result<(), String> {
    match expr {
        Expr::BinaryOperation { op, left, right } => {
            check_scope_expr(left, scope_id, symbol_table)?;
            check_scope_expr(right, scope_id, symbol_table)?;
            Ok(())
        }
        Expr::Variable(var_name) => {
            if let None = symbol_table.get(scope_id, var_name) {
                return Err(format!(
                    "Undefined variable {:} in scope {:}",
                    var_name, scope_id
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_scope(scope: &Scope, symbol_table: &SymbolTable) -> Result<(), String> {
    for s in scope.statements.iter() {
        match s {
            Statement::Return(expr)
            | Statement::Expression(expr)
            | Statement::VarDeclare {
                value: Some(expr), ..
            } => check_scope_expr(&expr, scope.id, symbol_table)?,
            Statement::If {
                condition,
                true_block,
                false_block,
            } => {
                check_scope_expr(condition, scope.id, symbol_table)?;
                check_scope(true_block, symbol_table)?;
                if let Some(false_scope) = false_block {
                    check_scope(false_scope, symbol_table)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn check_types() {}

pub fn check_syntax(declarations: &Vec<Declaration>) -> Result<SymbolTable, String> {
    // For now, we're only considering programs with a single declaration: a main function
    assert_eq!(declarations.len(), 1);

    let symbol_table = SymbolTable::from_function(&declarations[0])?;
    let Declaration::Function {
        name,
        args,
        return_type,
        scope,
    } = &declarations[0];

    check_scope(&scope, &symbol_table)?;
    Ok(symbol_table)
}

mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::symbol_table;
    use crate::tokenizer::tokenize;
    use std::fs::read_to_string;

    #[test]
    fn test_symantic_main() -> Result<(), String> {
        let s = read_to_string("test/main.c").unwrap();
        let tokens = tokenize(&s)?;
        let syntax_tree = parse(&tokens)?;
        assert_eq!(1, syntax_tree.len());

        let symbol_table = check_syntax(&syntax_tree)?;
        Ok(())
    }

    #[test]
    fn test_symantic_main_undef_var() -> Result<(), String> {
        let s = read_to_string("test/main_undef_var.c").unwrap();
        let tokens = tokenize(&s)?;
        let syntax_tree = parse(&tokens)?;
        assert_eq!(syntax_tree.len(), 1);

        assert_eq!(
            check_syntax(&syntax_tree),
            Err("Undefined variable z in scope 1".to_owned())
        );
        Ok(())
    }
}
