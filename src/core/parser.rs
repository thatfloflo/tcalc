use std::fmt::Display;

use crate::core::ast::{Ast, AstNode};
use crate::core::errors::SyntaxError;
use crate::core::patterns;
use crate::core::tokens::{Token, TokenType};

pub struct Parser {
    pub ast: Ast,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.ast = Ast::new();
    }

    pub fn take_ast(&mut self) -> Ast {
        std::mem::take(&mut self.ast)
    }

    fn _copy_while(input: &Vec<char>, charset: &str, start: usize, buf: &mut Vec<char>) {
        for character in &input[start..] {
            if charset.contains(*character) {
                buf.push(*character);
            } else {
                break;
            }
        }
    }

    fn _copy_matchedspan(
        input: &Vec<char>,
        opening_char: char,
        closing_char: char,
        start: usize,
        buf: &mut Vec<char>,
    ) -> Result<(), SyntaxError> {
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
            return Err(SyntaxError::new(
                "Could not match open parenthesis with closing parenthesis",
                Position::new(0, start),
            ));
        }
        Ok(())
    }

    pub fn parse<S: AsRef<str>>(
        &mut self,
        input: S,
        line: usize,
        chr: usize,
    ) -> Result<Ast, SyntaxError> {
        let input = input.as_ref().to_string();
        if let Err(e) = Self::_parse_recursively(input, line, chr, &mut self.ast) {
            return Err(e);
        }
        Ok(self.take_ast())
    }

    fn _parse_recursively(
        input: String,
        line: usize,
        chr: usize,
        tree: &mut Ast,
    ) -> Result<(), SyntaxError> {
        if let Err(e) = Self::tokenize(input, line, chr, tree) {
            return Err(e);
        }
        let mut i: usize = 0;
        while i < tree.len() {
            if tree[i].token.type_ == TokenType::Expression {
                let mut subtree = Ast::new();
                subtree.relevel_from(tree.level() + 1);
                match Self::_parse_recursively(
                    tree[i].token.content_to_string(),
                    line,
                    tree[i].token.position.chr + 1,
                    &mut subtree,
                ) {
                    Err(e) => return Err(e),
                    Ok(_) => tree[i].set_subtree(subtree),
                }
            }
            i += 1;
        }

        if let Err(e) = Self::disambiguate_operators(tree) {
            return Err(e);
        }

        if let Err(e) = Self::expose_implicit_multiplications(tree) {
            return Err(e);
        }

        if let Err(e) = Self::expose_implicit_mem0_call(tree) {
            return Err(e);
        }

        if let Err(e) = Self::incorporate_operands(tree) {
            return Err(e);
        }

        Ok(())
    }

    pub fn tokenize(
        input: String,
        line: usize,
        chr: usize,
        tree: &mut Ast,
    ) -> Result<(), SyntaxError> {
        let input: Vec<char> = input.chars().collect();
        let mut buf: Vec<char> = Vec::new();
        let mut i: usize = 0;
        while i < input.len() {
            if patterns::IGNORABLE_WHITESPACE_CHARS.contains(input[i]) {
                // do naught
            } else if input[i] == '(' {
                // Match TokenType.Expression
                // Find matching closing parenthesis and consume input along the way
                if let Err(e) = Self::_copy_matchedspan(&input, '(', ')', i + 1, &mut buf) {
                    return Err(SyntaxError::new(e.msg, Position::new(line, chr + i)));
                }
                let token = Token::new(
                    TokenType::Expression,
                    buf.clone(),
                    Position::new(line, chr + i),
                );
                tree.push_token(token);
                i += buf.len() + 1; // Skip the closing paren
                buf.clear();
            } else if patterns::NUMERAL_INITIAL_CHARS.contains(input[i]) {
                // Match TokenType.Numeral
                buf.push(input[i]);
                Self::_copy_while(&input, patterns::NUMERAL_INTERNAL_CHARS, i + 1, &mut buf);
                let token_type: TokenType;
                if buf.contains(&'.') || buf.contains(&',') {
                    token_type = TokenType::Rational;
                } else if buf.starts_with(&['0', 'b']) {
                    token_type = TokenType::Bitseq;
                } else {
                    token_type = TokenType::Integer;
                }
                tree.push_token(Token::new(
                    token_type,
                    buf.clone(),
                    Position::new(line, chr + i),
                ));
                i += buf.len() - 1;
                buf.clear();
            } else if patterns::IDENTIFIER_INITIAL_CHARS.contains(input[i]) {
                // Match TokenType.Identifier
                buf.push(input[i]);
                Self::_copy_while(&input, patterns::IDENTIFIER_INTERNAL_CHARS, i + 1, &mut buf);
                let token_type: TokenType;
                let buf_string = buf.iter().collect::<String>();
                if patterns::BUILTIN_UNARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                    token_type = TokenType::UnaryFunctionIdentifier;
                } else if patterns::BUILTIN_BINARY_FUNCTIONS.contains(&&buf_string.as_str()) {
                    token_type = TokenType::BinaryFunctionIdentifier;
                } else {
                    token_type = TokenType::VariableIdentifier;
                }
                tree.push_token(Token::new(
                    token_type,
                    buf.clone(),
                    Position::new(line, chr + i),
                ));
                i += buf.len() - 1;
                buf.clear();
            } else if patterns::OPERATOR_INITIAL_CHARS.contains(input[i]) {
                // Match TokenType.Operator
                buf.push(input[i]);
                Self::_copy_while(&input, patterns::OPERATOR_INTERNAL_CHARS, i + 1, &mut buf);
                let token_type: TokenType;
                let buf_string = buf.iter().collect::<String>();
                if patterns::AMBIGUOUS_OPERATORS.contains(&buf_string.as_str()) {
                    token_type = TokenType::AmbiguousOperator;
                } else if patterns::UNARY_OPERATORS.contains(&&buf_string.as_str()) {
                    token_type = TokenType::UnaryOperator;
                } else if patterns::BINARY_OPERATORS.contains(&&buf_string.as_str()) {
                    token_type = TokenType::BinaryOperator;
                } else {
                    return Err(SyntaxError::new(
                        format!("Unknown operator '{}'", buf_string),
                        Position::new(line, chr + i),
                    ));
                }
                tree.push_token(Token::new(
                    token_type,
                    buf.clone(),
                    Position::new(line, chr + i),
                ));
                i += buf.len() - 1;
                buf.clear();
            } else if input[i] == ')' {
                return Err(SyntaxError::new(
                    "Unexpected closing parenthesis",
                    Position::new(line, chr + i),
                ));
            } else {
                return Err(SyntaxError::new(
                    format!("Unknown character '{}'", input[i]),
                    Position::new(line, chr + i),
                ));
            }
            i += 1;
        }

        // if let Err(e) = Self::disambiguate_operators(tree) {
        //     return Err(e);
        // }

        // if let Err(e) = Self::expose_implicit_multiplications(tree) {
        //     return Err(e);
        // }

        // if let Err(e) = Self::expose_implicit_mem0_call(tree) {
        //     return Err(e);
        // }

        // if let Err(e) = Self::incorporate_operands(tree) {
        //     return Err(e);
        // }

        Ok(())
    }

    fn expose_implicit_multiplications(tree: &mut Ast) -> Result<(), SyntaxError> {
        let mut i: usize = 0;
        while i + 1 < tree.len() {
            let is_value = match tree[i].token.type_ {
                TokenType::UnaryOperator => tree[i].token.content == vec!['!'],
                TokenType::Bitseq
                | TokenType::Expression
                | TokenType::Integer
                | TokenType::Rational
                | TokenType::VariableIdentifier => true,
                _ => false,
            };
            let next_is_value = match tree[i + 1].token.type_ {
                TokenType::UnaryOperator => tree[i + 1].token.content != vec!['!'],
                TokenType::Bitseq
                | TokenType::Expression
                | TokenType::Integer
                | TokenType::Rational
                | TokenType::UnaryFunctionIdentifier
                | TokenType::VariableIdentifier => true,
                _ => false,
            };
            if is_value && next_is_value {
                let token = Token::new_implicit(
                    TokenType::BinaryOperator,
                    vec!['*'],
                    tree[i + 1].token.position,
                );
                tree.insert(i + 1, AstNode::new_from_token(token));
                i += 1;
            }
            i += 1;
        }
        Ok(())
    }

    fn disambiguate_operators(tree: &mut Ast) -> Result<(), SyntaxError> {
        let mut i: usize = 0;
        while i < tree.len() {
            if tree[i].token.type_ == TokenType::AmbiguousOperator {
                let has_left_value: bool;
                if i < 1 {
                    has_left_value = tree.level() == 0;
                } else {
                    has_left_value = match tree[i - 1].token.type_ {
                        TokenType::UnaryOperator => tree[i - 1].token.content == vec!['!'],
                        TokenType::Bitseq
                        | TokenType::Expression
                        | TokenType::Integer
                        | TokenType::Rational
                        | TokenType::VariableIdentifier => true,
                        _ => false,
                    };
                }
                let has_right_value: bool;
                if i + 1 >= tree.len() {
                    has_right_value = false; // +/- cannot be at end of expressions
                // Really just return ParseError here?
                } else {
                    has_right_value = match tree[i + 1].token.type_ {
                        TokenType::UnaryOperator => {
                            if tree[i + 1].token.content == vec!['!'] {
                                return Err(SyntaxError::new(
                                format!(
                                        "Ambiguous operator '{}' cannot precede unary operator '!'",
                                        tree[i].token.content_to_string()
                                    ),
                                    tree[i].token.position,
                                ));
                            }
                            true
                        }
                        TokenType::AmbiguousOperator // Will necessarily disambiguate to UnaryOp later
                        | TokenType::Bitseq
                        | TokenType::Expression
                        | TokenType::Integer
                        | TokenType::Rational
                        | TokenType::UnaryFunctionIdentifier
                        | TokenType::VariableIdentifier => true,
                        _ => false,
                    };
                }
                if has_left_value == true && has_right_value == true {
                    tree[i].token.type_ = TokenType::BinaryOperator;
                } else if has_left_value == false && has_right_value == true {
                    tree[i].token.type_ = TokenType::UnaryOperator;
                } else {
                    return Err(SyntaxError::new(
                        format!(
                            "Could not disambiguate ambiguous operator '{}', consider using parentheses",
                            tree[i].token.content_to_string()
                        ),
                        tree[i].token.position,
                    ));
                }
            }
            i += 1;
        }
        Ok(())
    }

    fn expose_implicit_mem0_call(tree: &mut Ast) -> Result<(), SyntaxError> {
        if tree.level() > 0 || tree.len() < 1 {
            return Ok(());
        }
        if tree[0].token.type_ == TokenType::BinaryFunctionIdentifier
            || tree[0].token.type_ == TokenType::BinaryOperator
        {
            let position = tree[0].token.position;
            tree.insert(0, Self::_generate_mem0_call(position));
            if tree[0].has_subtree() {
                let base_level = tree.level();
                tree[0]
                    .subtree
                    .as_mut()
                    .unwrap()
                    .relevel_from(base_level + 1);
            }
        }
        Ok(())
    }

    fn _generate_mem0_call(position: Position) -> AstNode {
        AstNode::new_with_subtree(
            Token::new_implicit(TokenType::Expression, "(mem 0)".chars().collect(), position),
            Ast::from(AstNode::new_with_subtree(
                Token::new_implicit(
                    TokenType::UnaryFunctionIdentifier,
                    vec!['m', 'e', 'm'],
                    position,
                ),
                Ast::from(AstNode::new_from_token(Token::new_implicit(
                    TokenType::Integer,
                    vec!['0'],
                    position,
                ))),
            )),
        )
    }

    pub fn incorporate_operands(tree: &mut Ast) -> Result<(), SyntaxError> {
        if let Err(e) = Self::_incorporate_factorials(tree) {
            return Err(e);
        }
        if let Err(e) = Self::_incorporate_unary_ops_and_funcs(tree) {
            return Err(e);
        }
        if let Err(e) = Self::_incorporate_binary_ops(tree) {
            return Err(e);
        }
        Ok(())
    }

    fn _incorporate_factorials(tree: &mut Ast) -> Result<(), SyntaxError> {
        // Go LTR so that "x! !"" -> (((x)!)!)
        let mut i: usize = 0;
        while i < tree.len() {
            if tree[i].token.type_ == TokenType::UnaryOperator && tree[i].token.content == &['!'] {
                if i < 1 {
                    return Err(SyntaxError::new(
                        "Unary operator '!' is missing a left-hand operand",
                        tree[i].token.position,
                    ));
                }
                i -= 1;
                let mut subtree = Ast::new();
                subtree.push(tree.remove(i));
                subtree.relevel_from(tree.level() + 1);
                tree[i].set_subtree(subtree);
            }
            i += 1;
        }

        return Ok(());
    }

    fn _incorporate_unary_ops_and_funcs(tree: &mut Ast) -> Result<(), SyntaxError> {
        // Go RTL so that "- +x" -> "(-(+(x)))"
        let mut i: usize = tree.len();
        if i < 1 {
            return Ok(());
        }
        loop {
            i -= 1;
            if (tree[i].token.type_ == TokenType::UnaryOperator && tree[i].token.content != &['!'])
                || tree[i].token.type_ == TokenType::UnaryFunctionIdentifier
            {
                let operand_i = i + 1;
                if operand_i >= tree.len() {
                    return Err(SyntaxError::new(
                        format!(
                            "Unary operator '{}' is missing a right-hand operand",
                            tree[i].token.content_to_string()
                        ),
                        tree[i].token.position,
                    ));
                }
                let mut subtree = Ast::from(tree.remove(operand_i));
                subtree.relevel_from(tree.level() + 1);
                tree[i].set_subtree(subtree);
                // We've pruned the tree behind the direction we're going, so no need to adjust i
            }
            if i == 0 {
                break;
            }
        }
        Ok(())
    }

    fn _incorporate_binary_ops(tree: &mut Ast) -> Result<(), SyntaxError> {
        for op_set in patterns::BINARY_OPERATOR_PRECEDENCE.iter() {
            if let Err(e) = Self::_incorporate_binary_op_set(tree, op_set) {
                return Err(e);
            }
        }
        Ok(())
    }

    fn _incorporate_binary_op_set(tree: &mut Ast, binops: &Vec<String>) -> Result<(), SyntaxError> {
        // Go RTL so that "a * b / c" -> "((a) * ((b) / (c)))"
        let mut i: usize = tree.len();
        if i < 1 {
            return Ok(());
        }
        loop {
            i -= 1;
            if tree[i].token.type_ == TokenType::BinaryOperator
                && binops.contains(&tree[i].token.content_to_string())
            {
                if i == 0 {
                    return Err(SyntaxError::new(
                        format!(
                            "Binary operator '{}' is missing a left-hand operand",
                            tree[i].token.content_to_string()
                        ),
                        tree[i].token.position,
                    ));
                }
                let left_operand_i: usize = i - 1;
                let right_operand_i: usize = i + 1;
                if right_operand_i >= tree.len() {
                    return Err(SyntaxError::new(
                        format!(
                            "Binary operator '{}' is missing a right-hand operand",
                            tree[i].token.content_to_string()
                        ),
                        tree[i].token.position,
                    ));
                }
                let mut operands = vec![tree.remove(right_operand_i), tree.remove(left_operand_i)];
                operands.reverse();
                let mut subtree = Ast::from(operands);
                subtree.relevel_from(tree.level() + 1);
                i -= 1; // Only -1 because we only adjust for the left_operand we removed
                tree[i].set_subtree(subtree);
            }
            if i == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self { ast: Ast::new() }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub chr: usize,
}

impl Position {
    pub fn new(line: usize, chr: usize) -> Self {
        Self { line, chr }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 0, chr: 0 }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.chr)
    }
}
