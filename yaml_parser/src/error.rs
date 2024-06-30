use crate::Input;
use std::{error::Error, fmt};
use winnow::error::{ContextError, ParseError};

#[derive(Clone, Debug)]
/// Error type for syntax errors.
pub struct SyntaxError<'s>(ParseError<Input<'s>, ContextError>);

impl SyntaxError<'_> {
    #[inline]
    /// The location where parsing failed.
    ///
    /// **Note:** This is an offset, not an index, and may point to the end of input on eof errors.
    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    #[inline]
    /// Message describing something is invalid or expected something else.
    pub fn message(&self) -> String {
        self.0.inner().to_string()
    }
}

impl fmt::Display for SyntaxError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'s> From<ParseError<Input<'s>, ContextError>> for SyntaxError<'s> {
    fn from(err: ParseError<Input<'s>, ContextError>) -> Self {
        SyntaxError(err)
    }
}

impl Error for SyntaxError<'_> {}
