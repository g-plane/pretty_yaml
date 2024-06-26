//! Abstract Syntax Tree, layered on top of untyped `SyntaxNode`s.

use super::{SyntaxKind, SyntaxNode, SyntaxToken, YamlLanguage};
use rowan::SyntaxNodeChildren;
use std::marker::PhantomData;

// --------------- Code below are copied from rust-analyzer ----------------

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_subtree()).unwrap()
    }
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren<YamlLanguage>,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
    AstChildren::new(parent)
}

fn token<T: AstToken>(parent: &SyntaxNode) -> Option<T> {
    parent
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find_map(T::cast)
}

fn token_with_kind(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == kind)
}

// -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndentIndicator {
    syntax: SyntaxToken,
}
impl AstToken for IndentIndicator {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::INDENT_INDICATOR
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(IndentIndicator { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GreaterThan {
    syntax: SyntaxToken,
}
impl AstToken for GreaterThan {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::GREATER_THAN
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(GreaterThan { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerbatimTag {
    syntax: SyntaxToken,
}
impl AstToken for VerbatimTag {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::VERBATIM_TAG
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(VerbatimTag { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShorthandTag {
    syntax: SyntaxToken,
}
impl AstToken for ShorthandTag {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::SHORTHAND_TAG
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(ShorthandTag { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagChar {
    syntax: SyntaxToken,
}
impl AstToken for TagChar {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::TAG_CHAR
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagChar { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagHandleNamed {
    syntax: SyntaxToken,
}
impl AstToken for TagHandleNamed {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::TAG_HANDLE_NAMED
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagHandleNamed { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagHandleSecondary {
    syntax: SyntaxToken,
}
impl AstToken for TagHandleSecondary {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::TAG_HANDLE_SECONDARY
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagHandleSecondary { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagHandlePrimary {
    syntax: SyntaxToken,
}
impl AstToken for TagHandlePrimary {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::TAG_HANDLE_PRIMARY
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagHandlePrimary { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagPrefix {
    syntax: SyntaxToken,
}
impl AstToken for TagPrefix {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::TAG_PREFIX
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagPrefix { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnchorName {
    syntax: SyntaxToken,
}
impl AstToken for AnchorName {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::ANCHOR_NAME
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(AnchorName { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoubleQuotedScalar {
    syntax: SyntaxToken,
}
impl AstToken for DoubleQuotedScalar {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::DOUBLE_QUOTED_SCALAR
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(DoubleQuotedScalar { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SingleQuotedScalar {
    syntax: SyntaxToken,
}
impl AstToken for SingleQuotedScalar {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::SINGLE_QUOTED_SCALAR
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(SingleQuotedScalar { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlainScalar {
    syntax: SyntaxToken,
}
impl AstToken for PlainScalar {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::PLAIN_SCALAR
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(PlainScalar { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockScalarText {
    syntax: SyntaxToken,
}
impl AstToken for BlockScalarText {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::BLOCK_SCALAR_TEXT
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockScalarText { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectivesEnd {
    syntax: SyntaxToken,
}
impl AstToken for DirectivesEnd {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::DIRECTIVES_END
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(DirectivesEnd { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveName {
    syntax: SyntaxToken,
}
impl AstToken for DirectiveName {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::DIRECTIVE_NAME
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(DirectiveName { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YamlVersion {
    syntax: SyntaxToken,
}
impl AstToken for YamlVersion {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::YAML_VERSION
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(YamlVersion { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveParam {
    syntax: SyntaxToken,
}
impl AstToken for DirectiveParam {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::DIRECTIVE_PARAM
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(DirectiveParam { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentEnd {
    syntax: SyntaxToken,
}
impl AstToken for DocumentEnd {
    fn can_cast(token: SyntaxKind) -> bool {
        token == SyntaxKind::DOCUMENT_END
    }
    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(DocumentEnd { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Properties {
    syntax: SyntaxNode,
}
impl AstNode for Properties {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PROPERTIES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Properties { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagProperty {
    syntax: SyntaxNode,
}
impl AstNode for TagProperty {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TAG_PROPERTY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagProperty { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagHandle {
    syntax: SyntaxNode,
}
impl AstNode for TagHandle {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TAG_HANDLE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagHandle { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonSpecificTag {
    syntax: SyntaxNode,
}
impl AstNode for NonSpecificTag {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::NON_SPECIFIC_TAG
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(NonSpecificTag { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnchorProperty {
    syntax: SyntaxNode,
}
impl AnchorProperty {
    pub fn ampersand(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::AMPERSAND)
    }
    pub fn anchor_name(&self) -> Option<AnchorName> {
        token(&self.syntax)
    }
}
impl AstNode for AnchorProperty {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ANCHOR_PROPERTY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(AnchorProperty { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    syntax: SyntaxNode,
}
impl Alias {
    pub fn asterisk(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::ASTERISK)
    }
    pub fn anchor_name(&self) -> Option<AnchorName> {
        token(&self.syntax)
    }
}
impl AstNode for Alias {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ALIAS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Alias { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowSeq {
    syntax: SyntaxNode,
}
impl FlowSeq {
    pub fn l_bracket(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::L_BRACKET)
    }
    pub fn entries(&self) -> Option<FlowSeqEntries> {
        child(&self.syntax)
    }
    pub fn r_bracket(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::R_BRACKET)
    }
}
impl AstNode for FlowSeq {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_SEQ
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowSeq { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowSeqEntries {
    syntax: SyntaxNode,
}
impl FlowSeqEntries {
    pub fn entries(&self) -> AstChildren<FlowSeqEntry> {
        children(&self.syntax)
    }
}
impl AstNode for FlowSeqEntries {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_SEQ_ENTRIES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowSeqEntries { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowSeqEntry {
    syntax: SyntaxNode,
}
impl FlowSeqEntry {
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
    pub fn flow_pair(&self) -> Option<FlowPair> {
        child(&self.syntax)
    }
}
impl AstNode for FlowSeqEntry {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_SEQ_ENTRY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowSeqEntry { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowMap {
    syntax: SyntaxNode,
}
impl FlowMap {
    pub fn l_brace(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::L_BRACE)
    }
    pub fn entries(&self) -> Option<FlowMapEntries> {
        child(&self.syntax)
    }
    pub fn r_brace(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::R_BRACE)
    }
}
impl AstNode for FlowMap {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_MAP
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowMap { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowMapEntries {
    syntax: SyntaxNode,
}
impl FlowMapEntries {
    pub fn entries(&self) -> AstChildren<FlowMapEntry> {
        children(&self.syntax)
    }
}
impl AstNode for FlowMapEntries {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_MAP_ENTRIES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowMapEntries { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowMapEntry {
    syntax: SyntaxNode,
}
impl FlowMapEntry {
    pub fn key(&self) -> Option<FlowMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::COLON)
    }
    pub fn value(&self) -> Option<FlowMapValue> {
        child(&self.syntax)
    }
}
impl AstNode for FlowMapEntry {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_MAP_ENTRY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowMapEntry { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowMapKey {
    syntax: SyntaxNode,
}
impl FlowMapKey {
    pub fn question_mark(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::QUESTION_MARK)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
}
impl AstNode for FlowMapKey {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_MAP_KEY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowMapKey { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowMapValue {
    syntax: SyntaxNode,
}
impl FlowMapValue {
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
}
impl AstNode for FlowMapValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_MAP_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowMapValue { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPair {
    syntax: SyntaxNode,
}
impl FlowPair {
    pub fn key(&self) -> Option<FlowMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::COLON)
    }
    pub fn value(&self) -> Option<FlowMapValue> {
        child(&self.syntax)
    }
}
impl AstNode for FlowPair {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW_PAIR
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(FlowPair { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Flow {
    syntax: SyntaxNode,
}
impl Flow {
    pub fn properties(&self) -> Option<Properties> {
        child(&self.syntax)
    }
    pub fn double_qouted_scalar(&self) -> Option<DoubleQuotedScalar> {
        token(&self.syntax)
    }
    pub fn single_quoted_scalar(&self) -> Option<SingleQuotedScalar> {
        token(&self.syntax)
    }
    pub fn plain_scalar(&self) -> Option<PlainScalar> {
        token(&self.syntax)
    }
    pub fn flow_seq(&self) -> Option<FlowSeq> {
        child(&self.syntax)
    }
    pub fn flow_map(&self) -> Option<FlowMap> {
        child(&self.syntax)
    }
}
impl AstNode for Flow {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::FLOW
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Flow { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChompingIndicator {
    syntax: SyntaxNode,
}
impl ChompingIndicator {
    pub fn plus(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::PLUS)
    }
    pub fn minus(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::MINUS)
    }
}
impl AstNode for ChompingIndicator {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::CHOMPING_INDICATOR
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(ChompingIndicator { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockScalar {
    syntax: SyntaxNode,
}
impl BlockScalar {
    pub fn bar(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::BAR)
    }
    pub fn greater_than(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::GREATER_THAN)
    }
    pub fn indent_indicator(&self) -> Option<IndentIndicator> {
        token(&self.syntax)
    }
    pub fn chomping_indicator(&self) -> Option<ChompingIndicator> {
        child(&self.syntax)
    }
    pub fn text(&self) -> Option<BlockScalarText> {
        token(&self.syntax)
    }
}
impl AstNode for BlockScalar {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_SCALAR
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockScalar { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockSeq {
    syntax: SyntaxNode,
}
impl BlockSeq {
    pub fn entries(&self) -> AstChildren<BlockSeqEntry> {
        children(&self.syntax)
    }
}
impl AstNode for BlockSeq {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_SEQ
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockSeq { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockSeqEntry {
    syntax: SyntaxNode,
}
impl BlockSeqEntry {
    pub fn minus(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::MINUS)
    }
    pub fn block(&self) -> Option<Block> {
        child(&self.syntax)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
}
impl AstNode for BlockSeqEntry {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_SEQ_ENTRY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockSeqEntry { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockMap {
    syntax: SyntaxNode,
}
impl BlockMap {
    pub fn entries(&self) -> AstChildren<BlockMapEntry> {
        children(&self.syntax)
    }
}
impl AstNode for BlockMap {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_MAP
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockMap { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockMapEntry {
    syntax: SyntaxNode,
}
impl BlockMapEntry {
    pub fn key(&self) -> Option<BlockMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::COLON)
    }
    pub fn value(&self) -> Option<BlockMapValue> {
        child(&self.syntax)
    }
}
impl AstNode for BlockMapEntry {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_MAP_ENTRY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockMapEntry { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockMapKey {
    syntax: SyntaxNode,
}
impl BlockMapKey {
    pub fn question_mark(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::QUESTION_MARK)
    }
    pub fn block(&self) -> Option<Block> {
        child(&self.syntax)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
}
impl AstNode for BlockMapKey {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_MAP_KEY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockMapKey { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockMapValue {
    syntax: SyntaxNode,
}
impl BlockMapValue {
    pub fn block(&self) -> Option<Block> {
        child(&self.syntax)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
}
impl AstNode for BlockMapValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK_MAP_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(BlockMapValue { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    syntax: SyntaxNode,
}
impl Block {
    pub fn properties(&self) -> Option<Properties> {
        child(&self.syntax)
    }
    pub fn block_scalar(&self) -> Option<BlockScalar> {
        child(&self.syntax)
    }
    pub fn block_seq(&self) -> Option<BlockSeq> {
        child(&self.syntax)
    }
    pub fn block_map(&self) -> Option<BlockMap> {
        child(&self.syntax)
    }
}
impl AstNode for Block {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::BLOCK
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Block { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YamlDirective {
    syntax: SyntaxNode,
}
impl YamlDirective {
    pub fn directive_name(&self) -> Option<DirectiveName> {
        token(&self.syntax)
    }
    pub fn yaml_version(&self) -> Option<YamlVersion> {
        token(&self.syntax)
    }
}
impl AstNode for YamlDirective {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::YAML_DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(YamlDirective { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagDirective {
    syntax: SyntaxNode,
}
impl TagDirective {
    pub fn directive_name(&self) -> Option<DirectiveName> {
        token(&self.syntax)
    }
    pub fn tag_handle(&self) -> Option<TagHandle> {
        child(&self.syntax)
    }
    pub fn tag_prefix(&self) -> Option<TagPrefix> {
        token(&self.syntax)
    }
}
impl AstNode for TagDirective {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::TAG_DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(TagDirective { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReservedDirective {
    syntax: SyntaxNode,
}
impl ReservedDirective {
    pub fn directive_name(&self) -> Option<DirectiveName> {
        token(&self.syntax)
    }
    pub fn directive_param(&self) -> Option<DirectiveParam> {
        token(&self.syntax)
    }
}
impl AstNode for ReservedDirective {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::RESERVED_DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(ReservedDirective { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Directive {
    syntax: SyntaxNode,
}
impl Directive {
    pub fn percent(&self) -> Option<SyntaxToken> {
        token_with_kind(&self.syntax, SyntaxKind::PERCENT)
    }
    pub fn yaml_directive(&self) -> Option<YamlDirective> {
        child(&self.syntax)
    }
    pub fn tag_directive(&self) -> Option<TagDirective> {
        child(&self.syntax)
    }
    pub fn reserved_directive(&self) -> Option<ReservedDirective> {
        child(&self.syntax)
    }
}
impl AstNode for Directive {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Directive { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    syntax: SyntaxNode,
}
impl Document {
    pub fn directives(&self) -> AstChildren<Directive> {
        children(&self.syntax)
    }
    pub fn directives_end(&self) -> Option<DirectivesEnd> {
        token(&self.syntax)
    }
    pub fn block(&self) -> Option<Block> {
        child(&self.syntax)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
    pub fn document_end(&self) -> Option<DocumentEnd> {
        token(&self.syntax)
    }
}
impl AstNode for Document {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::DOCUMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Document { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Root {
    syntax: SyntaxNode,
}
impl Root {
    pub fn documents(&self) -> AstChildren<Document> {
        children(&self.syntax)
    }
}
impl AstNode for Root {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Root { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
