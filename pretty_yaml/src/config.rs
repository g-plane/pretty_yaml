//! Types about configuration.
//!
//! For detailed documentation of configuration,
//! please read [configuration documentation](https://pretty-yaml.netlify.app/).

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// The whole configuration of Pretty YAML.
pub struct FormatOptions {
    #[cfg_attr(feature = "config_serde", serde(flatten))]
    pub layout: LayoutOptions,
    #[cfg_attr(feature = "config_serde", serde(flatten))]
    pub language: LanguageOptions,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// Configuration related to layout, such as indentation or print width.
pub struct LayoutOptions {
    #[cfg_attr(feature = "config_serde", serde(alias = "printWidth"))]
    pub print_width: usize,

    #[cfg_attr(feature = "config_serde", serde(alias = "indentWidth"))]
    pub indent_width: usize,

    #[cfg_attr(
        feature = "config_serde",
        serde(alias = "lineBreak", alias = "linebreak")
    )]
    pub line_break: LineBreak,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            print_width: 80,
            indent_width: 2,
            line_break: LineBreak::Lf,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum LineBreak {
    #[default]
    Lf,
    Crlf,
}

impl From<LineBreak> for tiny_pretty::LineBreak {
    fn from(value: LineBreak) -> Self {
        match value {
            LineBreak::Lf => tiny_pretty::LineBreak::Lf,
            LineBreak::Crlf => tiny_pretty::LineBreak::Crlf,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// Configuration related to syntax.
pub struct LanguageOptions {
    pub quotes: Quotes,

    #[cfg_attr(feature = "config_serde", serde(alias = "trailingComma"))]
    pub trailing_comma: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "formatComments"))]
    pub format_comments: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "indentBlockSequenceInMap"))]
    pub indent_block_sequence_in_map: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "braceSpacing"))]
    pub brace_spacing: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "bracketSpacing"))]
    pub bracket_spacing: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "dashSpacing"))]
    pub dash_spacing: DashSpacing,

    #[cfg_attr(feature = "config_serde", serde(alias = "preferSingleLine"))]
    pub prefer_single_line: bool,
    #[cfg_attr(
        feature = "config_serde",
        serde(
            rename = "flow_sequence.prefer_single_line",
            alias = "flowSequence.preferSingleLine"
        )
    )]
    pub flow_sequence_prefer_single_line: Option<bool>,
    #[cfg_attr(
        feature = "config_serde",
        serde(
            rename = "flow_map.prefer_single_line",
            alias = "flowMap.preferSingleLine"
        )
    )]
    pub flow_map_prefer_single_line: Option<bool>,

    #[cfg_attr(feature = "config_serde", serde(alias = "trimTrailingWhitespaces"))]
    pub trim_trailing_whitespaces: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "trimTrailingZero"))]
    pub trim_trailing_zero: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "ignoreCommentDirective"))]
    pub ignore_comment_directive: String,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        LanguageOptions {
            quotes: Quotes::default(),
            trailing_comma: true,
            format_comments: false,
            indent_block_sequence_in_map: true,
            brace_spacing: true,
            bracket_spacing: false,
            dash_spacing: DashSpacing::default(),
            prefer_single_line: false,
            flow_sequence_prefer_single_line: None,
            flow_map_prefer_single_line: None,
            trim_trailing_whitespaces: true,
            trim_trailing_zero: false,
            ignore_comment_directive: "pretty-yaml-ignore".into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum Quotes {
    #[default]
    #[cfg_attr(feature = "config_serde", serde(alias = "preferDouble"))]
    /// Make string to double quoted unless it contains single quotes inside.
    PreferDouble,

    #[cfg_attr(feature = "config_serde", serde(alias = "preferSingle"))]
    /// Make string to single quoted unless it contains double quotes inside.
    PreferSingle,

    #[cfg_attr(feature = "config_serde", serde(alias = "forceDouble"))]
    ForceDouble,

    #[cfg_attr(feature = "config_serde", serde(alias = "forceSingle"))]
    ForceSingle,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(rename_all = "kebab-case"))]
pub enum DashSpacing {
    #[default]
    #[cfg_attr(feature = "config_serde", serde(alias = "oneSpace"))]
    OneSpace,
    Indent,
}
