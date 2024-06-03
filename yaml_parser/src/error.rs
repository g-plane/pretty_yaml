use crate::Input;
use std::fmt;
use winnow::error::{ContextError, ParseError};

#[derive(Clone, Debug)]
pub struct SyntaxError<'s>(ParseError<Input<'s>, ContextError>);

impl SyntaxError<'_> {
    #[inline]
    pub fn offset(&self) -> usize {
        self.0.offset()
    }

    #[inline]
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
