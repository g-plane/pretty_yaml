pub use self::error::SyntaxError;
use self::{indent::ParserExt as _, set_state::ParserExt as _};
use either::Either;
use rowan::{GreenNode, GreenToken, NodeOrToken};
use winnow::{
    ascii::{digit1, multispace1, space1, take_escaped, till_line_ending},
    combinator::{alt, cut_err, dispatch, fail, opt, peek, repeat, terminated},
    error::{StrContext, StrContextValue},
    stream::Stateful,
    token::{any, none_of, one_of, take_till, take_while},
    PResult, Parser,
};

mod error;
mod indent;
mod set_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u16)]
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

type GreenResult = PResult<NodeOrToken<GreenNode, GreenToken>>;
type Input<'s> = Stateful<&'s str, State>;

fn tok(kind: SyntaxKind, text: &str) -> NodeOrToken<GreenNode, GreenToken> {
    NodeOrToken::Token(GreenToken::new(kind.into(), text))
}
fn node<I>(kind: SyntaxKind, children: I) -> NodeOrToken<GreenNode, GreenToken>
where
    I: IntoIterator<Item = NodeOrToken<GreenNode, GreenToken>>,
    I::IntoIter: ExactSizeIterator,
{
    NodeOrToken::Node(GreenNode::new(kind.into(), children))
}
fn ascii_char<const C: char>(
    kind: SyntaxKind,
) -> impl FnMut(&mut Input) -> PResult<NodeOrToken<GreenNode, GreenToken>> {
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
    dispatch! {peek(any);
        '&' => (anchor_property, opt((comments_or_whitespaces1, tag_property))),
        '!' => (cut_err(tag_property), opt((comments_or_whitespaces1, anchor_property))),
        _ => fail,
    }
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
    (
        '"',
        cut_err((take_escaped(none_of(['\\', '"']), '\\', any), '"')),
    )
        .recognize()
        .context(StrContext::Expected(StrContextValue::CharLiteral('"')))
        .parse_next(input)
        .map(|text| tok(DOUBLE_QUOTED_SCALAR, text))
}

fn single_qouted_scalar(input: &mut Input) -> GreenResult {
    (
        '\'',
        cut_err((
            repeat::<_, _, (), _, _>(0.., alt((none_of('\'').void(), "''".void()))),
            '\'',
        )),
    )
        .recognize()
        .context(StrContext::Expected(StrContextValue::CharLiteral('\'')))
        .parse_next(input)
        .map(|text| tok(SINGLE_QUOTED_SCALAR, text))
}

fn plain_scalar(input: &mut Input) -> GreenResult {
    let indent = input.state.indent;
    if matches!(
        input.state.bf_ctx,
        BlockFlowCtx::FlowIn | BlockFlowCtx::FlowOut
    ) {
        (
            plain_scalar_one_line,
            repeat::<_, _, (), _, _>(
                0..,
                (
                    multispace1.verify(move |text: &str| {
                        if let Some(detected) = detect_ws_indent(text) {
                            detected > indent
                        } else {
                            true
                        }
                    }),
                    plain_scalar_chars,
                ),
            ),
        )
            .recognize()
            .parse_next(input)
            .map(|text| tok(PLAIN_SCALAR, text))
    } else {
        plain_scalar_one_line
            .recognize()
            .parse_next(input)
            .map(|text| tok(PLAIN_SCALAR, text))
    }
}
fn plain_scalar_one_line(input: &mut Input) -> PResult<()> {
    (
        alt((
            none_of(is_indicator),
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
            terminated(':'.void(), peek(none_of(|c: char| c.is_ascii_whitespace()))),
            terminated(space1.void(), peek(none_of('#'))),
        )),
    )
    .parse_next(input)
}

fn flow_sequence(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'['>(L_BRACKET),
        cut_err((
            comments_or_whitespaces0,
            flow_sequence_entries.set_state(flow_collection_state),
            ascii_char::<']'>(R_BRACKET),
        )),
    )
        .context(StrContext::Expected(StrContextValue::CharLiteral(']')))
        .parse_next(input)
        .map(|(l_bracket, (mut leading_trivias, entries, r_bracket))| {
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
                comments_or_whitespaces0,
                alt((ascii_char::<','>(COMMA).map(Some), peek(']').value(None))),
            )
                .map(Either::Left),
            comments_or_whitespaces1.map(Either::Right),
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
    alt((flow, flow_pair))
        .parse_next(input)
        .map(|child| node(FLOW_SEQ_ENTRY, [child]))
}

fn flow_map(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'{'>(L_BRACE),
        cut_err((
            comments_or_whitespaces0,
            flow_map_entries.set_state(flow_collection_state),
            ascii_char::<'}'>(R_BRACE),
        )),
    )
        .context(StrContext::Expected(StrContextValue::CharLiteral('}')))
        .parse_next(input)
        .map(|(l_brace, (mut leading_trivias, entries, r_brace))| {
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
                comments_or_whitespaces0,
                alt((ascii_char::<','>(COMMA).map(Some), peek('}').value(None))),
            )
                .map(Either::Left),
            comments_or_whitespaces1.map(Either::Right),
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
            opt(flow_map_entry_key),
            comments_or_whitespaces0,
            ascii_char::<':'>(COLON),
            opt((comments_or_whitespaces0, flow)),
        )
            .map(Either::Left),
        flow_map_entry_key.map(Either::Right),
    ))
    .parse_next(input)
    .map(|either| match either {
        Either::Left((key, mut trivias_before_colon, colon, value)) => {
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
            node(FLOW_MAP_ENTRY, children)
        }
        Either::Right(key) => node(FLOW_MAP_ENTRY, [key]),
    })
}

fn flow_pair(input: &mut Input) -> GreenResult {
    (
        opt(flow_map_entry_key.set_state(|state| state.bf_ctx = BlockFlowCtx::FlowKey)),
        comments_or_whitespaces0,
        ascii_char::<':'>(COLON),
        opt((comments_or_whitespaces0, flow)),
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
        flow.map(Either::Left),
        (
            ascii_char::<'?'>(QUESTION_MARK),
            opt((whitespace, comments_or_whitespaces0, flow)),
        )
            .map(Either::Right),
    ))
    .parse_next(input)
    .map(|either| match either {
        Either::Left(key) => node(FLOW_MAP_KEY, [key]),
        Either::Right((question_mark, key)) => {
            let mut children = Vec::with_capacity(3);
            children.push(question_mark);
            if let Some((ws, mut trivias, key)) = key {
                children.push(ws);
                children.append(&mut trivias);
                children.push(key);
            }
            node(FLOW_MAP_KEY, children)
        }
    })
}

fn flow_content(input: &mut Input) -> GreenResult {
    dispatch! {peek(any);
        '"' => double_qouted_scalar,
        '\'' => single_qouted_scalar,
        '[' => flow_sequence,
        '{' => flow_map,
        _ => plain_scalar,
    }
    .parse_next(input)
}

fn flow(input: &mut Input) -> GreenResult {
    dispatch! {peek(any);
        '*' => alias.map(|child| node(FLOW, [child])),
        '&' | '!' => (properties, opt((comments_or_whitespaces1, flow_content))).map(|(properties, content)| {
            let mut children = Vec::with_capacity(3);
            children.push(properties);
            if let Some((mut trivias, content)) = content {
                children.append(&mut trivias);
                children.push(content);
            }
            node(FLOW, children)
        }),
        _ => flow_content.map(|child| node(FLOW, [child])),
    }
    .parse_next(input)
}

fn block_scalar(input: &mut Input) -> GreenResult {
    (
        (alt((ascii_char::<'|'>(BAR), ascii_char::<'>'>(GREATER_THAN)))),
        alt((
            (opt(indent_indicator), opt(chomping_indicator)).map(Either::Left),
            (opt(chomping_indicator), opt(indent_indicator)).map(Either::Right),
        )),
        comments_or_spaces,
        peek(opt(multispace1.verify_map(detect_ws_indent))),
    )
        .flat_map(|(style, indicator, mut trivias, mut indent)| {
            let mut children = Vec::with_capacity(3);
            children.push(style);
            match indicator {
                Either::Left((indent_token, chomping_token)) => {
                    if let Some((indent_token, indent_value)) = indent_token {
                        children.push(indent_token);
                        indent = Some(indent_value);
                    }
                    if let Some(chomping) = chomping_token {
                        children.push(chomping);
                    }
                }
                Either::Right((chomping_token, indent_token)) => {
                    if let Some(chomping) = chomping_token {
                        children.push(chomping);
                    }
                    if let Some((indent_token, indent_value)) = indent_token {
                        children.push(indent_token);
                        indent = Some(indent_value);
                    }
                }
            }
            children.append(&mut trivias);
            let indent = indent.unwrap_or_default();
            repeat::<_, _, (), _, _>(
                0..,
                (
                    multispace1.verify(move |text: &str| {
                        detect_ws_indent(text).is_some_and(|detected| detected >= indent)
                    }),
                    take_till(0.., ['\n', '\r']),
                ),
            )
            .recognize()
            .map(move |text| {
                let mut children = children.clone();
                children.push(tok(BLOCK_SCALAR_TEXT, text));
                node(BLOCK_SCALAR, children)
            })
        })
        .parse_next(input)
}
fn indent_indicator(input: &mut Input) -> PResult<(NodeOrToken<GreenNode, GreenToken>, usize)> {
    one_of(|c: char| c.is_ascii_digit())
        .recognize()
        .try_map(|text: &str| {
            text.parse()
                .map(|value| (tok(INDENT_INDICATOR, text), value))
        })
        .parse_next(input)
}
fn chomping_indicator(input: &mut Input) -> GreenResult {
    alt((ascii_char::<'-'>(MINUS), ascii_char::<'+'>(PLUS))).parse_next(input)
}

fn block_sequence(input: &mut Input) -> GreenResult {
    (
        block_sequence_entry,
        repeat(
            0..,
            (
                comments_or_whitespaces1.verify_indent(),
                cut_err(block_sequence_entry),
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
            node(BLOCK_SEQ, children)
        })
}

fn block_sequence_entry(input: &mut Input) -> GreenResult {
    (
        ascii_char::<'-'>(MINUS).context(StrContext::Expected(StrContextValue::CharLiteral('-'))),
        opt(alt((
            (
                space_before_block_compact_collection.track_indent(),
                alt((block_sequence, block_map)),
            )
                .map(|(space, collection)| (vec![space], collection)),
            (comments_or_whitespaces1.track_indent(), block),
        )))
        .set_state(|state| state.bf_ctx = BlockFlowCtx::BlockIn),
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
fn space_before_block_compact_collection(input: &mut Input) -> GreenResult {
    let (space, text) = space.with_recognized().parse_next(input)?;
    input.state.indent += text.len() + 1;
    Ok(space)
}

fn block_map(input: &mut Input) -> GreenResult {
    (
        alt((block_map_implicit_entry, block_map_explicit_entry)),
        repeat(
            0..,
            (
                comments_or_whitespaces1.verify_indent(),
                cut_err(
                    alt((block_map_implicit_entry, block_map_explicit_entry))
                        .context(StrContext::Label("block map entry")),
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
    (
        block_map_explicit_key,
        opt((
            comments_or_whitespaces0.verify_indent(),
            ascii_char::<':'>(COLON),
            opt((comments_or_whitespaces1.track_indent(), block)
                .set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut)),
        )),
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
        opt((
            space,
            comments_or_whitespaces0,
            block.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut),
        )),
    )
        .parse_next(input)
        .map(|(question_mark, key)| {
            if let Some((space, mut trivias, key)) = key {
                let mut children = Vec::with_capacity(3);
                children.push(question_mark);
                children.push(space);
                children.append(&mut trivias);
                children.push(key);
                node(BLOCK_MAP_KEY, children)
            } else {
                node(BLOCK_MAP_KEY, [question_mark])
            }
        })
}

fn block_map_implicit_entry(input: &mut Input) -> GreenResult {
    (
        opt((block_map_implicit_key, opt(space))),
        ascii_char::<':'>(COLON),
        comments_or_whitespaces1.track_indent(),
        block.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockOut),
    )
        .parse_next(input)
        .map(|(key, colon, mut trivias, value)| {
            let mut children = Vec::with_capacity(4);
            if let Some((key, space)) = key {
                children.push(key);
                if let Some(space) = space {
                    children.push(space);
                }
            }
            children.push(colon);
            children.append(&mut trivias);
            children.push(node(BLOCK_MAP_VALUE, [value]));
            node(BLOCK_MAP_ENTRY, children)
        })
}

fn block_map_implicit_key(input: &mut Input) -> GreenResult {
    flow.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockKey)
        .parse_next(input)
        .map(|child| node(BLOCK_MAP_KEY, [child]))
}

fn block(input: &mut Input) -> GreenResult {
    alt((
        (
            opt((properties, comments_or_whitespaces1)),
            alt((block_sequence, block_map, block_scalar)),
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
        flow.set_state(|state| state.bf_ctx = BlockFlowCtx::FlowOut)
            .map(|child| node(BLOCK, [child])),
    ))
    .parse_next(input)
}

fn directives_end(input: &mut Input) -> GreenResult {
    "---"
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
        take_till(0.., is_url_char),
    )
        .recognize()
        .parse_next(input)
        .map(|text| tok(TAG_PREFIX, text))
}

fn reserved_directive(input: &mut Input) -> GreenResult {
    (
        take_till(1.., |c: char| c.is_ascii_whitespace()),
        space,
        take_till(1.., |c: char| c.is_ascii_whitespace()),
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
    (
        repeat(0.., (directive, comments_or_whitespaces0)),
        opt((directives_end, comments_or_whitespaces0)),
        block.set_state(|state| state.bf_ctx = BlockFlowCtx::BlockIn),
    )
        .parse_next(input)
        .map(|(directives, end, block): (Vec<_>, _, _)| {
            let mut children = Vec::with_capacity(2 + directives.len());
            directives.into_iter().for_each(|(directive, mut trivias)| {
                children.push(directive);
                children.append(&mut trivias);
            });
            if let Some((end, mut trivias)) = end {
                children.push(end);
                children.append(&mut trivias);
            }
            children.push(block);
            node(DOCUMENT, children)
        })
}

fn document_end(input: &mut Input) -> GreenResult {
    "...".map(|text| tok(DOCUMENT_END, text)).parse_next(input)
}

fn root(input: &mut Input) -> PResult<SyntaxNode> {
    repeat(0.., alt((comments_or_whitespaces, document_end, document)))
        .parse_next(input)
        .map(|children: Vec<_>| SyntaxNode::new_root(GreenNode::new(ROOT.into(), children)))
}

fn comment(input: &mut Input) -> GreenResult {
    ('#', till_line_ending)
        .recognize()
        .parse_next(input)
        .map(|text| tok(COMMENT, text))
}

fn space(input: &mut Input) -> GreenResult {
    space1.parse_next(input).map(|text| tok(WHITESPACE, text))
}

fn whitespace(input: &mut Input) -> GreenResult {
    let text = multispace1.parse_next(input)?;
    if let Some(indent) = detect_ws_indent(text) {
        input.state.indent = indent;
    }
    Ok(tok(WHITESPACE, text))
}

fn comments_or_spaces(input: &mut Input) -> PResult<Vec<NodeOrToken<GreenNode, GreenToken>>> {
    repeat(0.., alt((comment, space))).parse_next(input)
}
fn comments_or_whitespaces(input: &mut Input) -> GreenResult {
    dispatch! {peek(any);
        ' ' | '\n' | '\t' | '\r' => whitespace,
        '#' => comment,
        _ => fail,
    }
    .parse_next(input)
}
fn comments_or_whitespaces0(input: &mut Input) -> PResult<Vec<NodeOrToken<GreenNode, GreenToken>>> {
    repeat(0.., comments_or_whitespaces).parse_next(input)
}
fn comments_or_whitespaces1(input: &mut Input) -> PResult<Vec<NodeOrToken<GreenNode, GreenToken>>> {
    repeat(1.., comments_or_whitespaces).parse_next(input)
}

pub fn parse(code: &str) -> Result<SyntaxNode, SyntaxError> {
    let code = code.trim_start_matches('\u{feff}');
    let base_indent = detect_base_indent(code).unwrap_or_default();
    let input = Stateful {
        input: code,
        state: State {
            indent: base_indent,
            tracked_indents: 1 << base_indent,
            bf_ctx: BlockFlowCtx::BlockIn,
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
    indent: usize,
    // Does someone's YAML file has more than 63 columns of indentation?
    tracked_indents: u64,
    bf_ctx: BlockFlowCtx,
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
