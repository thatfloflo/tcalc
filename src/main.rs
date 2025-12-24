#![allow(dead_code)]

mod patterns;
mod types;
use types::{TokenType, Value, Position, ParseTree, ParseError, ParseNode, Token};

fn copy_while(input: &Vec<char>, charset: &str, start: usize, buf: &mut Vec<char>) {
    for character in &input[start..] {
        if charset.contains(*character) {
            buf.push(*character);
        } else {
            break;
        }
    }
}

fn copy_matchedspan(
    input: &Vec<char>,
    opening_char: char,
    closing_char: char,
    start: usize,
    buf: &mut Vec<char>,
) -> Result<(), ParseError> {
    let mut parens: usize = 1;
    for character in &input[start..] {
        if *character == opening_char {
            parens += 1;
        } else if *character == closing_char {
            parens -= 1;
        }
        if parens < 1 {
            break;
        }
        if parens > 0 {
            buf.push(*character);
        }
    }
    if parens > 0 {
        return Err(ParseError {
            msg: "Could not match open parenthesis with closing parenthesis".to_string(),
            position: Position {
                line: 0,
                chr: start,
            },
        });
    }
    return Ok(());
}

fn tokenize(
    input: String,
    line: Option<usize>,
    chr: Option<usize>,
    tree: &mut ParseTree,
) -> Result<(), ParseError> {
    let input: Vec<char> = input.chars().collect();
    let line: usize = line.unwrap_or(0);
    let chr: usize = chr.unwrap_or(0);
    let mut buf: Vec<char> = Vec::new();
    let mut i: usize = 0;
    while i < input.len() {
        if patterns::IGNORABLE_WHITESPACE_CHARS.contains(input[i]) {
            // do naught
        } else if input[i] == '(' {
            // Match TokenType.Expression
            // Find matching closing parenthesis and consume input along the way
            if let Err(e) = copy_matchedspan(&input, '(', ')', i + 1, &mut buf) {
                return Err(ParseError {
                    msg: e.msg,
                    position: Position {
                        line: line,
                        chr: chr + i,
                    },
                });
            }
            let token = Token {
                type_: TokenType::Expression,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            };
            let mut subtree = ParseTree::new_subtree(tree.level() + 1);
            match tokenize(
                token.content_to_string(),
                Some(line),
                Some(chr + i),
                &mut subtree,
            ) {
                Err(e) => return Err(e),
                Ok(_) => tree.push_subtree(token, subtree),
            }
            i += buf.len() + 1; // Skip the closing paren
            buf.clear();
        } else if patterns::NUMERAL_INITIAL_CHARS.contains(input[i]) {
            // Match TokenType.Numeral
            buf.push(input[i]);
            copy_while(&input, patterns::NUMERAL_INTERNAL_CHARS, i + 1, &mut buf);
            let token_type: TokenType;
            if buf.contains(&'.') || buf.contains(&',') {
                token_type = TokenType::Rational;
            } else if buf.starts_with(&['0', 'b']) {
                token_type = TokenType::Bitseq;
            } else {
                token_type = TokenType::Integer;
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if patterns::IDENTIFIER_INITIAL_CHARS.contains(input[i]) {
            // Match TokenType.Identifier
            buf.push(input[i]);
            copy_while(&input, patterns::IDENTIFIER_INTERNAL_CHARS, i + 1, &mut buf);
            let token_type: TokenType;
            let buf_string = buf.iter().collect::<String>();
            if patterns::BUILTIN_UNARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                token_type = TokenType::UnaryFunctionIdentifier;
            } else if patterns::BUILTIN_BINARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                token_type = TokenType::BinaryFunctionIdentifier;
            } else {
                token_type = TokenType::VariableIdentifier;
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if patterns::OPERATOR_INITIAL_CHARS.contains(input[i]) {
            // Match TokenType.Operator
            buf.push(input[i]);
            copy_while(&input, patterns::OPERATOR_INTERNAL_CHARS, i + 1, &mut buf);
            let token_type: TokenType;
            let buf_string = buf.iter().collect::<String>();
            if patterns::AMBIGUOUS_OPERATORS.contains(&buf_string.as_str()) {
                token_type = TokenType::AmbiguousOperator;
            } else if patterns::UNARY_OPERATORS.contains(&&buf_string.as_str()) {
                token_type = TokenType::UnaryOperator;
            } else if patterns::BINARY_OPERATORS.contains(&&buf_string.as_str()) {
                token_type = TokenType::BinaryOperator;
            } else {
                return Err(ParseError {
                    msg: format!("Unknown operator '{}'", buf_string),
                    position: Position {
                        line: line,
                        chr: chr + i,
                    },
                });
            }
            tree.push_token(Token {
                type_: token_type,
                content: buf.clone(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
            i += buf.len() - 1;
            buf.clear();
        } else if input[i] == ')' {
            return Err(ParseError {
                msg: "Unexpected closing parenthesis".to_string(),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
        } else {
            return Err(ParseError {
                msg: format!("Unknown character '{}'", input[i]),
                position: Position {
                    line: line,
                    chr: chr + i,
                },
            });
        }
        i += 1;
    }

    if let Err(e) = disambiguate_operators(tree) {
        return Err(e);
    }

    return expose_implicit_multiplications(tree);
}

fn expose_implicit_multiplications(tree: &mut ParseTree) -> Result<(), ParseError> {
    let mut i: usize = 0;
    while i + 1 < tree.len() {
        let is_value = match tree[i].token.type_ {
            TokenType::AmbiguousOperator => false,
            TokenType::BinaryFunctionIdentifier => false,
            TokenType::BinaryOperator => false,
            TokenType::Bitseq => true,
            TokenType::Expression => true,
            TokenType::ImplicitMultiplication => false,
            TokenType::Integer => true,
            TokenType::Rational => true,
            TokenType::UnaryFunctionIdentifier => false,
            TokenType::UnaryOperator => tree[i].token.content == vec!['!'],
            TokenType::VariableIdentifier => true,
        };
        let next_is_value = match tree[i + 1].token.type_ {
            TokenType::AmbiguousOperator => false, // We just don't know, so they ought to be disambiguated first
            TokenType::BinaryFunctionIdentifier => false,
            TokenType::BinaryOperator => false,
            TokenType::Bitseq => true,
            TokenType::Expression => true,
            TokenType::ImplicitMultiplication => false,
            TokenType::Integer => true,
            TokenType::Rational => true,
            TokenType::UnaryFunctionIdentifier => true,
            TokenType::UnaryOperator => tree[i + 1].token.content != vec!['!'],
            TokenType::VariableIdentifier => true,
        };
        if is_value && next_is_value {
            let token = Token {
                type_: TokenType::ImplicitMultiplication,
                content: vec![],
                position: tree[i + 1].token.position,
            };
            tree.insert(i + 1, ParseNode::new_from_token(token, None));
            i += 1;
        }
        i += 1;
    }
    return Ok(());
}

fn disambiguate_operators(tree: &mut ParseTree) -> Result<(), ParseError> {
    let mut i: usize = 0;
    while i < tree.len() {
        if tree[i].token.type_ == TokenType::AmbiguousOperator {
            let has_left_value: bool;
            if i < 1 {
                has_left_value = tree.level() == 0;
            } else {
                has_left_value = match tree[i - 1].token.type_ {
                    TokenType::AmbiguousOperator => false,
                    TokenType::BinaryFunctionIdentifier => false,
                    TokenType::BinaryOperator => false,
                    TokenType::Bitseq => true,
                    TokenType::Expression => true,
                    TokenType::ImplicitMultiplication => false,
                    TokenType::Integer => true,
                    TokenType::Rational => true,
                    TokenType::UnaryFunctionIdentifier => false,
                    TokenType::UnaryOperator => tree[i - 1].token.content == vec!['!'],
                    TokenType::VariableIdentifier => true,
                };
            }
            let has_right_value: bool;
            if i + 1 >= tree.len() {
                has_right_value = false; // +/- cannot be at end of expressions
            // Really just return ParseError here?
            } else {
                has_right_value = match tree[i + 1].token.type_ {
                    TokenType::AmbiguousOperator => true, // Will necessarily disambiguate to UnaryOp later
                    TokenType::BinaryFunctionIdentifier => true,
                    TokenType::BinaryOperator => false,
                    TokenType::Bitseq => true,
                    TokenType::Expression => true,
                    TokenType::ImplicitMultiplication => false,
                    TokenType::Integer => true,
                    TokenType::Rational => true,
                    TokenType::UnaryFunctionIdentifier => true,
                    TokenType::UnaryOperator => {
                        if tree[i + 1].token.content == vec!['!'] {
                            return Err(ParseError {
                                msg: format!(
                                    "Ambiguous operator '{}' cannot precede unary operator '!'",
                                    tree[i].token.content_to_string()
                                ),
                                position: tree[i].token.position,
                            });
                        }
                        true
                    }
                    TokenType::VariableIdentifier => true,
                };
            }
            if has_left_value == true && has_right_value == true {
                tree[i].token.type_ = TokenType::BinaryOperator;
            } else if has_left_value == false && has_right_value == true {
                tree[i].token.type_ = TokenType::UnaryOperator;
            } else {
                return Err(ParseError {
                    msg: format!(
                        "Could not disambiguate ambiguous operator '{}', consider using parentheses",
                        tree[i].token.content_to_string()
                    ),
                    position: tree[i].token.position,
                });
            }
        }
        i += 1;
    }
    return Ok(());
}

fn resolve_numerals(tree: &mut ParseTree) -> Result<(), ParseError> {
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

fn resolve(tree: &mut ParseTree) -> Result<(), ParseError> {
    // - Resolve subexpressions to values (if any)
    // - Resolve numerals to values
    if let Err(e) = resolve_numerals(tree) {
        return Err(e);
    }
    // - Resolve variable identifiers to values
    // - Resolve unary operators to values (precedence, then RTL)
    // - Resolve unary functions to values (RTL)
    // - Resolve binary operators to values (precedence, then RTL)
    // - Resolve binary functios to values (RTL)
    Ok(())
}

fn main() {
    //let input = "0b1001101.100101 + 83_382_292x22 / 0b000101 * (0xDEADBEEF0 - D17,343 (28.1 + 3)) + sqrt(1+7)";
    let input = "0b0010010 - 2.55 0D587 0b010.01 ( - 7)";
    let mut parse_tree = ParseTree::new();
    println!("INPUT: {}", input);
    tokenize(input.to_string(), None, None, &mut parse_tree).unwrap();
    resolve(&mut parse_tree).unwrap();
    println!("===== PARSED TOKENS =====");
    println!("{}", parse_tree);
}
