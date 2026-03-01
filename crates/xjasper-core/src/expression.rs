//! Expression parser and evaluator.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Field reference: $F{fieldName}
    FieldRef(String),
    /// Variable reference: $V{variableName}
    VariableRef(String),
    /// Static string literal
    Literal(String),
}

#[derive(Debug, Error)]
pub enum ExpressionError {
    #[error("Invalid expression syntax: {0}")]
    SyntaxError(String),
}

pub type Result<T> = std::result::Result<T, ExpressionError>;

/// Parse an expression string
pub fn parse_expression(expr: &str) -> Result<Expression> {
    let trimmed = expr.trim();

    // Check for field reference: $F{fieldName}
    if let Some(field_name) = parse_field_ref(trimmed) {
        return Ok(Expression::FieldRef(field_name));
    }

    // Check for variable reference: $V{variableName}
    if let Some(var_name) = parse_variable_ref(trimmed) {
        return Ok(Expression::VariableRef(var_name));
    }

    // Otherwise, treat as literal
    Ok(Expression::Literal(trimmed.to_string()))
}

/// Parse field reference: $F{fieldName}
fn parse_field_ref(expr: &str) -> Option<String> {
    if expr.starts_with("$F{") && expr.ends_with('}') {
        let field_name = &expr[3..expr.len() - 1];
        Some(field_name.to_string())
    } else {
        None
    }
}

/// Parse variable reference: $V{variableName}
fn parse_variable_ref(expr: &str) -> Option<String> {
    if expr.starts_with("$V{") && expr.ends_with('}') {
        let var_name = &expr[3..expr.len() - 1];
        Some(var_name.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_ref() {
        let expr = "$F{customerName}";
        let result = parse_expression(expr).unwrap();
        assert_eq!(result, Expression::FieldRef("customerName".to_string()));
    }

    #[test]
    fn test_parse_variable_ref() {
        let expr = "$V{total}";
        let result = parse_expression(expr).unwrap();
        assert_eq!(result, Expression::VariableRef("total".to_string()));
    }

    #[test]
    fn test_parse_literal() {
        let expr = "Hello World";
        let result = parse_expression(expr).unwrap();
        assert_eq!(result, Expression::Literal("Hello World".to_string()));
    }
}
