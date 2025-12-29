#![allow(dead_code)]

mod parser;
mod patterns;
mod types;

use std::collections::HashMap;

use crate::types::{Ast, Position, SyntaxError, TokenType, Value};
use crate::parser::Parser;

//use crate::patterns;

pub struct VariableStore {
    pub map: HashMap<String, Value>,
}

impl VariableStore {
    fn new() -> Self {
        let mut vs = VariableStore {
            map: HashMap::new(),
        };
        vs.set(
            "test",
            Value::from_str("12345.06789", Position::default()).unwrap(),
        );
        vs.set(
            "\\blah",
            Value::from_str("0b000101011", Position::default()).unwrap(),
        );
        vs
    }

    fn set(&mut self, identifier: &str, value: Value) {
        self.map.insert(identifier.to_string(), value);
    }

    fn get(&self, identifier: &str) -> Option<&Value> {
        self.map.get(&identifier.to_string())
    }

    fn clear(&mut self) {
        self.map.clear();
    }
}

fn resolve_numerals(tree: &mut Ast) -> Result<(), SyntaxError> {
    let mut i: usize = 0;
    while i < tree.len() {
        if tree[i].token.type_ == TokenType::Bitseq
            || tree[i].token.type_ == TokenType::Integer
            || tree[i].token.type_ == TokenType::Rational
        {
            match Value::from_str(&tree[i].token.content_to_string(), tree[i].token.position) {
                Ok(v) => {
                    tree[i].value = Some(v);
                }
                Err(e) => return Err(e),
            };
        }
        i += 1;
    }
    Ok(())
}

fn resolve_variables(tree: &mut Ast, vs: &VariableStore) -> Result<(), SyntaxError> {
    let mut i: usize = 0;
    while i < tree.len() {
        if tree[i].token.type_ == TokenType::VariableIdentifier
            && (i + 1 >= tree.len()
                || tree[i + 1].token.type_ != TokenType::BinaryOperator
                || tree[i + 1].token.content != vec![':', '='])
        {
            match vs.get(&tree[i].token.content_to_string()) {
                Some(v) => tree[i].value = Some(v.clone()),
                None => {
                    return Err(SyntaxError {
                        msg: format!(
                            "The variable identifier \"{}\" is undefined",
                            tree[i].token.content_to_string()
                        ),
                        position: tree[i].token.position,
                    });
                }
            };
        }
        i += 1;
    }
    Ok(())
}

fn resolve(tree: &mut Ast) -> Result<(), SyntaxError> {
    let vs = VariableStore::new();
    // - Resolve subexpressions to values (if any)
    // - Resolve numerals to values
    if let Err(e) = resolve_numerals(tree) {
        return Err(e);
    }
    // - Resolve variable identifiers to values
    if let Err(e) = resolve_variables(tree, &vs) {
        return Err(e);
    }
    // - Resolve unary operators to values (precedence, then RTL)
    // - Resolve unary functions to values (RTL)
    // - Resolve binary operators to values (precedence, then RTL)
    // - Resolve binary functios to values (RTL)
    Ok(())
}

fn main() {
    let input = "0b1001101.100101 + 83_382_292x22 / 0b000101 * (0xDEADBEEF0 - D17,343 (28.1 + 3)) + sqrt(1+7)";
    // Sample AST:
    //         ________________ + ________________
    //         |                                 |
    // 0b1001101.[...]     _____________________ + _______________________
    //                     |                                             |
    //         ___________ / ________                                  sqrt
    //         |                    |                                    |
    // 83_382_292x22     __________ * __________                     Expression
    //                   |                     |                         |
    //               0b000101              Expression              _____ + _____
    //                                           |                 |           |
    //                                __________ - __________      1           7
    //                                |                     |
    //                          0xDEADBEEF0     __________ (*) __________
    //                                          |                       |
    //                                          D           __________ (*) __________
    //                                                      |                       |
    //                                                   17,343                 Expression
    //                                                                              |
    //                                                                   __________ + __________
    //                                                                   |                     |
    //                                                                 28.1                    3
    //let input = "+ ~ sqrt ¬ + -test! \\blah foo := 0b0010010 - 2.55 0D587 0b010.01 (2 * 7)";
    let mut parser = Parser::new();
    println!("INPUT: {}", input);
    let ast = parser.parse(input, 0, 0).unwrap();
    //resolve(&mut parse_tree).unwrap();
    println!("===== Abstract Syntax Tree =====");
    println!("{}", ast);
}
