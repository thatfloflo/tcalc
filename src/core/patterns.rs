use lazy_static::lazy_static;
use regex::Regex;

macro_rules! vec_into {
    ($($x:expr),*) => (vec![$($x.into()),*]);
}

lazy_static! {
    pub static ref BASE_PREFIX: Regex = Regex::new(r"^0[bBdDoOxX]").unwrap();
    pub static ref BINARY_INTEGER: Regex = Regex::new(r"^0[bB][01_]*[01]$").unwrap();
    pub static ref BINARY_RATIONAL: Regex =
        Regex::new(r"^0[bB][01_]*[.,](?:[01_]*[01])?$").unwrap();
    pub static ref DECIMAL_INTEGER: Regex =
        Regex::new(r"^(?:0[dD]_?[0-9]|[0-9])(?:[0-9_]*[0-9])?$").unwrap();
    pub static ref DECIMAL_RATIONAL: Regex =
        Regex::new(r"^(?:0[dD]_?)?(?:[0-9]*|[0-9][0-9_]*)[.,](?:[0-9]*|[0-9_]*[0-9])$").unwrap();
    pub static ref HEXADECIMAL_INTEGER: Regex =
        Regex::new(r"^0[xX][0-9a-fA-F_]*[0-9a-fA-F]$").unwrap();
    pub static ref HEXADECIMAL_RATIONAL: Regex =
        Regex::new(r"^0[xX][0-9a-fA-F_]*[.,](?:[0-9a-fA-F_]*[0-9a-fA-F])?$").unwrap();
    pub static ref OCTAL_INTEGER: Regex = Regex::new(r"^0[oO][0-7_]*[0-7]$").unwrap();
    pub static ref OCTAL_RATIONAL: Regex =
        Regex::new(r"^0[oO][0-7_]*[.,](?:[0-7_]*[0-7])?$").unwrap();
    pub static ref BINARY_OPERATOR_PRECEDENCE: Vec<Vec<String>> = vec![
        vec_into!["^"],                          // Exponentiation
        vec_into!["*", "/", "%"],                // Multiplication, Division, Modulo
        vec_into!["+", "-"],                     // Addition, Subtraction
        vec_into!["<<", ">>", "<<<", ">>>"],     // Bit shifts
        vec_into!["&"],                          // Bitwise and
        vec_into!["|"],                          // Bitwise or
        vec_into!["^|"],                         // Bitwise xor
        vec_into![">", "<", "<=", ">=", "!=", "==", "<=>", "??", "!?"], // Comparisons
        vec_into!["&&", "||"],                   // Logical conjunction/disjunction
        vec_into![":="],                         // Assignment
    ];
}

pub const NUMERAL_INITIAL_CHARS: &str = "0123456789.,";
pub const NUMERAL_INTERNAL_CHARS: &str = "0123456789.,abcdefoxABCDEFOX_";
pub const IGNORABLE_WHITESPACE_CHARS: &str = " \t";
pub const OPERATOR_INITIAL_CHARS: &str = "+-!^*/%¬<>=:&|?~";
pub const OPERATOR_INTERNAL_CHARS: &str = OPERATOR_INITIAL_CHARS;
pub const IDENTIFIER_INITIAL_CHARS: &str = "abcdefghojklmnopqrstuvwxyzABCDEFGHOJKLMNOPQRSTUVWXYZ\\";
pub const IDENTIFIER_INTERNAL_CHARS: &str = IDENTIFIER_INITIAL_CHARS;

pub const AMBIGUOUS_OPERATORS: &[&str] = &["+", "-"];
pub const UNARY_OPERATORS: &[&str] = &["+", "-", "!", "¬", "~"];
pub const BINARY_OPERATORS: &[&str] = &[
    "^", "*", "/", "%", "+", "-", "<=>", "<=", ">=", ":=", "<<<", ">>>", "<<", ">>", "<", ">",
    "!=", "==", "&&", "||", "??", "!?", "&", "|", "^|",
];
pub const BUILTIN_UNARY_FUNCTIONS: &[&str] = &[
    "abs", "not", "sin", "cos", "tan", "cot", "sec", "csc", "exp", "ln", "lg", "log", "sqrt",
    "cbrt", "mem",
];
pub const BUILTIN_BINARY_FUNCTIONS: &[&str] = &["rt", "logb", "choose"];
pub const BUILTIN_VARIABLE_IDENTIFIERS: &[&str] = &[
    "\\inbase",
    "\\outbase",
    "\\showfracs",
    "\\precision",
    "pi",
    "e",
];
