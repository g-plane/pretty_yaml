use super::{Input, State};
use std::marker::PhantomData;
use winnow::Parser;

pub(super) struct SetState<'s, O, E, F, P>
where
    F: FnMut(&mut State),
    P: Parser<Input<'s>, O, E>,
{
    parser: P,
    f: F,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, F, P> Parser<Input<'s>, O, E> for SetState<'s, O, E, F, P>
where
    F: FnMut(&mut State),
    P: Parser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> Result<O, E> {
        let original_state = input.state.clone();
        (self.f)(&mut input.state);
        let result = self.parser.parse_next(input);
        input.state = original_state;
        result
    }
}

pub(super) trait ParserExt<'s, O, E, F, P>
where
    F: FnMut(&mut State),
    P: Parser<Input<'s>, O, E>,
{
    fn set_state(self, f: F) -> SetState<'s, O, E, F, P>;
}

impl<'s, O, E, F, P> ParserExt<'s, O, E, F, P> for P
where
    F: FnMut(&mut State),
    P: Parser<Input<'s>, O, E>,
{
    fn set_state(self, f: F) -> SetState<'s, O, E, F, P> {
        SetState {
            parser: self,
            f,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }
}
