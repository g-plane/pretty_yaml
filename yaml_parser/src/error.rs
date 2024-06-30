use crate::Input;
use std::{error::Error, fmt};
use winnow::error::{ContextError, ParseError};

#[derive(Clone, Debug)]
/// Error type for syntax errors.
pub struct SyntaxError {
    input: String,
    offset: usize,
    message: String,
    code_frame: String,
}

impl SyntaxError {
    /// The input at the initial location when parsing started.
    pub fn input(&self) -> &str {
        &self.input
    }

    #[inline]
    /// The location where parsing failed.
    ///
    /// **Note:** This is an offset, not an index, and may point to the end of input on eof errors.
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    /// Message describing something is invalid or expected something else.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code_frame)
    }
}

impl<'s> From<ParseError<Input<'s>, ContextError>> for SyntaxError {
    fn from(err: ParseError<Input<'s>, ContextError>) -> Self {
        Self {
            input: err.input().to_string(),
            offset: err.offset(),
            message: err.inner().to_string(),
            code_frame: err.to_string(),
        }
    }
}

impl Error for SyntaxError {}
