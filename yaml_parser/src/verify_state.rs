use super::{Input, State};
use winnow::{
    combinator::trace,
    error::{ErrMode, ParserError},
    ModalParser,
};

pub(super) fn verify_state<'s, E, F>(mut predicate: F) -> impl ModalParser<Input<'s>, (), E>
where
    E: ParserError<Input<'s>>,
    F: FnMut(&State) -> bool,
{
    trace("verify_state", move |input: &mut Input<'s>| {
        if predicate(&input.state) {
            Ok(())
        } else {
            Err(ErrMode::Backtrack(E::from_input(input)))
        }
    })
}
