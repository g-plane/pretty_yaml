use super::{Input, State};
use winnow::{
    error::{ErrMode, ErrorKind, ParserError},
    PResult,
};

pub(super) fn verify_state<'s, E, F>(
    mut predicate: F,
) -> impl FnMut(&mut Input<'s>) -> PResult<(), E>
where
    E: ParserError<Input<'s>>,
    F: FnMut(&State) -> bool,
{
    move |input: &mut Input| {
        if predicate(&input.state) {
            Ok(())
        } else {
            Err(ErrMode::Backtrack(E::from_error_kind(
                input,
                ErrorKind::Verify,
            )))
        }
    }
}
