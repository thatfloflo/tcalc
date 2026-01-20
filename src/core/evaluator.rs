use crate::core::ast::{Ast, AstNode};
use crate::core::bitseqs::Bitseq;
use crate::core::decimals::AngleUnit;
use crate::core::environment::Environment;
use crate::core::errors::{SyntaxError, TCalcError};
use crate::core::tokens::TokenType;
use crate::core::values::Value;
use crate::unwrap_or_propagate;

pub struct Evaluator {
    pub environment: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut n = Self::default();
        n.environment
            .variables
            .set("foo", Value::from(Bitseq::ZERO));
        n.environment.variables.set("D", Value::from(Bitseq::ONE));
        n
    }

    pub fn evaluate_node(&mut self, node: &mut AstNode) -> Result<(), TCalcError> {
        if node.value.is_some() {
            return Ok(()); // No need to evaluate nodes that have already been valued
            // This should not normally happen anyways, so maybe add some reporting?
        }
        if node.token.type_.is_terminal() {
            if node.token.type_.is_numeral() {
                unwrap_or_propagate!(
                    self._evaluate_numeral(node),
                    position: node.token.position.clone()
                );
            } else if node.token.type_.is_variable_identifier() {
                unwrap_or_propagate!(
                    self._evaluate_variable(node),
                    position: node.token.position.clone()
                );
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
                    "Attempting to evaluate unary operation that has {} children (expected 1)",
                    node.subtree.len()
                )
            }
            if node.token.type_.is_operator() {
                unwrap_or_propagate!(
                    self._evaluate_unary_operator(node),
                    position: node.token.position.clone()
                );
            } else {
                // node.token.type_.is_function_identifier()
                unwrap_or_propagate!(
                    self._evaluate_unary_function_call(node),
                    position: node.token.position.clone()
                );
            }
        } else {
            // node.token.type_.is_binary()
            if node.subtree.len() != 2 {
                panic!(
                    "Attempting to evaluate binary operation that has {} children (expected 2)",
                    node.subtree.len()
                )
            }
            if node.token.type_.is_operator() {
                unwrap_or_propagate!(
                    self._evaluate_binary_operator(node),
                    position: node.token.position.clone()
                );
            } else {
                // node.token.type_.is_function_identifier()
                unwrap_or_propagate!(
                    self._evaluate_binary_function_call(node),
                    position: node.token.position.clone()
                );
            }
        }
        Ok(())
    }

    pub fn evaluate(&mut self, ast: &mut Ast) -> Result<(), TCalcError> {
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
        match Value::from_str(&node.token.content_to_string()) {
            Ok(v) => {
                node.value = Some(v);
                Ok(())
            }
            Err(e) => Err(e.with_position(node.token.position.clone())),
        }
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
                return Err(SyntaxError::newp(
                    format!("The variable \"{identifier}\" is undefined"),
                    node.token.position.clone(),
                ));
            }
        }
        Ok(())
    }

    fn _evaluate_unary_operator(&mut self, node: &mut AstNode) -> Result<(), TCalcError> {
        // pub const UNARY_OPERATORS: &[&str] = &["+", "-", "!", "¬", "~"];
        let operand = node.subtree[0].value.as_ref().unwrap();
        let operator = node.token.content_to_string();
        let result = match operator.as_str() {
            "+" => operand.unary_pos(),
            "-" => operand.unary_neg(),
            "!" => operand.factorial()?,
            "¬" => operand.logical_neg(),
            "~" => operand.bitwise_neg()?,
            _ => {
                return Err(SyntaxError::newp(
                    format!("The operator \"{operator}\" is undefined"),
                    node.token.position.clone(),
                )
                .into());
            }
        };
        node.value = Some(result);
        Ok(())
    }

    fn _evaluate_unary_function_call(&mut self, node: &mut AstNode) -> Result<(), TCalcError> {
        // pub const BUILTIN_UNARY_FUNCTIONS: &[&str] = &[
        //     "abs", "not", "sin", "cos", "tan", "cot", "sec", "csc", "exp", "ln", "lg", "log", "sqrt",
        //     "cbrt", "mem",
        // ];
        let operand = node.subtree[0].value.as_ref().unwrap();
        let func_identifier = node.token.content_to_string();
        println!("Evaluating unary function {func_identifier}( {operand} )");
        let result = match func_identifier.as_str() {
            "abs" => operand.abs(),
            "not" => operand.logical_neg(),
            "sin" => operand.sin(AngleUnit::Degrees).unwrap(),
            _ => {
                return Err(SyntaxError::new(format!(
                    "The function \"{func_identifier}\" is undefined"
                ))
                .into());
            }
        };
        node.value = Some(result);
        Ok(())
    }

    fn _evaluate_binary_operator(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        // pub const BINARY_OPERATORS: &[&str] = &[
        //     "^", "*", "/", "%", "+", "-", "<=>", "<=", ">=", ":=", "<<<", ">>>", "<<", ">>", "<", ">",
        //     "!=", "==", "&&", "||", "??", "!?", "&", "|", "^|",
        // ];
        todo!()
    }

    fn _evaluate_binary_function_call(&mut self, node: &mut AstNode) -> Result<(), SyntaxError> {
        // M rt N, M logb N, M choose N
        todo!()
    }

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
                        return Err(SyntaxError::newp(
                            format!(
                                "The variable identifier \"{}\" is undefined",
                                ast[i].token.content_to_string()
                            ),
                            ast[i].token.position.clone(),
                        ));
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
