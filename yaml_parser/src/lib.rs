pub use self::error::SyntaxError;
use self::{indent::ParserExt as _, set_state::ParserExt as _, verify_state::verify_state};
use either::Either;
use rowan::{GreenNode, GreenToken, NodeOrToken};
use winnow::{
    ascii::{digit1, line_ending, multispace1, space1, take_escaped, till_line_ending},
    combinator::{
        alt, cond, cut_err, dispatch, eof, fail, not, opt, peek, preceded, repeat, repeat_till,
        terminated, trace,
    },
    error::{StrContext, StrContextValue},
    stream::Stateful,
    token::{any, none_of, one_of, take_till, take_while},
    PResult, Parser,
};

mod error;
mod indent;
mod set_state;
mod verify_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
/// Syntax kind enum for nodes and tokens.
pub enum SyntaxKind {
    // SyntaxToken
    L_BRACE = 0,
    R_BRACE,
    L_BRACKET,
    R_BRACKET,
    AMPERSAND,
    ASTERISK,
    COLON,
    COMMA,
    EXCLAMATION_MARK,
    PLUS,
    MINUS,
    QUESTION_MARK,
    BAR,
    PERCENT,
    INDENT_INDICATOR,
    GREATER_THAN,
    VERBATIM_TAG,
    SHORTHAND_TAG,
    TAG_CHAR,
    TAG_HANDLE_NAMED,
    TAG_HANDLE_SECONDARY,
    TAG_HANDLE_PRIMARY,
    TAG_PREFIX,
    ANCHOR_NAME,
    DOUBLE_QUOTED_SCALAR,
    SINGLE_QUOTED_SCALAR,
    PLAIN_SCALAR,
    BLOCK_SCALAR_TEXT,
    DIRECTIVES_END,
    DIRECTIVE_NAME,
    YAML_VERSION,
    DIRECTIVE_PARAM,
    DOCUMENT_END,

    // SyntaxNode
    PROPERTIES,
    TAG_PROPERTY,
    TAG_HANDLE,
    NON_SPECIFIC_TAG,
    ANCHOR_PROPERTY,
    ALIAS,
    FLOW_SEQ,
    FLOW_SEQ_ENTRIES,
    FLOW_SEQ_ENTRY,
    FLOW_MAP,
    FLOW_MAP_ENTRIES,
    FLOW_MAP_ENTRY,
    FLOW_MAP_KEY,
    FLOW_MAP_VALUE,
    FLOW_PAIR,
    FLOW,
    CHOMPING_INDICATOR,
    BLOCK_SCALAR,
    BLOCK_SEQ,
    BLOCK_SEQ_ENTRY,
    BLOCK_MAP,
    BLOCK_MAP_ENTRY,
    BLOCK_MAP_KEY,
    BLOCK_MAP_VALUE,
    BLOCK,
    YAML_DIRECTIVE,
    TAG_DIRECTIVE,
    RESERVED_DIRECTIVE,
    DIRECTIVE,
    DOCUMENT,

    COMMENT,
    WHITESPACE,
    ROOT,
}
use SyntaxKind::*;

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YamlLanguage {}
impl rowan::Language for YamlLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<YamlLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<YamlLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<YamlLanguage>;

type GreenElement = NodeOrToken<GreenNode, GreenToken>;
type GreenResult = PResult<GreenElement>;
type Input<'s> = Stateful<&'s str, State>;

fn tok(kind: SyntaxKind, text: &str) -> GreenElement {
    NodeOrToken::Token(GreenToken::new(kind.into(), text))
}
fn node<I>(kind: SyntaxKind, children: I) -> GreenElement
where
    I: IntoIterator<Item = GreenElement>,
    I::IntoIter: ExactSizeIterator,
{
    NodeOrToken::Node(GreenNode::new(kind.into(), children))
}
fn ascii_char<const C: char>(kind: SyntaxKind) -> impl FnMut(&mut Input) -> GreenResult {
    debug_assert!(C.is_ascii());
    move |input| {
        C.map(|_| {
            let mut buffer = [0; 1];
            NodeOrToken::Token(GreenToken::new(kind.into(), C.encode_utf8(&mut buffer)))
        })
        .parse_next(input)
    }
}

fn tag_property(input: &mut Input) -> GreenResult {
    alt((verbatim_tag, shorthand_tag, non_specific_tag))
        .context(StrContext::Label("tag property"))
        .parse_next(input)
        .map(|child| node(TAG_PROPERTY, [child]))
}

fn verbatim_tag(input: &mut Input) -> GreenResult {
    ("!<", cut_err((take_while(1.., is_url_char), '>')))
        .recognize()
        .context(StrContext::Label("verbatim tag"))
        .parse_next(input)
        .map(|text| tok(VERBATIM_TAG, text))
}

fn shorthand_tag(input: &mut Input) -> GreenResult {
    (
        tag_handle,
        take_while(1.., is_tag_char).map(|text| tok(TAG_CHAR, text)),
    )
        .parse_next(input)
        .map(|(tag_handle, tag_char)| node(SHORTHAND_TAG, [tag_handle, tag_char]))
}

fn tag_handle(input: &mut Input) -> GreenResult {
    alt((
        ('!', take_while(1.., is_word_char), '!')
            .recognize()
            .map(|text| tok(TAG_HANDLE_NAMED, text)),
        "!!".map(|text| tok(TAG_HANDLE_SECONDARY, text)),
        "!".map(|text| tok(TAG_HANDLE_PRIMARY, text)),
    ))
    .parse_next(input)
    .map(|child| node(TAG_HANDLE, [child]))
}

fn non_specific_tag(input: &mut Input) -> GreenResult {
    ascii_char::<'!'>(EXCLAMATION_MARK)
        .parse_next(input)
        .map(|child| node(NON_SPECIFIC_TAG, [child]))
}

fn anchor_property(input: &mut Input) -> GreenResult {
    (ascii_char::<'&'>(AMPERSAND), cut_err(anchor_name))
        .context(StrContext::Label("anchor property"))
        .parse_next(input)
        .map(|(ampersand, name)| {
            NodeOrToken::Node(GreenNode::new(ANCHOR_PROPERTY.into(), [ampersand, name]))
        })
}

fn properties(input: &mut Input) -> GreenResult {
    trace(
        "properties",
        dispatch! {peek(any);
            '&' => (
                anchor_property,
                opt(terminated((stateless_separate, tag_property), peek(not((space1, one_of(['&', '!'])))))),
            ),
            '!' => (
                cut_err(tag_property),
                opt(terminated((stateless_separate, anchor_property), peek(not((space1, one_of(['&', '!'])))))),
            ),
            _ => fail,
        },
    )
    .parse_next(input)
    .map(|(first, second)| {
        let mut children = vec![first];
        if let Some((mut trivias, second)) = second {
            children.append(&mut trivias);
            children.push(second);
        }
        node(PROPERTIES, children)
    })
}

fn alias(input: &mut Input) -> GreenResult {
    (ascii_char::<'*'>(ASTERISK), cut_err(anchor_name))
        .context(StrContext::Label("alias"))
        .parse_next(input)
        .map(|(asterisk, name)| NodeOrToken::Node(GreenNode::new(ALIAS.into(), [asterisk, name])))
}

fn anchor_name(input: &mut Input) -> GreenResult {
    take_till(1.., |c| is_flow_indicator(c) || c.is_ascii_whitespace())
        .parse_next(input)
        .map(|text| tok(ANCHOR_NAME, text))
}

fn double_qouted_scalar(input: &mut Input) -> GreenResult {
    trace(
        "double_qouted_scalar",
        (
            '"',
            cut_err((take_escaped(none_of(['\\', '"']), '\\', any), '"')),
        )
            .recognize()
            .context(StrContext::Expected(StrContextValue::CharLiteral('"'))),
    )
    .parse_next(input)
    .map(|text| tok(DOUBLE_QUOTED_SCALAR, text))
}

fn single_qouted_scalar(input: &mut Input) -> GreenResult {
    trace(
        "single_qouted_scalar",
        (
            '\'',
            cut_err((
                repeat::<_, _, (), _, _>(0.., alt((none_of('\'').void(), "''".void()))),
                '\'',
            )),
        )
            .recognize()
            .context(StrContext::Expected(StrContextValue::CharLiteral('\''))),
    )
    .parse_next(input)
    .map(|text| tok(SINGLE_QUOTED_SCALAR, text))
}

fn plain_scalar(input: &mut Input) -> GreenResult {
    let indent = input.state.indent;
    let last_ws_has_nl = input.state.last_ws_has_nl;
    let document_top = input.state.document_top;
    if matches!(
        input.state.bf_ctx,
        BlockFlowCtx::FlowIn | BlockFlowCtx::FlowOut
    ) {
        let safe_in = matches!(input.state.bf_ctx, BlockFlowCtx::FlowIn);
        trace(
            "plain_scalar",
            (
                plain_scalar_one_line,
                repeat::<_, _, (), _, _>(
                    0..,
                    (
                        (
                            multispace1,
                            peek(opt(alt((
                                one_of(move |c: char| {
                                    matches!(c, '\n' | '\r' | '#')
                                        || safe_in && is_flow_indicator(c)
                                })
                                .recognize(),
                                (
                                    ':',
                                    one_of(move |c: char| {
                                        c.is_ascii_whitespace() || safe_in && is_flow_indicator(c)
                                    }),
                                )
                                    .recognize(),
                                terminated(alt(("---", "...")), multispace1),
                                eof,
                            )))),
                        )
                            .verify_map(
                                move |(text, peeked): (&str, _)| {
                                    match peeked {
                                        Some("---" | "...") => !text.ends_with(['\n', '\r']),
                                        Some(..) => false,
                                        None => {
                                            if let Some(detected) = detect_ws_indent(text) {
                                                if last_ws_has_nl {
                                                    detected >= indent
                                                } else {
                                                    detected > indent || document_top
                                                }
                                            } else {
                                                true
                                            }
                                        }
                                    }
                                    .then_some(text)
                                },
                            ),
                        plain_scalar_chars,
                    ),
                ),
            )
                .recognize(),
        )
        .parse_next(input)
        .map(|text| tok(PLAIN_SCALAR, text))
    } else {
        trace("plain_scalar", plain_scalar_one_line.recognize())
            .parse_next(input)
            .map(|text| tok(PLAIN_SCALAR, text))
    }
}
fn plain_scalar_one_line(input: &mut Input) -> PResult<()> {
    (
        alt((
            none_of(|c: char| c.is_ascii_whitespace() || is_indicator(c)),
            terminated(
                one_of(['-', ':', '?']),
                peek(none_of(|c: char| {
                    c.is_ascii_whitespace() || is_flow_indicator(c)
                })),
            ),
        )),
        plain_scalar_chars,
    )
        .void()
        .parse_next(input)
}
fn plain_scalar_chars(input: &mut Input) -> PResult<()> {
    let safe_in = matches!(
        input.state.bf_ctx,
        BlockFlowCtx::FlowIn | BlockFlowCtx::FlowKey
    );
    repeat(
        0..,
        alt((
            take_till(1.., move |c: char| {
                c.is_ascii_whitespace() || c == ':' || safe_in && is_flow_indicator(c)
            })
            .void(),
            terminated(
                ':'.void(),
                peek(none_of(move |c: char| {
                    c.is_ascii_whitespace() || safe_in && is_flow_indicator(c)
                })),
            ),
            terminated(
                space1.void(),
                peek(not(alt((
                    one_of(move |c| {
                        matches!(c, '\n' | '\r' | '#') || safe_in && is_flow_indicator(c)
                    })
                    .void(),
                    (
                        ':',
                        one_of(move |c: char| {
                            c.is_ascii_whitespace() || safe_in && is_flow_indicator(c)
                        }),
                    )
                        .void(),
                    eof.void(),
                )))),
            ),
        )),
    )
    .parse_next(input)
}

fn flow_sequence(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'['>(L_BRACKET),
        stateless_cmts_or_ws0,
        flow_sequence_entries.set_state(flow_collection_state),
        ascii_char::<']'>(R_BRACKET),
    )
        .context(StrContext::Expected(StrContextValue::CharLiteral(']')))
        .parse_next(input)
        .map(|(l_bracket, mut leading_trivias, entries, r_bracket)| {
            let mut children = Vec::with_capacity(3);
            children.push(l_bracket);
            children.append(&mut leading_trivias);
            children.push(entries);
            children.push(r_bracket);
            node(FLOW_SEQ, children)
        })
}

fn flow_sequence_entries(input: &mut Input) -> GreenResult {
    repeat(
        0..,
        alt((
            (
                flow_sequence_entry,
                stateless_cmts_or_ws0,
                alt((ascii_char::<','>(COMMA).map(Some), peek(']').value(None))),
            )
                .map(Either::Left),
            stateless_cmts_or_ws1.map(Either::Right),
        )),
    )
    .fold(Vec::new, |mut children, either| {
        match either {
            Either::Left((entry, mut trailing_trivias, comma)) => {
                children.reserve(3);
                children.push(entry);
                children.append(&mut trailing_trivias);
                if let Some(comma) = comma {
                    children.push(comma);
                }
            }
            Either::Right(mut trivias) => children.append(&mut trivias),
        }
        children
    })
    .parse_next(input)
    .map(|children| node(FLOW_SEQ_ENTRIES, children))
}

fn flow_sequence_entry(input: &mut Input) -> GreenResult {
    alt((
        terminated(flow, peek(not((stateless_cmts_or_ws0, ':')))),
        flow_pair,
    ))
    .parse_next(input)
    .map(|child| node(FLOW_SEQ_ENTRY, [child]))
}

fn flow_map(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'{'>(L_BRACE),
        stateless_cmts_or_ws0,
        flow_map_entries.set_state(flow_collection_state),
        ascii_char::<'}'>(R_BRACE),
    )
        .context(StrContext::Expected(StrContextValue::CharLiteral('}')))
        .parse_next(input)
        .map(|(l_brace, mut leading_trivias, entries, r_brace)| {
            let mut children = Vec::with_capacity(3);
            children.push(l_brace);
            children.append(&mut leading_trivias);
            children.push(entries);
            children.push(r_brace);
            node(FLOW_MAP, children)
        })
}

fn flow_map_entries(input: &mut Input) -> GreenResult {
    repeat(
        0..,
        alt((
            (
                flow_map_entry,
                stateless_cmts_or_ws0,
                alt((ascii_char::<','>(COMMA).map(Some), peek('}').value(None))),
            )
                .map(Either::Left),
            stateless_cmts_or_ws1.map(Either::Right),
        )),
    )
    .fold(Vec::new, |mut children, either| {
        match either {
            Either::Left((entry, mut trailing_trivias, comma)) => {
                children.reserve(3);
                children.push(entry);
                children.append(&mut trailing_trivias);
                if let Some(comma) = comma {
                    children.push(comma);
                }
            }
            Either::Right(mut trivias) => children.append(&mut trivias),
        }
        children
    })
    .parse_next(input)
    .map(|children| node(FLOW_MAP_ENTRIES, children))
}

fn flow_map_entry(input: &mut Input) -> GreenResult {
    alt((
        (
            opt((flow_map_entry_key, stateless_cmts_or_ws0)),
            ascii_char::<':'>(COLON),
            opt((stateless_cmts_or_ws0, flow)),
        )
            .map(|(key, colon, value)| {
                let mut children = Vec::with_capacity(3);
                if let Some((key, mut trivias_before_colon)) = key {
                    children.push(key);
                    children.append(&mut trivias_before_colon);
                }
                children.push(colon);
                if let Some((mut trivias_after_colon, value)) = value {
                    children.append(&mut trivias_after_colon);
                    children.push(node(FLOW_MAP_VALUE, [value]));
                }
                node(FLOW_MAP_ENTRY, children)
            }),
        flow_map_entry_key.map(|child| node(FLOW_MAP_ENTRY, [child])),
    ))
    .parse_next(input)
}

fn flow_pair(input: &mut Input) -> GreenResult {
    trace(
        "flow_pair",
        (
            opt(dispatch! {peek((any, any));
                ('?', ' ' | '\t' | '\n' | '\r') => flow_map_entry_key,
                _ => flow_map_entry_key.set_state(|state| state.bf_ctx = BlockFlowCtx::FlowKey),
            }),
            stateless_cmts_or_ws0,
            ascii_char::<':'>(COLON),
            opt((stateless_cmts_or_ws0, flow)),
        ),
    )
    .parse_next(input)
    .map(|(key, mut trivias_before_colon, colon, value)| {
        let mut children = Vec::with_capacity(3);
        if let Some(key) = key {
            children.push(key);
        }
        children.append(&mut trivias_before_colon);
        children.push(colon);
        if let Some((mut trivias_after_colon, value)) = value {
            children.append(&mut trivias_after_colon);
            children.push(node(FLOW_MAP_VALUE, [value]));
        }
        node(FLOW_PAIR, children)
    })
}

fn flow_map_entry_key(input: &mut Input) -> GreenResult {
    alt((
        flow.map(|child| node(FLOW_MAP_KEY, [child])),
        (
            ascii_char::<'?'>(QUESTION_MARK),
            opt((stateless_cmts_or_ws1, flow)),
        )
            .map(|(question_mark, key)| {
                let mut children = Vec::with_capacity(3);
                children.push(question_mark);
                if let Some((mut trivias, key)) = key {
                    children.append(&mut trivias);
                    children.push(key);
                }
                node(FLOW_MAP_KEY, children)
            }),
    ))
    .parse_next(input)
}

fn flow_content(input: &mut Input) -> GreenResult {
    trace(
        "flow_content",
        dispatch! {peek(any);
            '"' => double_qouted_scalar,
            '\'' => single_qouted_scalar,
            '[' => flow_sequence,
            '{' => flow_map,
            _ => plain_scalar,
        },
    )
    .parse_next(input)
}

fn flow(input: &mut Input) -> GreenResult {
    trace("flow", dispatch! {peek(any);
        '*' => alias.map(|child| node(FLOW, [child])),
        '&' | '!' => (properties, opt((stateless_separate, flow_content))).map(|(properties, content)| {
            let mut children = Vec::with_capacity(3);
            children.push(properties);
            if let Some((mut trivias, content)) = content {
                children.append(&mut trivias);
                children.push(content);
            }
            node(FLOW, children)
        }),
        _ => flow_content.map(|child| node(FLOW, [child])),
    })
    .parse_next(input)
}

fn block_scalar(input: &mut Input) -> GreenResult {
    let base_indent = input.state.prev_indent.unwrap_or(input.state.indent);
    let document_top = input.state.document_top;
    (
        (alt((ascii_char::<'|'>(BAR), ascii_char::<'>'>(GREATER_THAN)))),
        opt(alt((
            (indent_indicator, opt(chomping_indicator)).map(Either::Left),
            (chomping_indicator, opt(indent_indicator)).map(Either::Right),
        )))
        .context(StrContext::Label("block scalar header")),
        opt(space),
        opt(comment),
        peek(opt(linebreaks_or_spaces.verify_map(detect_ws_indent))),
    )
        .flat_map(|(style, indicator, space, comment, mut indent)| {
            let mut children = Vec::with_capacity(3);
            children.push(style);
            match indicator {
                Some(Either::Left(((indent_token, indent_value), chomping_token))) => {
                    children.push(indent_token);
                    indent = Some(base_indent + indent_value);
                    if let Some(chomping) = chomping_token {
                        children.push(chomping);
                    }
                }
                Some(Either::Right((chomping_token, indent_indicator))) => {
                    children.push(chomping_token);
                    if let Some((indent_token, indent_value)) = indent_indicator {
                        children.push(indent_token);
                        indent = Some(base_indent + indent_value);
                    }
                }
                None => {}
            }
            if let Some(space) = space {
                children.push(space);
            }
            if let Some(comment) = comment {
                children.push(comment);
            }
            let indent = indent.unwrap_or_default();
            cond(
                indent > base_indent || document_top,
                repeat::<_, _, (), _, _>(
                    0..,
                    (
                        linebreaks_or_spaces.verify(move |text: &str| {
                            detect_ws_indent(text).is_some_and(|detected| detected >= indent)
                        }),
                        till_line_ending,
                    )
                        .verify(|(ws, line): &(&str, _)| {
                            !line.is_empty()
                                && !(ws.ends_with(['\n', '\r'])
                                    && (*line == "..." || *line == "---"))
                        }),
                )
                .recognize(),
            )
            .map(move |text| {
                let mut children = children.clone();
                if let Some(text) = text {
                    children.push(tok(BLOCK_SCALAR_TEXT, text));
                }
                node(BLOCK_SCALAR, children)
            })
        })
        .parse_next(input)
}
fn indent_indicator(input: &mut Input) -> PResult<(GreenElement, usize)> {
    one_of(|c: char| c.is_ascii_digit())
        .recognize()
        .try_map(|text: &str| {
            text.parse()
                .map(|value| (tok(INDENT_INDICATOR, text), value))
        })
        .parse_next(input)
}
fn chomping_indicator(input: &mut Input) -> GreenResult {
    dispatch! {peek(any);
        '+' => ascii_char::<'+'>(PLUS),
        '-' => ascii_char::<'-'>(MINUS),
        ' ' | '\n' | '\t' | '\r' => fail,
        _ => cut_err(fail),
    }
    .parse_next(input)
    .map(|child| node(CHOMPING_INDICATOR, [child]))
}

fn block_sequence(input: &mut Input) -> GreenResult {
    trace(
        "block_sequence",
        (
            block_sequence_entry,
            repeat(0.., (cmts_or_ws1.verify_indent(), block_sequence_entry)),
        ),
    )
    .parse_next(input)
    .map(|(first, rest): (_, Vec<_>)| {
        let mut children = Vec::with_capacity(1 + rest.len());
        children.push(first);
        for (mut trivias, entry) in rest {
            children.append(&mut trivias);
            children.push(entry);
        }
        node(BLOCK_SEQ, children)
    })
}

fn block_sequence_entry(input: &mut Input) -> GreenResult {
    trace(
        "block_sequence_entry",
        (
            ascii_char::<'-'>(MINUS)
                .context(StrContext::Expected(StrContextValue::CharLiteral('-'))),
            alt((
                block_compact_collection,
                (cmts_or_ws1.store_prev_indent().track_indent(), block).map(Some),
                peek((opt(space1), opt(comment), alt((line_ending, eof)))).value(None),
            ))
            .set_state(|state| {
                state.bf_ctx = BlockFlowCtx::BlockIn;
                state.document_top = false;
            }),
        ),
    )
    .parse_next(input)
    .map(|(minus, value)| {
        if let Some((mut ws, value)) = value {
            let mut children = Vec::with_capacity(3);
            children.push(minus);
            children.append(&mut ws);
            children.push(value);
            node(BLOCK_SEQ_ENTRY, children)
        } else {
            node(BLOCK_SEQ_ENTRY, [minus])
        }
    })
}

fn block_compact_collection(
    input: &mut Input,
) -> PResult<Option<(Vec<GreenElement>, GreenElement)>> {
    let original_state = input.state.clone();
    let result = (
        space_before_block_compact_collection.track_indent(),
        alt((block_sequence, block_map)),
    )
        .map(|(space, collection)| Some((vec![space], node(BLOCK, [collection]))))
        .parse_next(input);
    input.state = original_state;
    result
}
fn space_before_block_compact_collection(input: &mut Input) -> GreenResult {
    let (space, text) = space.with_recognized().parse_next(input)?;
    input.state.prev_indent = Some(input.state.indent);
    input.state.indent += text.len() + 1;
    Ok(space)
}

fn block_map(input: &mut Input) -> GreenResult {
    let indent = input.state.indent;
    trace(
        "block_map",
        (
            alt((block_map_implicit_entry, block_map_explicit_entry)),
            repeat(
                0..,
                (
                    terminated(
                        cmts_or_ws1,
                        verify_state(move |state| state.indent == indent),
                    ),
                    alt((block_map_implicit_entry, block_map_explicit_entry)),
                ),
            ),
        ),
    )
    .parse_next(input)
    .map(|(first, rest): (_, Vec<_>)| {
        let mut children = Vec::with_capacity(1 + rest.len());
        children.push(first);
        for (mut trivias, entry) in rest {
            children.append(&mut trivias);
            children.push(entry);
        }
        node(BLOCK_MAP, children)
    })
}

fn block_map_explicit_entry(input: &mut Input) -> GreenResult {
    trace(
        "block_map_explicit_entry",
        (
            trace(
                "block_map_explicit_key",
                block_map_explicit_key.store_prev_indent(),
            ),
            opt((
                cmts_or_ws1,
                ascii_char::<':'>(COLON),
                alt((
                    block_compact_collection,
                    opt((cmts_or_ws1.track_indent(), block)),
                )),
            ))
            .set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut),
        )
            .set_state(|state| state.document_top = false),
    )
    .parse_next(input)
    .map(|(key, value)| {
        if let Some((mut trivias_before_colon, colon, value)) = value {
            let mut children = Vec::with_capacity(3);
            children.push(key);
            children.append(&mut trivias_before_colon);
            children.push(colon);
            if let Some((mut trivias_after_colon, value)) = value {
                children.append(&mut trivias_after_colon);
                children.push(node(BLOCK_MAP_VALUE, [value]));
            }
            node(BLOCK_MAP_ENTRY, children)
        } else {
            node(BLOCK_MAP_ENTRY, [key])
        }
    })
}

fn block_map_explicit_key(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'?'>(QUESTION_MARK),
        alt((
            block_compact_collection,
            (
                cmts_or_ws1.track_indent(),
                block.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut),
            )
                .map(Some),
            line_ending.value(None),
        )),
    )
        .parse_next(input)
        .map(|(question_mark, key)| {
            if let Some((mut trivias, key)) = key {
                let mut children = Vec::with_capacity(3);
                children.push(question_mark);
                children.append(&mut trivias);
                children.push(key);
                node(BLOCK_MAP_KEY, children)
            } else {
                node(BLOCK_MAP_KEY, [question_mark])
            }
        })
}

fn block_map_implicit_entry(input: &mut Input) -> GreenResult {
    trace(
        "block_map_implicit_entry",
        (
            opt((block_map_implicit_key.store_prev_indent(), opt(space))),
            ascii_char::<':'>(COLON),
            opt((
                cmts_or_ws1.track_indent(),
                block.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut),
            )),
        )
            .set_state(|state| state.document_top = false),
    )
    .parse_next(input)
    .map(|(key, colon, value)| {
        let mut children = Vec::with_capacity(4);
        if let Some((key, space)) = key {
            children.push(key);
            if let Some(space) = space {
                children.push(space);
            }
        }
        children.push(colon);
        if let Some((mut trivias, value)) = value {
            children.append(&mut trivias);
            children.push(node(BLOCK_MAP_VALUE, [value]));
        }
        node(BLOCK_MAP_ENTRY, children)
    })
}

fn block_map_implicit_key(input: &mut Input) -> GreenResult {
    trace(
        "block_map_implicit_key",
        flow.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockKey),
    )
    .parse_next(input)
    .map(|child| node(BLOCK_MAP_KEY, [child]))
}

fn block(input: &mut Input) -> GreenResult {
    let mut bf_ctx = |input: &mut Input| -> PResult<_> { Ok(input.state.bf_ctx.clone()) };

    trace(
        "block",
        alt((
            (
                opt((
                    properties,
                    terminated(
                        cmts_or_ws1.track_indent(),
                        alt((
                            verify_state(|state| state.last_ws_has_nl),
                            peek(one_of(['|', '>'])).void(),
                        )),
                    ),
                )),
                alt((
                    dispatch! {bf_ctx;
                        BlockFlowCtx::BlockIn => block_sequence.require_deeper_indent(),
                        _ => preceded(
                            verify_state(|state| state.prev_indent.is_some_and(|prev_indent| state.indent >= prev_indent)),
                            block_sequence,
                        ),
                    },
                    dispatch! {bf_ctx;
                        BlockFlowCtx::BlockOut => block_map.require_deeper_indent(),
                        _ => block_map,
                    },
                    trace("block_scalar", block_scalar),
                )),
            )
                .map(|(properties, block)| {
                    let mut children = Vec::with_capacity(3);
                    if let Some((properties, mut trivias)) = properties {
                        children.push(properties);
                        children.append(&mut trivias);
                    }
                    children.push(block);
                    node(BLOCK, children)
                }),
            flow.require_deeper_indent()
                .set_state(|state| state.bf_ctx = BlockFlowCtx::FlowOut),
            properties.map(|child| node(BLOCK, [child])),
        )),
    )
    .parse_next(input)
}

fn directives_end(input: &mut Input) -> GreenResult {
    terminated("---", peek(multispace1))
        .map(|text| tok(DIRECTIVES_END, text))
        .parse_next(input)
}

fn yaml_directive(input: &mut Input) -> GreenResult {
    ("YAML", space, (digit1, '.', digit1).recognize())
        .parse_next(input)
        .map(|(name, space, version)| {
            node(
                YAML_DIRECTIVE,
                [tok(DIRECTIVE_NAME, name), space, tok(YAML_VERSION, version)],
            )
        })
}

fn tag_directive(input: &mut Input) -> GreenResult {
    ("TAG", space, tag_handle, space, tag_prefix)
        .parse_next(input)
        .map(|(name, space1, tag_handle, space2, tag_prefix)| {
            node(
                TAG_DIRECTIVE,
                [
                    tok(DIRECTIVE_NAME, name),
                    space1,
                    tag_handle,
                    space2,
                    tag_prefix,
                ],
            )
        })
}
fn tag_prefix(input: &mut Input) -> GreenResult {
    (
        one_of(|c| c == '!' || is_tag_char(c)),
        take_while(0.., is_url_char),
    )
        .recognize()
        .parse_next(input)
        .map(|text| tok(TAG_PREFIX, text))
}

fn reserved_directive(input: &mut Input) -> GreenResult {
    (
        take_till(1.., |c: char| c.is_ascii_whitespace()),
        space,
        repeat::<_, _, (), _, _>(
            0..,
            alt((
                take_till(1.., |c: char| c.is_ascii_whitespace()),
                terminated(space1, peek(none_of('#'))),
            )),
        )
        .recognize(),
    )
        .parse_next(input)
        .map(|(name, space, param)| {
            node(
                RESERVED_DIRECTIVE,
                [
                    tok(DIRECTIVE_NAME, name),
                    space,
                    tok(DIRECTIVE_PARAM, param),
                ],
            )
        })
}

fn directive(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'%'>(PERCENT),
        cut_err(alt((yaml_directive, tag_directive, reserved_directive))),
    )
        .context(StrContext::Label("directive"))
        .parse_next(input)
        .map(|(percent, directive)| node(DIRECTIVE, [percent, directive]))
}

fn document(input: &mut Input) -> GreenResult {
    let prev_document_finished = input.state.prev_document_finished;
    alt((
        (
            repeat(1.., (directive, cmts_or_ws0)),
            directives_end,
            opt((cmts_or_ws0, top_level_block.store_prev_indent())),
            opt((cmts_or_ws1, document_end)),
        )
            .map(
                |(directives, directives_end, block, document_end): (Vec<_>, _, _, _)| {
                    let mut children = Vec::with_capacity(3 + directives.len());
                    directives.into_iter().for_each(|(directive, mut trivias)| {
                        children.push(directive);
                        children.append(&mut trivias);
                    });
                    children.push(directives_end);
                    if let Some((mut trivias, block)) = block {
                        children.append(&mut trivias);
                        children.push(block);
                    }
                    if let Some((mut trivias, document_end)) = document_end {
                        children.append(&mut trivias);
                        children.push(document_end);
                    }
                    node(DOCUMENT, children)
                },
            ),
        document_end.map(|child| node(DOCUMENT, [child])),
        (
            cut_err(
                opt((directives_end, cmts_or_ws0))
                    .verify(move |end| end.is_some() || prev_document_finished)
                    .context(StrContext::Expected(StrContextValue::StringLiteral("..."))),
            ),
            top_level_block.store_prev_indent(),
            opt((cmts_or_ws1, document_end)),
        )
            .map(|(directives_end, block, document_end)| {
                let mut children = Vec::with_capacity(1);
                if let Some((end, mut trivias)) = directives_end {
                    children.push(end);
                    children.append(&mut trivias);
                }
                children.push(block);
                if let Some((mut trivias, document_end)) = document_end {
                    children.append(&mut trivias);
                    children.push(document_end);
                }
                node(DOCUMENT, children)
            }),
        (directives_end, opt((cmts_or_ws1, document_end))).map(|(directives_end, document_end)| {
            let mut children = vec![directives_end];
            if let Some((mut trivias, document_end)) = document_end {
                children.append(&mut trivias);
                children.push(document_end);
            }
            node(DOCUMENT, children)
        }),
    ))
    .parse_next(input)
}
fn top_level_block(input: &mut Input) -> GreenResult {
    let result = preceded(
        not("..."),
        block.set_state(|state| {
            state.bf_ctx = BlockFlowCtx::BlockIn;
            state.document_top = true;
        }),
    )
    .parse_next(input);
    if result.is_ok() {
        input.state.prev_document_finished = false;
    }
    result
}

fn document_end(input: &mut Input) -> GreenResult {
    match "...".parse_next(input) {
        Ok(text) => {
            input.state.prev_document_finished = true;
            Ok(tok(DOCUMENT_END, text))
        }
        Err(err) => Err(err),
    }
}

fn root(input: &mut Input) -> PResult<SyntaxNode> {
    // `eof` parser is required because winnow will still try to parse the input even if it's empty,
    // but the validation of `directives_end` will fail since there's no input.
    repeat_till(0.., alt((cmt_or_ws, document)), eof)
        .parse_next(input)
        .map(|(children, _): (Vec<_>, _)| {
            SyntaxNode::new_root(GreenNode::new(ROOT.into(), children))
        })
}

fn comment(input: &mut Input) -> GreenResult {
    ('#', till_line_ending)
        .recognize()
        .parse_next(input)
        .map(|text| tok(COMMENT, text))
}

fn space(input: &mut Input) -> GreenResult {
    let text = space1.parse_next(input)?;
    input.state.last_ws_has_nl = false;
    Ok(tok(WHITESPACE, text))
}
/// Without tabs.
fn linebreaks_or_spaces<'s>(input: &mut Input<'s>) -> PResult<&'s str> {
    take_while(1.., |c| c == ' ' || c == '\n' || c == '\r').parse_next(input)
}
fn ws(input: &mut Input) -> GreenResult {
    let text = multispace1.parse_next(input)?;
    if let Some(indent) = detect_ws_indent(text) {
        input.state.indent = indent;
        input.state.last_ws_has_nl = true;
    } else {
        input.state.last_ws_has_nl = false;
    }
    Ok(tok(WHITESPACE, text))
}

/// Parse single comment or whitespace.
fn cmt_or_ws(input: &mut Input) -> GreenResult {
    trace(
        "cmt_or_ws",
        dispatch! {peek(any);
            ' ' | '\n' | '\t' | '\r' => ws,
            '#' => comment,
            _ => fail,
        },
    )
    .parse_next(input)
}
/// Parse zero or more comments or whitespaces.
fn cmts_or_ws0(input: &mut Input) -> PResult<Vec<GreenElement>> {
    repeat(0.., cmt_or_ws).parse_next(input)
}
/// Parse one or more comments or whitespaces.
fn cmts_or_ws1(input: &mut Input) -> PResult<Vec<GreenElement>> {
    repeat(1.., cmt_or_ws).parse_next(input)
}
/// Parse one or more comments or whitespaces without updating state.
fn stateless_cmt_or_ws(input: &mut Input) -> GreenResult {
    trace(
        "stateless_cmt_or_ws",
        dispatch! {peek(any);
            ' ' | '\n' | '\t' | '\r' => multispace1.map(|text| tok(WHITESPACE, text)),
            '#' => comment,
            _ => fail,
        },
    )
    .parse_next(input)
}
/// Parse zero or more comments or whitespaces without updating state.
fn stateless_cmts_or_ws0(input: &mut Input) -> PResult<Vec<GreenElement>> {
    repeat(0.., stateless_cmt_or_ws).parse_next(input)
}
/// Parse one or more comments or whitespaces without updating state.
fn stateless_cmts_or_ws1(input: &mut Input) -> PResult<Vec<GreenElement>> {
    repeat(1.., stateless_cmt_or_ws).parse_next(input)
}
/// Parse "s-separate" rule of YAML spec without updating state.
fn stateless_separate(input: &mut Input) -> PResult<Vec<GreenElement>> {
    if matches!(
        input.state.bf_ctx,
        BlockFlowCtx::FlowKey | BlockFlowCtx::BlockKey
    ) {
        space1
            .parse_next(input)
            .map(|text| vec![tok(WHITESPACE, text)])
    } else {
        stateless_cmts_or_ws1.parse_next(input)
    }
}

/// Parse the given YAML code into CST.
pub fn parse(code: &str) -> Result<SyntaxNode, SyntaxError> {
    let code = code.trim_start_matches('\u{feff}');
    let base_indent = detect_base_indent(code).unwrap_or_default();
    let input = Stateful {
        input: code,
        state: State {
            prev_indent: None,
            indent: base_indent,
            tracked_indents: 1 << base_indent,
            last_ws_has_nl: false,
            bf_ctx: BlockFlowCtx::BlockIn,
            document_top: true,
            prev_document_finished: true,
        },
    };
    root.parse(input).map_err(SyntaxError::from)
}

const CHAR_LOOKUP: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 5, 1, 5, 4, 5, 5, 5, 4, 4, 5, 4, 7, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 4, 0, 4, 1, 5,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 7, 0, 7, 0, 4,
    0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 1, 3, 4, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
fn is_indicator(c: char) -> bool {
    c.is_ascii() && CHAR_LOOKUP[c as usize] & 1 != 0
}
fn is_flow_indicator(c: char) -> bool {
    c.is_ascii() && CHAR_LOOKUP[c as usize] & 2 != 0
}
fn is_url_char(c: char) -> bool {
    c.is_ascii() && CHAR_LOOKUP[c as usize] & 4 != 0
}
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}
fn is_tag_char(c: char) -> bool {
    is_url_char(c) && c != '!' && !is_flow_indicator(c)
}

fn detect_base_indent(code: &str) -> Option<usize> {
    code.find(|c: char| !c.is_ascii_whitespace())
        .map(|first_contentful| {
            let first_linebreak = code[..first_contentful].rfind('\n');
            if let Some(first_linbreak) = first_linebreak {
                (first_contentful - first_linbreak).saturating_sub(1)
            } else {
                first_contentful
            }
        })
}

fn detect_ws_indent(text: &str) -> Option<usize> {
    text.rfind(['\n', '\r']).map(|index| text.len() - index - 1)
}

#[derive(Clone, Debug)]
struct State {
    prev_indent: Option<usize>,
    indent: usize,
    // Does someone's YAML file has more than 63 columns of indentation?
    tracked_indents: u64,
    // Indicates if the last whitespace token has linebreaks.
    last_ws_has_nl: bool,
    bf_ctx: BlockFlowCtx,
    document_top: bool,
    prev_document_finished: bool,
}

#[derive(Clone, Debug)]
enum BlockFlowCtx {
    BlockIn,
    BlockOut,
    BlockKey,
    FlowIn,
    FlowOut,
    FlowKey,
}

// https://yaml.org/spec/1.2.2/#rule-in-flow
fn flow_collection_state(state: &mut State) {
    state.bf_ctx = match &state.bf_ctx {
        BlockFlowCtx::FlowOut => BlockFlowCtx::FlowIn,
        BlockFlowCtx::FlowIn => BlockFlowCtx::FlowIn,
        BlockFlowCtx::BlockKey => BlockFlowCtx::FlowKey,
        BlockFlowCtx::FlowKey => BlockFlowCtx::FlowKey,
        ctx => ctx.clone(),
    };
}
