use crate::{config::Quotes, ctx::Ctx};
use rowan::Direction;
use tiny_pretty::Doc;
use yaml_parser::{ast::*, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub(super) trait DocGen {
    fn doc(&self, ctx: &Ctx) -> Doc<'static>;
}

impl DocGen for Alias {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        let mut docs = vec![Doc::text("*")];
        if let Some(name) = self.anchor_name() {
            docs.push(Doc::text(name.to_string()));
        }
        Doc::list(docs)
    }
}

impl DocGen for AnchorProperty {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        let mut docs = vec![Doc::text("&")];
        if let Some(name) = self.anchor_name() {
            docs.push(Doc::text(name.to_string()));
        }
        Doc::list(docs)
    }
}

impl DocGen for Block {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(1);
        let mut trivia_after_props_docs = vec![];
        let has_properties = if let Some(properties) = self.properties() {
            docs.push(properties.doc(ctx));
            if let Some(token) = properties
                .syntax()
                .next_sibling_or_token()
                .and_then(SyntaxElement::into_token)
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
            {
                trivia_after_props_docs = format_trivias_after_token(&token, ctx);
            }
            true
        } else {
            false
        };
        if let Some(block_map) = self.block_map() {
            if has_properties {
                if !trivia_after_props_docs.is_empty() {
                    docs.push(Doc::space());
                    docs.append(&mut trivia_after_props_docs);
                } else {
                    docs.push(Doc::hard_line());
                }
            }
            docs.push(block_map.doc(ctx));
        } else if let Some(block_seq) = self.block_seq() {
            if has_properties {
                if !trivia_after_props_docs.is_empty() {
                    docs.push(Doc::space());
                    docs.append(&mut trivia_after_props_docs);
                } else {
                    docs.push(Doc::hard_line());
                }
            }
            docs.push(block_seq.doc(ctx));
        } else if let Some(block_scalar) = self.block_scalar() {
            if has_properties {
                docs.push(Doc::space());
                if !trivia_after_props_docs.is_empty() {
                    docs.append(&mut trivia_after_props_docs);
                }
            }
            docs.push(block_scalar.doc(ctx));
        }
        Doc::list(docs)
    }
}

impl DocGen for BlockMap {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        Doc::list(format_line_break_separated_list::<_, BlockMapEntry, false>(
            self, ctx,
        ))
    }
}

impl DocGen for BlockMapEntry {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_key_value_pair(self.key(), self.colon(), self.value(), ctx)
    }
}

impl DocGen for BlockMapKey {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let question_mark = self.question_mark();
        if let Some(block) = self.block() {
            format_key(self, question_mark, Some(block), ctx)
        } else {
            format_key(self, question_mark, self.flow(), ctx)
        }
    }
}

impl DocGen for BlockMapValue {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        if let Some(block) = self.block() {
            block.doc(ctx)
        } else if let Some(flow) = self.flow() {
            flow.doc(ctx)
        } else {
            unreachable!("expected block or flow in block map value")
        }
    }
}

impl DocGen for BlockScalar {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        Doc::list(
            self.syntax()
                .children_with_tokens()
                .map(|element| match element {
                    SyntaxElement::Token(token) => match token.kind() {
                        SyntaxKind::WHITESPACE => Doc::nil(),
                        SyntaxKind::COMMENT => Doc::space().append(format_comment(&token, ctx)),
                        SyntaxKind::BLOCK_SCALAR_TEXT => {
                            let text = token.text();
                            if self
                                .syntax()
                                .children_with_tokens()
                                .any(|element| element.kind() == SyntaxKind::INDENT_INDICATOR)
                            {
                                return Doc::list(reflow(token.text()).collect());
                            }
                            let space_len = text.find(|c: char| !c.is_ascii_whitespace()).map(
                                |first_contentful| {
                                    let first_linebreak = text[..first_contentful].rfind('\n');
                                    if let Some(first_linbreak) = first_linebreak {
                                        (first_contentful - first_linbreak).saturating_sub(1)
                                    } else {
                                        first_contentful
                                    }
                                },
                            );
                            if let Some(space_len) = space_len {
                                let lines = text.lines().map(|s| {
                                    if s.trim().is_empty() {
                                        String::new()
                                    } else {
                                        s[space_len..].trim_end().to_owned()
                                    }
                                });
                                let mut docs = vec![];
                                intersperse_lines(&mut docs, lines);
                                Doc::list(docs).nest(ctx.indent_width)
                            } else {
                                Doc::nil()
                            }
                        }
                        _ => Doc::text(token.to_string()),
                    },
                    SyntaxElement::Node(node) => Doc::text(node.to_string()),
                })
                .collect(),
        )
    }
}

impl DocGen for BlockSeq {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        Doc::list(format_line_break_separated_list::<_, BlockSeqEntry, false>(
            self, ctx,
        ))
    }
}

impl DocGen for BlockSeqEntry {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(3);

        if let Some(token) = self.minus() {
            docs.push(Doc::text("-"));
            if let Some(token) = token
                .next_sibling_or_token()
                .and_then(SyntaxElement::into_token)
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
            {
                let mut trivia_docs = format_trivias_after_token(&token, ctx);
                docs.push(Doc::space());
                docs.append(&mut trivia_docs);
            } else if self.block().is_some() || self.flow().is_some() {
                docs.push(Doc::space());
            }
        }

        if let Some(block) = self.block() {
            docs.push(block.doc(ctx));
        } else if let Some(flow) = self.flow() {
            docs.push(flow.doc(ctx));
        }

        Doc::list(docs).nest(ctx.indent_width)
    }
}

impl DocGen for Directive {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(2);
        docs.push(Doc::text("%"));
        if let Some(tag) = self.tag_directive() {
            docs.push(tag.doc(ctx));
        } else if let Some(yaml) = self.yaml_directive() {
            docs.push(yaml.doc(ctx));
        } else if let Some(reserved) = self.reserved_directive() {
            docs.push(reserved.doc(ctx));
        }
        Doc::list(docs)
    }
}

impl DocGen for Document {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(2);

        let mut children = self.syntax().children_with_tokens().peekable();
        while let Some(element) = children.next() {
            match element {
                SyntaxElement::Node(node) => {
                    if should_ignore(&node, ctx) {
                        docs.extend(reflow(&node.to_string()));
                    } else {
                        match node.kind() {
                            SyntaxKind::BLOCK => {
                                if let Some(block) = Block::cast(node) {
                                    docs.push(block.doc(ctx));
                                }
                            }
                            SyntaxKind::FLOW => {
                                if let Some(flow) = Flow::cast(node) {
                                    docs.push(flow.doc(ctx));
                                }
                            }
                            SyntaxKind::DIRECTIVE => {
                                if let Some(directive) = Directive::cast(node) {
                                    docs.push(directive.doc(ctx));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                SyntaxElement::Token(token) => match token.kind() {
                    SyntaxKind::COMMENT => {
                        docs.push(format_comment(&token, ctx));
                    }
                    SyntaxKind::WHITESPACE => {
                        match token.text().chars().filter(|c| *c == '\n').count() {
                            0 => {
                                if children
                                    .peek()
                                    .is_some_and(|element| element.kind() == SyntaxKind::COMMENT)
                                {
                                    docs.push(Doc::space());
                                } else {
                                    docs.push(Doc::hard_line());
                                }
                            }
                            1 => {
                                docs.push(Doc::hard_line());
                            }
                            _ => {
                                docs.push(Doc::empty_line());
                                docs.push(Doc::hard_line());
                            }
                        }
                    }
                    SyntaxKind::DIRECTIVES_END => {
                        docs.push(Doc::text("---"));
                    }
                    SyntaxKind::DOCUMENT_END => {
                        docs.push(Doc::text("..."));
                    }
                    _ => {}
                },
            }
        }

        Doc::list(docs)
    }
}

impl DocGen for Flow {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(1);
        if let Some(properties) = self.properties() {
            docs.push(properties.doc(ctx));
            docs.push(Doc::space());
            if let Some(token) = properties
                .syntax()
                .next_sibling_or_token()
                .and_then(SyntaxElement::into_token)
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
            {
                let mut trivia_docs = format_trivias_after_token(&token, ctx);
                docs.append(&mut trivia_docs);
            }
        }
        if let Some(double_quoted) = self.double_qouted_scalar() {
            let text = double_quoted.text();
            let is_double_preferred = matches!(ctx.options.quotes, Quotes::PreferDouble);
            let (quotes_option, quote) = if is_double_preferred || text.contains(['\\', '\'']) {
                (None, "\"")
            } else {
                (Some(&ctx.options.quotes), "'")
            };
            docs.push(Doc::text(quote));
            let lines = text
                .get(1..text.len() - 1)
                .expect("expected double quoted scalar")
                .split('\n')
                .map(|s| format_quoted_scalar(s, quotes_option));
            intersperse_lines(&mut docs, lines);
            docs.push(Doc::text(quote));
        } else if let Some(single_quoted) = self.single_quoted_scalar() {
            let text = single_quoted.text();
            let is_single_preferred = matches!(ctx.options.quotes, Quotes::PreferSingle);
            let (quotes_option, quote) = if is_single_preferred || text.contains(['\\', '"']) {
                (None, "'")
            } else {
                (Some(&ctx.options.quotes), "\"")
            };
            docs.push(Doc::text(quote));
            let lines = text
                .get(1..text.len() - 1)
                .expect("expected single quoted scalar")
                .split('\n')
                .map(|s| format_quoted_scalar(s, quotes_option));
            intersperse_lines(&mut docs, lines);
            docs.push(Doc::text(quote));
        } else if let Some(plain) = self.plain_scalar() {
            let lines = plain.text().lines().map(|s| s.trim().to_owned());
            intersperse_lines(&mut docs, lines);
        } else if let Some(flow_seq) = self.flow_seq() {
            docs.push(flow_seq.doc(ctx));
        } else if let Some(flow_map) = self.flow_map() {
            docs.push(flow_map.doc(ctx));
        } else if let Some(alias) = self.alias() {
            docs.push(alias.doc(ctx));
        }
        Doc::list(docs)
    }
}

impl DocGen for FlowMap {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        if self
            .entries()
            .is_some_and(|entries| entries.syntax().children_with_tokens().count() == 0)
            && self
                .syntax()
                .children_with_tokens()
                .all(|element| element.kind() != SyntaxKind::COMMENT)
        {
            return Doc::text("{}");
        }

        let mut docs = vec![Doc::text("{")];
        if let Some(token) = self
            .l_brace()
            .and_then(|token| token.next_token())
            .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
        {
            if token.text().contains(['\n', '\r']) {
                docs.push(Doc::hard_line());
            } else {
                docs.push(Doc::line_or_space());
            }
            let mut trivia_docs = format_trivias_after_token(&token, ctx);
            docs.append(&mut trivia_docs);
        } else {
            docs.push(Doc::line_or_space());
        }

        let mut has_trailing_comment = false;
        if let Some(entries) = self.entries() {
            docs.push(entries.doc(ctx));
            let last_ws_index = self
                .r_brace()
                .and_then(|token| token.prev_token())
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
                .map(|token| token.index());
            if let Some(index) = last_ws_index {
                let mut trivia_docs = format_trivias(
                    entries
                        .syntax()
                        .siblings_with_tokens(Direction::Next)
                        .filter(|element| element.index() != index),
                    &mut has_trailing_comment,
                    ctx,
                );
                docs.append(&mut trivia_docs);
            }
        }

        Doc::list(docs)
            .nest(ctx.indent_width)
            .append(if has_trailing_comment {
                Doc::hard_line()
            } else {
                Doc::line_or_space()
            })
            .append(Doc::text("}"))
            .group()
    }
}

impl DocGen for FlowMapEntries {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_flow_collection_entries(self, self.entries(), ctx)
    }
}

impl DocGen for FlowMapEntry {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_key_value_pair(self.key(), self.colon(), self.value(), ctx)
    }
}

impl DocGen for FlowMapKey {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_key(self, self.question_mark(), self.flow(), ctx)
    }
}

impl DocGen for FlowMapValue {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        self.flow()
            .map(|flow| flow.doc(ctx))
            .unwrap_or_else(|| Doc::nil())
    }
}

impl DocGen for FlowPair {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_key_value_pair(self.key(), self.colon(), self.value(), ctx)
    }
}

impl DocGen for FlowSeq {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        if self
            .entries()
            .is_some_and(|entries| entries.syntax().children_with_tokens().count() == 0)
            && self
                .syntax()
                .children_with_tokens()
                .all(|element| element.kind() != SyntaxKind::COMMENT)
        {
            return Doc::text("[]");
        }

        let mut docs = vec![Doc::text("[")];
        if let Some(token) = self
            .l_bracket()
            .and_then(|token| token.next_token())
            .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
        {
            if token.text().contains(['\n', '\r']) {
                docs.push(Doc::hard_line());
            } else {
                docs.push(Doc::line_or_nil());
            }
            let mut trivia_docs = format_trivias_after_token(&token, ctx);
            docs.append(&mut trivia_docs);
        } else {
            docs.push(Doc::line_or_nil());
        }

        let mut has_trailing_comment = false;
        if let Some(entries) = self.entries() {
            docs.push(entries.doc(ctx));
            let last_ws_index = self
                .r_bracket()
                .and_then(|token| token.prev_token())
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
                .map(|token| token.index());
            if let Some(index) = last_ws_index {
                let mut trivia_docs = format_trivias(
                    entries
                        .syntax()
                        .siblings_with_tokens(Direction::Next)
                        .filter(|element| element.index() != index),
                    &mut has_trailing_comment,
                    ctx,
                );
                docs.append(&mut trivia_docs);
            }
        }

        Doc::list(docs)
            .nest(ctx.indent_width)
            .append(if has_trailing_comment {
                Doc::hard_line()
            } else {
                Doc::line_or_nil()
            })
            .append(Doc::text("]"))
            .group()
    }
}

impl DocGen for FlowSeqEntries {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        format_flow_collection_entries(self, self.entries(), ctx)
    }
}

impl DocGen for FlowSeqEntry {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        if let Some(flow) = self.flow() {
            flow.doc(ctx)
        } else if let Some(flow_pair) = self.flow_pair() {
            flow_pair.doc(ctx)
        } else {
            unreachable!("expected flow or flow pair in flow sequence entry")
        }
    }
}

impl DocGen for NonSpecificTag {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        Doc::text("!")
    }
}

impl DocGen for Properties {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        Doc::list(
            self.syntax()
                .children_with_tokens()
                .map(|element| match element {
                    SyntaxElement::Token(token) => match token.kind() {
                        SyntaxKind::WHITESPACE => {
                            if token.text().contains(['\n', '\r']) {
                                Doc::hard_line()
                            } else {
                                Doc::line_or_space()
                            }
                        }
                        SyntaxKind::COMMENT => format_comment(&token, ctx),
                        _ => Doc::text(token.to_string()),
                    },
                    SyntaxElement::Node(node) => {
                        if let Some(anchor) = AnchorProperty::cast(node.clone()) {
                            anchor.doc(ctx)
                        } else if let Some(tag) = TagProperty::cast(node) {
                            tag.doc(ctx)
                        } else {
                            unreachable!("expected tag property or anchor property in properties")
                        }
                    }
                })
                .collect(),
        )
        .group()
    }
}

impl DocGen for ReservedDirective {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(3);
        if let Some(name) = self.directive_name() {
            docs.push(Doc::text(name.to_string()));
        }
        if let Some(param) = self.directive_param() {
            docs.push(Doc::space());
            docs.push(Doc::text(param.to_string()));
        }
        Doc::list(docs)
    }
}

impl DocGen for Root {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = format_line_break_separated_list::<_, Document, true>(self, ctx);
        docs.push(Doc::hard_line());
        Doc::list(docs)
    }
}

impl DocGen for ShorthandTag {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = Vec::with_capacity(2);
        if let Some(tag_handle) = self.tag_handle() {
            docs.push(tag_handle.doc(ctx));
        }
        if let Some(tag_char) = self.tag_char() {
            docs.push(Doc::text(tag_char.to_string()));
        }
        Doc::list(docs)
    }
}

impl DocGen for TagDirective {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        let mut docs = vec![Doc::text("TAG")];
        if let Some(tag_handle) = self.tag_handle() {
            docs.push(Doc::space());
            docs.push(tag_handle.doc(ctx));
        }
        if let Some(tag_prefix) = self.tag_prefix() {
            docs.push(Doc::space());
            docs.push(Doc::text(tag_prefix.to_string()));
        }
        Doc::list(docs)
    }
}

impl DocGen for TagHandle {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        if let Some(primary) = self.primary() {
            Doc::text(primary.to_string())
        } else if let Some(secondary) = self.secondary() {
            Doc::text(secondary.to_string())
        } else if let Some(named) = self.named() {
            Doc::text(named.to_string())
        } else {
            unreachable!("expected primary, secondary or named in tag handle")
        }
    }
}

impl DocGen for TagProperty {
    fn doc(&self, ctx: &Ctx) -> Doc<'static> {
        if let Some(shorthand) = self.shorthand_tag() {
            shorthand.doc(ctx)
        } else if let Some(non_specific) = self.non_specific_tag() {
            non_specific.doc(ctx)
        } else if let Some(verbatim) = self.verbatim_tag() {
            Doc::text(verbatim.to_string())
        } else {
            unreachable!("expected shorthand tag or non specific tag in tag property")
        }
    }
}

impl DocGen for YamlDirective {
    fn doc(&self, _: &Ctx) -> Doc<'static> {
        if let Some(version) = self.yaml_version() {
            Doc::text(format!("YAML {}", version.text()))
        } else {
            Doc::text("YAML")
        }
    }
}

fn format_key<K, C>(
    key: &K,
    question_mark: Option<SyntaxToken>,
    content: Option<C>,
    ctx: &Ctx,
) -> Doc<'static>
where
    K: AstNode,
    C: AstNode + DocGen,
{
    let mut docs = Vec::with_capacity(1);

    let mut has_line_break = false;
    if let Some(question_mark) = question_mark {
        if !can_omit_question_mark(key.syntax()) {
            docs.push(Doc::text("?"));
            if content.is_some() {
                docs.push(Doc::space());
            }
        }
        if let Some(token) = question_mark
            .next_token()
            .filter(|token| token.kind() == SyntaxKind::WHITESPACE && content.is_some())
        {
            if token.text().contains(['\n', '\r']) {
                docs.push(Doc::hard_line());
                has_line_break = true;
            }
            let last_ws_index = content
                .as_ref()
                .and_then(|content| content.syntax().prev_sibling_or_token())
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
                .map(|token| token.index());
            if let Some(index) = last_ws_index {
                let mut has_comment = false;
                let mut trivia_docs = format_trivias(
                    token
                        .siblings_with_tokens(Direction::Next)
                        .filter(|token| token.index() != index),
                    &mut has_comment,
                    ctx,
                );
                docs.append(&mut trivia_docs);
                if has_comment {
                    docs.push(Doc::hard_line());
                    has_line_break = true;
                }
            }
        }
    }

    if let Some(content) = &content {
        docs.push(content.doc(ctx));
    }

    let doc = Doc::list(docs).group();
    if has_line_break
        || content
            .iter()
            .flat_map(|content| content.syntax().children_with_tokens())
            .any(|element| {
                if let SyntaxElement::Token(token) = element {
                    token.text().contains(['\n', '\r'])
                } else {
                    false
                }
            })
    {
        doc.nest(ctx.indent_width)
    } else {
        doc
    }
}

fn format_key_value_pair<K, V>(
    key: Option<K>,
    colon: Option<SyntaxToken>,
    value: Option<V>,
    ctx: &Ctx,
) -> Doc<'static>
where
    K: AstNode + DocGen,
    V: AstNode + DocGen,
{
    let mut docs = Vec::with_capacity(4);

    let mut trivia_before_colon_docs = vec![];
    let mut has_question_mark = false;
    if let Some(key) = key {
        has_question_mark = key
            .syntax()
            .children_with_tokens()
            .any(|node| node.kind() == SyntaxKind::QUESTION_MARK)
            && !can_omit_question_mark(key.syntax());
        docs.push(key.doc(ctx));
        if let Some(token) = key
            .syntax()
            .next_sibling_or_token()
            .and_then(SyntaxElement::into_token)
            .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
        {
            trivia_before_colon_docs = format_trivias_after_token(&token, ctx);
        }

        if key
            .syntax()
            .children()
            .find(|node| node.kind() == SyntaxKind::FLOW)
            .iter()
            .flat_map(|flow| flow.children())
            .any(|child| child.kind() == SyntaxKind::ALIAS)
        {
            docs.push(Doc::space());
        }
    }

    let has_trivias_before_colon = !trivia_before_colon_docs.is_empty();
    if let Some(colon) = colon {
        if has_question_mark {
            if trivia_before_colon_docs.is_empty() {
                docs.push(Doc::hard_line());
            } else {
                docs.push(Doc::space());
                docs.push(Doc::list(trivia_before_colon_docs));
            }
            docs.push(Doc::text(":"));
        } else {
            docs.push(Doc::text(":"));
            if !trivia_before_colon_docs.is_empty() {
                docs.push(Doc::space());
                docs.push(Doc::list(trivia_before_colon_docs).nest(ctx.indent_width));
            }
        }

        let mut has_line_break = false;

        if let Some(value) = value {
            let mut value_docs = vec![];
            if let Some(token) = colon
                .next_token()
                .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
            {
                if value.syntax().kind() == SyntaxKind::FLOW_MAP_VALUE {
                    value_docs.push(Doc::space());
                } else if token.text().contains(['\n', '\r']) {
                    value_docs.push(Doc::hard_line());
                    has_line_break = true;
                } else {
                    value_docs.push(Doc::space());
                }
                let last_ws_index = value
                    .syntax()
                    .prev_sibling_or_token()
                    .and_then(SyntaxElement::into_token)
                    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
                    .map(|token| token.index());
                if let Some(index) = last_ws_index {
                    let mut has_comment = false;
                    let mut trivia_docs = format_trivias(
                        token
                            .siblings_with_tokens(Direction::Next)
                            .filter(|token| token.index() != index),
                        &mut has_comment,
                        ctx,
                    );
                    value_docs.append(&mut trivia_docs);
                    if has_comment {
                        value_docs.push(Doc::hard_line());
                        has_line_break = true;
                    }
                }
            } else if !has_trivias_before_colon {
                docs.push(Doc::space());
            }
            let doc = Doc::list(value_docs).append(value.doc(ctx));
            if value
                .syntax()
                .children()
                .find(|child| child.kind() == SyntaxKind::BLOCK)
                .iter()
                .flat_map(|block| block.children())
                .any(|child| child.kind() == SyntaxKind::BLOCK_SEQ)
            {
                if ctx.options.indent_block_sequence_in_map {
                    docs.push(doc.nest(ctx.indent_width));
                } else {
                    docs.push(doc);
                }
            } else if has_line_break
                || value
                    .syntax()
                    .children()
                    .find(|child| child.kind() == SyntaxKind::BLOCK)
                    .iter()
                    .flat_map(|block| block.children())
                    .any(|child| child.kind() == SyntaxKind::BLOCK_MAP)
                || value
                    .syntax()
                    .children()
                    .find(|child| child.kind() == SyntaxKind::FLOW)
                    .iter()
                    .flat_map(|block| block.children_with_tokens())
                    .any(|element| {
                        if let SyntaxElement::Token(token) = element {
                            token.text().contains(['\n', '\r'])
                        } else {
                            false
                        }
                    })
            {
                docs.push(doc.nest(ctx.indent_width));
            } else {
                docs.push(doc);
            }
        }
    }

    Doc::list(docs).group()
}

fn format_flow_collection_entries<N, Entry>(
    node: &N,
    entries: AstChildren<Entry>,
    ctx: &Ctx,
) -> Doc<'static>
where
    N: AstNode,
    Entry: AstNode + DocGen,
{
    let mut docs = vec![];
    let mut entries = entries.peekable();
    let mut commas = node
        .syntax()
        .children_with_tokens()
        .filter_map(|element| match element {
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::COMMA => Some(token),
            _ => None,
        });
    while let Some(entry) = entries.next() {
        docs.push(entry.doc(ctx));
        if entries.peek().is_some() {
            docs.push(Doc::text(","));
        } else if ctx.options.trailing_comma {
            docs.push(Doc::flat_or_break(Doc::nil(), Doc::text(",")));
        }

        let comma = commas.next();
        let mut has_comment_before_comma = false;
        let last_ws_index = comma
            .as_ref()
            .and_then(|comma| comma.prev_token())
            .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
            .map(|token| token.index());
        if let Some(index) = last_ws_index {
            let mut trivia_docs = format_trivias(
                entry
                    .syntax()
                    .siblings_with_tokens(Direction::Next)
                    .filter(|token| token.index() != index),
                &mut has_comment_before_comma,
                ctx,
            );
            docs.append(&mut trivia_docs);
        }

        if let Some(comma) = &comma {
            let mut trivia_docs = format_trivias(
                comma.siblings_with_tokens(Direction::Next),
                &mut has_comment_before_comma,
                ctx,
            );
            if !trivia_docs.is_empty() {
                docs.append(&mut trivia_docs);
            } else if trivia_docs.is_empty() && entries.peek().is_some() {
                docs.push(Doc::line_or_space());
            }
        }
    }
    Doc::list(docs)
}

fn format_line_break_separated_list<N, Item, const SKIP_SIDE_WS: bool>(
    node: &N,
    ctx: &Ctx,
) -> Vec<Doc<'static>>
where
    N: AstNode,
    Item: AstNode + DocGen,
{
    let mut docs = Vec::with_capacity(2);

    let mut children = node.syntax().children_with_tokens().peekable();
    let mut prev_kind = SyntaxKind::WHITESPACE;
    while let Some(element) = children.next() {
        let kind = element.kind();
        match element {
            SyntaxElement::Node(node) => {
                if should_ignore(&node, ctx) {
                    docs.extend(reflow(&node.to_string()));
                } else if let Some(item) = Item::cast(node) {
                    docs.push(item.doc(ctx));
                }
            }
            SyntaxElement::Token(token) => match token.kind() {
                SyntaxKind::COMMENT => {
                    docs.push(format_comment(&token, ctx));
                }
                SyntaxKind::WHITESPACE => {
                    if !SKIP_SIDE_WS || token.index() > 0 && children.peek().is_some() {
                        match token.text().chars().filter(|c| *c == '\n').count() {
                            0 => {
                                if prev_kind == SyntaxKind::COMMENT {
                                    docs.push(Doc::hard_line());
                                } else {
                                    docs.push(Doc::space());
                                }
                            }
                            1 => {
                                docs.push(Doc::hard_line());
                            }
                            _ => {
                                docs.push(Doc::empty_line());
                                docs.push(Doc::hard_line());
                            }
                        }
                    }
                }
                _ => {}
            },
        }
        prev_kind = kind;
    }

    docs
}

fn format_trivias_after_token(token: &SyntaxToken, ctx: &Ctx) -> Vec<Doc<'static>> {
    let mut _has_comment = false;
    format_trivias(
        token.siblings_with_tokens(Direction::Next),
        &mut _has_comment,
        ctx,
    )
}

fn format_trivias(
    it: impl Iterator<Item = SyntaxElement>,
    has_comment: &mut bool,
    ctx: &Ctx,
) -> Vec<Doc<'static>> {
    let mut docs = vec![];
    let mut trivias = it
        .skip(1)
        .map_while(|element| match element {
            SyntaxElement::Token(token)
                if token.kind() == SyntaxKind::WHITESPACE
                    || token.kind() == SyntaxKind::COMMENT =>
            {
                Some(token)
            }
            _ => None,
        })
        .peekable();
    while let Some(token) = trivias.next() {
        match token.kind() {
            SyntaxKind::WHITESPACE => match token.text().chars().filter(|c| *c == '\n').count() {
                0 => {
                    if *has_comment {
                        docs.push(Doc::hard_line());
                    } else if trivias
                        .peek()
                        .is_some_and(|token| token.kind() == SyntaxKind::COMMENT)
                    {
                        docs.push(Doc::space());
                    } else {
                        docs.push(Doc::line_or_space());
                    }
                }
                1 => {
                    if *has_comment {
                        docs.push(Doc::hard_line());
                    } else {
                        docs.push(Doc::line_or_space());
                    }
                }
                _ => {
                    docs.push(Doc::empty_line());
                    docs.push(Doc::hard_line());
                }
            },
            SyntaxKind::COMMENT => {
                docs.push(format_comment(&token, ctx));
                *has_comment = true;
            }
            _ => {}
        }
    }
    docs
}

fn format_comment(token: &SyntaxToken, ctx: &Ctx) -> Doc<'static> {
    let text = token.text().trim_end();
    if ctx.options.format_comments {
        let content = text.strip_prefix('#').expect("comment must start with '#'");
        if content.is_empty() || content.starts_with([' ', '\t']) {
            Doc::text(text.to_string())
        } else {
            Doc::text(format!("# {content}"))
        }
    } else {
        Doc::text(text.to_string())
    }
}

fn format_quoted_scalar(s: &str, quotes_option: Option<&Quotes>) -> String {
    let s = s.trim();
    match quotes_option {
        Some(Quotes::PreferDouble) => s.replace("''", "'"),
        Some(Quotes::PreferSingle) => s.replace('\'', "''"),
        None => s.to_owned(),
    }
}

fn can_omit_question_mark(key: &SyntaxNode) -> bool {
    let parent = key.parent();
    // question mark can be omitted in flow map
    (parent
        .as_ref()
        .is_some_and(|parent| parent.kind() == SyntaxKind::FLOW_MAP_ENTRY)
        // or, if there's map value, it can be omitted;
        // otherwise, this can lead invalid or incorrect syntax
        || parent
            .iter()
            .flat_map(|parent| parent.children())
            .any(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::FLOW_MAP_VALUE | SyntaxKind::BLOCK_MAP_VALUE
                )
            }))
        // when there're comments, there must be line breaks, so don't omit
        && !key
            .children_with_tokens()
            .any(|element| element.kind() == SyntaxKind::COMMENT)
        // also check comments after key but before colon
        && key
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .take_while(|element| {
                matches!(element.kind(), SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
            })
            .all(|element| element.kind() != SyntaxKind::COMMENT)
        // when there're flow scalar with line breaks, don't omit
        && key
            .children()
            .find(|child| child.kind() == SyntaxKind::FLOW)
            .iter()
            .flat_map(|flow| flow.children_with_tokens())
            .any(|element| {
                if let SyntaxElement::Token(token) = element {
                    matches!(
                        token.kind(),
                        SyntaxKind::DOUBLE_QUOTED_SCALAR
                            | SyntaxKind::SINGLE_QUOTED_SCALAR
                            | SyntaxKind::PLAIN_SCALAR
                    ) && !token.text().contains(['\n', '\r'])
                } else {
                    element.kind() == SyntaxKind::ALIAS
                }
            })
}

fn intersperse_lines(docs: &mut Vec<Doc<'static>>, mut lines: impl Iterator<Item = String>) {
    if let Some(line) = lines.next() {
        docs.push(Doc::text(line));
    }
    for line in lines {
        if line.is_empty() {
            docs.push(Doc::empty_line());
        } else {
            docs.push(Doc::hard_line());
            docs.push(Doc::text(line));
        }
    }
}

fn reflow(text: &str) -> impl Iterator<Item = Doc<'static>> + '_ {
    itertools::intersperse(
        text.lines().map(|s| Doc::text(s.to_owned())),
        Doc::empty_line(),
    )
}

fn should_ignore(node: &SyntaxNode, ctx: &Ctx) -> bool {
    // for the case that comment comes in the middle of a list of nodes
    node.prev_sibling_or_token()
        .and_then(|element| element.prev_sibling_or_token())
        .or_else(|| {
            // for the case that comment comes at the start or the end of a list of nodes
            node.parent()
                .and_then(|parent| parent.prev_sibling_or_token())
                .and_then(|parent| parent.prev_sibling_or_token())
        })
        .as_ref()
        .and_then(|element| match element {
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::COMMENT => {
                token.text().strip_prefix('#').and_then(|s| {
                    s.trim_start()
                        .strip_prefix(&ctx.options.ignore_comment_directive)
                })
            }
            _ => None,
        })
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(|c: char| c.is_ascii_whitespace()))
}
