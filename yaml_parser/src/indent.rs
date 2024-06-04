use super::Input;
use std::marker::PhantomData;
use winnow::{
    combinator::Context,
    error::{AddContext, ErrMode, ErrorKind, ParserError, StrContext},
    PResult, Parser,
};

pub(super) struct TrackIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: Parser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, E> for TrackIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: Parser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> PResult<O, E> {
        let result = self.parser.parse_next(input);
        if result.is_ok() {
            input.state.tracked_indents |= 1 << input.state.indent;
        }
        result
    }
}

pub(super) struct VerifyIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: Parser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, E> for VerifyIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: Parser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> PResult<O, E> {
        let indent = input.state.indent;
        let output = self.parser.parse_next(input)?;
        if input.state.indent == indent {
            Ok(output)
        } else if input.state.tracked_indents & (1 << input.state.indent) == 0 {
            Err(ErrMode::Cut(E::from_error_kind(input, ErrorKind::Verify)))
        } else {
            input.state.tracked_indents -= 1 << indent;
            Err(ErrMode::Backtrack(E::from_error_kind(
                input,
                ErrorKind::Verify,
            )))
        }
    }
}

pub(super) trait ParserExt<'s, O, E, P>
where
    E: ParserError<Input<'s>> + AddContext<Input<'s>, StrContext>,
    P: Parser<Input<'s>, O, E>,
{
    fn track_indent(self) -> TrackIndent<'s, O, E, P>;
    fn verify_indent(self) -> Context<VerifyIndent<'s, O, E, P>, Input<'s>, O, E, StrContext>;
}

impl<'s, O, E, P> ParserExt<'s, O, E, P> for P
where
    E: ParserError<Input<'s>> + AddContext<Input<'s>, StrContext>,
    P: Parser<Input<'s>, O, E>,
{
    fn track_indent(self) -> TrackIndent<'s, O, E, P> {
        TrackIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }

    fn verify_indent(self) -> Context<VerifyIndent<'s, O, E, P>, Input<'s>, O, E, StrContext> {
        VerifyIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
        .context(StrContext::Label("indentation"))
    }
}
