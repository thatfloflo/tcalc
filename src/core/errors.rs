use std::error::Error;
use std::fmt::Display;

macro_rules! define_errors {
    ( $($err_ident:ident, $err_code:literal, $err_desc:literal);*; ) => {
        $(
            #[derive(Debug, Clone)]
            pub struct $err_ident {
                pub msg: String,
                pub position: InputPosition,
            }

            impl $err_ident {
                const CODE: i32 = $err_code;

                pub fn new<S: AsRef<str>>(msg: S) -> Self {
                    Self {
                        msg: msg.as_ref().to_string(),
                        position: Default::default(),
                    }
                }

                pub fn newp<S: AsRef<str>>(msg: S, position: InputPosition) -> Self {
                    Self {
                        msg: msg.as_ref().to_string(),
                        position,
                    }
                }

                pub fn with_position(self, position: InputPosition) -> Self {
                    Self {
                        position,
                        ..self
                    }
                }
            }

            impl Display for $err_ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    // let position_hint = if self.position.is_some() {
                    //     format!(" at {}", self.position.as_ref().unwrap())
                    // } else {
                    //     "".to_string()
                    // };
                    write!(f, "{}: {}{}", stringify!{$err_desc}, self.msg, self.position)
                }
            }

            impl Error for $err_ident {}
        )*

        #[derive(Debug, Clone)]
        pub enum TCalcErrorKind {
            $(
                $err_ident = $err_code,
            )*
        }

        impl Display for TCalcErrorKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    $(
                        Self::$err_ident => stringify!{$err_desc},
                    )*
                };
                write!(f, "{s}")
            }
        }

        $(
            impl From<$err_ident> for TCalcError {
                fn from(value: $err_ident) -> Self {
                    Self {
                        msg: value.msg,
                        kind: TCalcErrorKind::$err_ident,
                        position: value.position,
                    }
                }
            }
        )*
    };
}

define_errors! {
    // Identifier,         Error Code,   Description
    SyntaxError,           10,           "Syntax Error";
    ConversionError,       11,           "Conversion Error";
    InvalidOperationError, 12,           "Invalid Operation Error";
}

#[derive(Debug, Clone)]
pub struct TCalcError {
    msg: String,
    kind: TCalcErrorKind,
    position: InputPosition,
}

impl TCalcError {
    fn with_position(self, position: InputPosition) -> Self {
        Self { position, ..self }
    }
}

impl Display for TCalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} at {}", self.kind, self.msg, self.position)
    }
}

impl Error for TCalcError {}

#[derive(Debug, Clone)]
pub struct InputPosition {
    pub file: String,
    pub line: usize,
    pub chr: usize,
}

impl InputPosition {
    pub fn new<S: AsRef<str>>(file: S, line: usize, chr: usize) -> Self {
        Self {
            file: file.as_ref().to_string(),
            line,
            chr,
        }
    }

    pub fn is_default(self) -> bool {
        self.file == "unknown".to_string() && self.line == 0 && self.chr == 0
    }
}

impl Default for InputPosition {
    fn default() -> Self {
        Self::new("unknown", 0, 0)
    }
}

impl Display for InputPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.chr)
    }
}
