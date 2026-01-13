#![allow(dead_code)]

mod core;

use crate::core::evaluator::Evaluator;
use crate::core::parser::Parser;

fn main() {
    //let input = "0b1001101.100101 + 83_382_292_22 / 0b000101 * (0xDEADBEEF0 - D17,343 (28.1 + 3)) + sqrt(1+foo)";
    // Sample AST:
    //         ________________ + ________________
    //         |                                 |
    // 0b1001101.[...]     _____________________ + _______________________
    //                     |                                             |
    //         ___________ / ________                                  sqrt
    //         |                    |                                    |
    // 83_382_292_22     __________ * __________                     Expression
    //                   |                     |                         |
    //               0b000101              Expression              _____ + _____
    //                                           |                 |           |
    //                                __________ - __________      1        Var(foo)
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
    let mut evaluator = Evaluator::new();
    let input = "pi!";
    println!("INPUT: {}", input);
    let mut ast = parser.parse(input, 0, 0).unwrap();
    evaluator.evaluate(&mut ast).unwrap();
    println!("===== Abstract Syntax Tree =====");
    println!("{}", ast);
}
