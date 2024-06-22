use super::{Input, State};
use winnow::{
    combinator::trace,
    error::{ErrMode, ErrorKind, ParserError},
    Parser,
};

pub(super) fn verify_state<'s, E, F>(mut predicate: F) -> impl Parser<Input<'s>, (), E>
where
    E: ParserError<Input<'s>>,
    F: FnMut(&State) -> bool,
{
    trace("verify_state", move |input: &mut Input<'s>| {
        if predicate(&input.state) {
            Ok(())
        } else {
            Err(ErrMode::Backtrack(E::from_error_kind(
                input,
                ErrorKind::Verify,
            )))
        }
    })
}
