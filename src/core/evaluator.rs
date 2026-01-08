use crate::core::ast::{Ast, AstNode};
use crate::core::environment::Environment;
use crate::core::errors::SyntaxError;
use crate::core::tokens::TokenType;
use crate::core::values::Value;

pub struct Evaluator {
    pub environment: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut n = Self::default();
        n.environment
            .variables
            .set("foo", Value::from(i128::from(100)));
        n.environment
            .variables
            .set("D", Value::from(i128::from(1)));
        n
    }

    pub fn evaluate_node(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        if node.value.is_some() {
            return Ok(()); // No need to evaluate nodes that have already been valued
            // This should not normally happen anyways, so maybe add some reporting?
        }
        if node.token.type_.is_terminal() {
            if node.token.type_.is_numeral() {
                self._evaluate_numeral(node).unwrap();
            } else if node.token.type_.is_variable_identifier() {
                self._evaluate_variable(node).unwrap();
            }
            return Ok(());
        }
        if node.has_children() {
            for child in node.subtree.iter_mut() {
                self.evaluate_node(child)?;
            }
        }
        // if node.has_unvalued_children() {
        //     panic!("Failed to value all children on non-terminal AstNode");
        // }
        if !node.has_children() {
            panic!("Attempting to evaluate child-less non-terminal AstNode");
        }
        if node.token.type_.is_unary() {
            if node.subtree.len() != 1 {
                panic!(
                    "Attempting to evaluate unary operator that has {} children (expected 1)",
                    node.subtree.len()
                )
            }
            if node.token.type_.is_operator() {
                self._evaluate_unary_operator(node).unwrap();
            } else { // node.token.type_.is_function_identifier()
            }
        } else { // node.token.type_.is_binary()
        }
        Ok(())
    }

    pub fn evaluate(&mut self, ast: &mut Ast) -> Result<(), SyntaxError> {
        for node in ast.iter_mut() {
            self.evaluate_node(node)?;
        }
        // - Resolve subexpressions to values (if any)
        // - Resolve numerals to values
        // if let Err(e) = self._evaluate_numerals(ast) {
        //     return Err(e);
        // }
        // - Resolve variable identifiers to values
        // if let Err(e) = self._evaluate_variables(ast) {
        //     return Err(e);
        // }
        // - Resolve unary operators to values (precedence, then RTL)
        // - Resolve unary functions to values (RTL)
        // - Resolve binary operators to values (precedence, then RTL)
        // - Resolve binary functios to values (RTL)
        Ok(())
    }

    fn _evaluate_numeral(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        // if !node.token.type_.is_numeral() {
        //     panic!(
        //         "Attempting to evaluate node with token of type {} as numeral (source: {})",
        //         node.token.type_, node.token.position
        //     );
        // }
        node.value = Some(Value::from_str(
            &node.token.content_to_string(),
            node.token.position,
        )?);
        Ok(())
    }

    fn _evaluate_variable(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        // if !node.token.type_.is_variable_identifier() {
        //     panic!(
        //         "Attempting to evaluate node with token of type {} as variable (source: {})",
        //         node.token.type_, node.token.position
        //     )
        // }
        let identifier = node.token.content_to_string();
        match self.environment.variables.get(&identifier) {
            Some(value) => node.value = Some(value.clone()),
            None => {
                return Err(SyntaxError::new(
                    format!("The variable identifier \"{identifier}\" is undefined"),
                    node.token.position,
                ));
            }
        }
        Ok(())
    }

    fn _evaluate_unary_operator(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        let operand = node.subtree[0].value.as_ref().unwrap();
        let operator = node.token.content_to_string();
        let result = match operator.as_str() {
            "+" => { operand.unary_pos() },
            "-" => { operand.unary_neg() },
            "!" => { operand.factorial().unwrap() },
            "¬" => { operand.logical_neg() },
            "~" => { operand.bitwise_neg().unwrap() },
            _ => {
                return Err(SyntaxError::new(
                    format!("The operator \"{operator}\" is undefined"),
                    node.token.position,
                ));
            }
        };
        node.value = Some(result);
        Ok(())
    }

    // pub const UNARY_OPERATORS: &[&str] = &["+", "-", "!", "¬", "~"];
    // pub const BINARY_OPERATORS: &[&str] = &[
    //     "^", "*", "/", "%", "+", "-", "<=>", "<=", ">=", ":=", "<<<", ">>>", "<<", ">>", "<", ">",
    //     "!=", "==", "&&", "||", "??", "!?", "&", "|", "^|",
    // ];

    fn _evaluate_variables(&mut self, ast: &mut Ast) -> Result<(), SyntaxError> {
        let mut i: usize = 0;
        while i < ast.len() {
            if ast[i].token.type_ == TokenType::VariableIdentifier
                && (i + 1 >= ast.len()
                    || ast[i + 1].token.type_ != TokenType::BinaryOperator
                    || ast[i + 1].token.content != vec![':', '='])
            {
                match self
                    .environment
                    .variables
                    .get(&ast[i].token.content_to_string())
                {
                    Some(v) => ast[i].value = Some(v.clone()),
                    None => {
                        return Err(SyntaxError {
                            msg: format!(
                                "The variable identifier \"{}\" is undefined",
                                ast[i].token.content_to_string()
                            ),
                            position: ast[i].token.position,
                        });
                    }
                };
            }
            i += 1;
        }
        Ok(())
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            environment: Environment::default(),
        }
    }
}
