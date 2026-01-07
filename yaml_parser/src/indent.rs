use super::Input;
use std::marker::PhantomData;
use winnow::{
    ModalParser, ModalResult, Parser,
    combinator::impls::Context,
    error::{AddContext, ErrMode, ParserError, StrContext},
};

pub(super) struct TrackIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, ErrMode<E>> for TrackIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> ModalResult<O, E> {
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
    P: ModalParser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, ErrMode<E>> for VerifyIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> ModalResult<O, E> {
        let indent = input.state.indent;
        let output = self.parser.parse_next(input)?;
        if input.state.indent == indent || input.is_empty() {
            Ok(output)
        } else if input.state.tracked_indents & (1 << input.state.indent) == 0 {
            Err(ErrMode::Cut(E::from_input(input)))
        } else {
            input.state.tracked_indents -= 1 << indent;
            Err(ErrMode::Backtrack(E::from_input(input)))
        }
    }
}

pub(super) struct StorePrevIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, ErrMode<E>> for StorePrevIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> ModalResult<O, E> {
        let prev_indent = input.state.prev_indent;
        input.state.prev_indent = Some(input.state.indent);
        let result = self.parser.parse_next(input);
        if result.is_err() {
            input.state.prev_indent = prev_indent;
        }
        result
    }
}

pub(super) struct RequireDeeperIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    parser: P,
    s: PhantomData<&'s ()>,
    o: PhantomData<O>,
    e: PhantomData<E>,
}

impl<'s, O, E, P> Parser<Input<'s>, O, ErrMode<E>> for RequireDeeperIndent<'s, O, E, P>
where
    E: ParserError<Input<'s>>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn parse_next(&mut self, input: &mut Input<'s>) -> ModalResult<O, E> {
        if !input.state.document_top
            && input.state.last_ws_has_nl
            && input
                .state
                .prev_indent
                .is_some_and(|prev_indent| prev_indent >= input.state.indent)
        {
            Err(ErrMode::Backtrack(E::from_input(input)))
        } else {
            self.parser.parse_next(input)
        }
    }
}

pub(super) trait ParserExt<'s, O, E, P>
where
    E: ParserError<Input<'s>> + AddContext<Input<'s>, StrContext>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn track_indent(self) -> TrackIndent<'s, O, E, P>;
    fn verify_indent(
        self,
    ) -> Context<VerifyIndent<'s, O, E, P>, Input<'s>, O, ErrMode<E>, StrContext>;
    fn store_prev_indent(self) -> StorePrevIndent<'s, O, E, P>;
    fn require_deeper_indent(self) -> RequireDeeperIndent<'s, O, E, P>;
}

impl<'s, O, E, P> ParserExt<'s, O, E, P> for P
where
    E: ParserError<Input<'s>> + AddContext<Input<'s>, StrContext>,
    P: ModalParser<Input<'s>, O, E>,
{
    fn track_indent(self) -> TrackIndent<'s, O, E, P> {
        TrackIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }

    fn verify_indent(
        self,
    ) -> Context<VerifyIndent<'s, O, E, P>, Input<'s>, O, ErrMode<E>, StrContext> {
        VerifyIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
        .context(StrContext::Label("indentation"))
    }

    fn store_prev_indent(self) -> StorePrevIndent<'s, O, E, P> {
        StorePrevIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }

    fn require_deeper_indent(self) -> RequireDeeperIndent<'s, O, E, P> {
        RequireDeeperIndent {
            parser: self,
            s: PhantomData,
            o: PhantomData,
            e: PhantomData,
        }
    }
}
