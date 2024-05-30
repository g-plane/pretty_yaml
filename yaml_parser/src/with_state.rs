use super::{Input, State};
use std::marker::PhantomData;
use winnow::{PResult, Parser};

pub(super) struct WithState<'s, O, E, P>
where
    P: Parser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, (O, State), E> for WithState<'s, O, E, P>
where
    P: Parser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> PResult<(O, State), E> {
        match self.parser.parse_next(input) {
            Ok(output) => Ok((output, input.state.clone())),
            Err(err) => Err(err),
        }
    }
}

pub(super) trait ParserExt<'s, O, E, P>
where
    P: Parser<Input<'s>, O, E>,
{
    fn with_state(self) -> WithState<'s, O, E, P>;
}

impl<'s, O, E, P> ParserExt<'s, O, E, P> for P
where
    P: Parser<Input<'s>, O, E>,
{
    fn with_state(self) -> WithState<'s, O, E, P> {
        WithState {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }
}
