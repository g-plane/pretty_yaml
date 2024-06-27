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

fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|it| it.into_token())
        .find(|it| it.kind() == kind)
}

// -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Syntax for `&anchor` and/or `!!tag`.
pub struct Properties {
    syntax: SyntaxNode,
}
impl Properties {
    pub fn anchor_property(&self) -> Option<AnchorProperty> {
        child(&self.syntax)
    }
    pub fn tag_property(&self) -> Option<TagProperty> {
        child(&self.syntax)
    }
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
/// Syntax for `!<...>`, `!!xxx`, or `!`.
pub struct TagProperty {
    syntax: SyntaxNode,
}
impl TagProperty {
    pub fn verbatim_tag(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::VERBATIM_TAG)
    }
    pub fn shorthand_tag(&self) -> Option<ShorthandTag> {
        child(&self.syntax)
    }
    pub fn non_specific_tag(&self) -> Option<NonSpecificTag> {
        child(&self.syntax)
    }
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
/// Syntax for `!xxx`, `!!`, or `!`.
pub struct TagHandle {
    syntax: SyntaxNode,
}
impl TagHandle {
    pub fn primary(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::TAG_HANDLE_PRIMARY)
    }
    pub fn secondary(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::TAG_HANDLE_SECONDARY)
    }
    pub fn named(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::TAG_HANDLE_NAMED)
    }
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
/// Syntax for `!!xxx`.
pub struct ShorthandTag {
    syntax: SyntaxNode,
}
impl ShorthandTag {
    pub fn tag_handle(&self) -> Option<TagHandle> {
        child(&self.syntax)
    }
    pub fn tag_char(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::TAG_CHAR)
    }
}
impl AstNode for ShorthandTag {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::SHORTHAND_TAG
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(ShorthandTag { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Syntax for `!`.
pub struct NonSpecificTag {
    syntax: SyntaxNode,
}
impl NonSpecificTag {
    pub fn exclamation_mark(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::EXCLAMATION_MARK)
    }
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
/// Syntax for `&anchor`.
pub struct AnchorProperty {
    syntax: SyntaxNode,
}
impl AnchorProperty {
    pub fn ampersand(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::AMPERSAND)
    }
    pub fn anchor_name(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::ANCHOR_NAME)
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
/// Syntax for `*anchor`.
pub struct Alias {
    syntax: SyntaxNode,
}
impl Alias {
    pub fn asterisk(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::ASTERISK)
    }
    pub fn anchor_name(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::ANCHOR_NAME)
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
/// Syntax for `[1, 2]`.
pub struct FlowSeq {
    syntax: SyntaxNode,
}
impl FlowSeq {
    pub fn l_bracket(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::L_BRACKET)
    }
    pub fn entries(&self) -> Option<FlowSeqEntries> {
        child(&self.syntax)
    }
    pub fn r_bracket(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::R_BRACKET)
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
/// Syntax for `1, 2` in `[1, 2]` (without brackets).
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
/// Syntax for each item in `[1, 2]` (without comma).
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
/// Syntax for `{a: 1, b: 2}`.
pub struct FlowMap {
    syntax: SyntaxNode,
}
impl FlowMap {
    pub fn l_brace(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::L_BRACE)
    }
    pub fn entries(&self) -> Option<FlowMapEntries> {
        child(&self.syntax)
    }
    pub fn r_brace(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::R_BRACE)
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
/// Syntax for `a: 1, b: 2` in `{a: 1, b: 2}` (without braces).
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
/// Syntax for each item (like `a: 1`) in `{a: 1, b: 2}` (without comma).
pub struct FlowMapEntry {
    syntax: SyntaxNode,
}
impl FlowMapEntry {
    pub fn key(&self) -> Option<FlowMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::COLON)
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
/// Syntax for `a` or `b` in `{a: 1, b: 2}`.
pub struct FlowMapKey {
    syntax: SyntaxNode,
}
impl FlowMapKey {
    pub fn question_mark(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::QUESTION_MARK)
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
/// Syntax for `1` or `2` in `{a: 1, b: 2}`.
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
/// Syntax for `a: 1` in `[a: 1]`.
pub struct FlowPair {
    syntax: SyntaxNode,
}
impl FlowPair {
    pub fn key(&self) -> Option<FlowMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::COLON)
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
/// Syntax for `""`, `''`, `plain`, `[]` or `{}`.
pub struct Flow {
    syntax: SyntaxNode,
}
impl Flow {
    pub fn properties(&self) -> Option<Properties> {
        child(&self.syntax)
    }
    pub fn double_qouted_scalar(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DOUBLE_QUOTED_SCALAR)
    }
    pub fn single_quoted_scalar(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::SINGLE_QUOTED_SCALAR)
    }
    pub fn plain_scalar(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::PLAIN_SCALAR)
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
/// Syntax for `+` or `-` in block scalar.
/// ```yaml
/// |+
///   ...
/// >-
///   ...
/// ```
pub struct ChompingIndicator {
    syntax: SyntaxNode,
}
impl ChompingIndicator {
    pub fn plus(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::PLUS)
    }
    pub fn minus(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::MINUS)
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
/// Syntax for multi-line text that starts with `|` or `>`.
/// ```yaml
/// |+
///   ...
/// >-
///   ...
/// ```
pub struct BlockScalar {
    syntax: SyntaxNode,
}
impl BlockScalar {
    pub fn bar(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::BAR)
    }
    pub fn greater_than(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::GREATER_THAN)
    }
    pub fn indent_indicator(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::INDENT_INDICATOR)
    }
    pub fn chomping_indicator(&self) -> Option<ChompingIndicator> {
        child(&self.syntax)
    }
    pub fn text(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::BLOCK_SCALAR_TEXT)
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
/// Syntax for sequence that contains one or more `- item`.
/// ```yaml
/// - item1
/// - item2
/// ```
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
/// Syntax for each item like `- item1` in block sequence.
pub struct BlockSeqEntry {
    syntax: SyntaxNode,
}
impl BlockSeqEntry {
    pub fn minus(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::MINUS)
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
/// Syntax for key-value pairs object.
/// ```yaml
/// key1: value1
/// key2: value2
/// ```
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
/// Syntax for each key-value pair like `key1: value1` in block map.
pub struct BlockMapEntry {
    syntax: SyntaxNode,
}
impl BlockMapEntry {
    pub fn key(&self) -> Option<BlockMapKey> {
        child(&self.syntax)
    }
    pub fn colon(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::COLON)
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
/// Syntax for `key1` in `key1: value1`.
pub struct BlockMapKey {
    syntax: SyntaxNode,
}
impl BlockMapKey {
    pub fn question_mark(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::QUESTION_MARK)
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
/// Syntax for `value1` in `key1: value1`.
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
/// Syntax for block scalar, block sequence or block map.
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
/// Syntax for `%YAML 1.2`.
pub struct YamlDirective {
    syntax: SyntaxNode,
}
impl YamlDirective {
    pub fn directive_name(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DIRECTIVE_NAME)
    }
    pub fn yaml_version(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::YAML_VERSION)
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
/// Syntax for `%TAG ! tag:yaml.org,2002:`.
pub struct TagDirective {
    syntax: SyntaxNode,
}
impl TagDirective {
    pub fn directive_name(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DIRECTIVE_NAME)
    }
    pub fn tag_handle(&self) -> Option<TagHandle> {
        child(&self.syntax)
    }
    pub fn tag_prefix(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::TAG_PREFIX)
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
/// Syntax for `%unknown ...`.
pub struct ReservedDirective {
    syntax: SyntaxNode,
}
impl ReservedDirective {
    pub fn directive_name(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DIRECTIVE_NAME)
    }
    pub fn directive_param(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DIRECTIVE_PARAM)
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
/// Syntax for `%YAML 1.2`, `%TAG ! tag:yaml.org,2002:`, or `%unknown ...`.
pub struct Directive {
    syntax: SyntaxNode,
}
impl Directive {
    pub fn percent(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::PERCENT)
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
/// Syntax for a whole document which can contain directives, block/flow.
pub struct Document {
    syntax: SyntaxNode,
}
impl Document {
    pub fn directives(&self) -> AstChildren<Directive> {
        children(&self.syntax)
    }
    pub fn directives_end(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DIRECTIVES_END)
    }
    pub fn block(&self) -> Option<Block> {
        child(&self.syntax)
    }
    pub fn flow(&self) -> Option<Flow> {
        child(&self.syntax)
    }
    pub fn document_end(&self) -> Option<SyntaxToken> {
        token(&self.syntax, SyntaxKind::DOCUMENT_END)
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
/// Root contains zero or more documents.
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
