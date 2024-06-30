//! Types about configuration.

#[cfg(feature = "config_serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "config_serde", serde(default))]
/// The whole configuration of Pretty YAML.
///
/// For detail, please refer to [Configuration](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md) on GitHub.
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
    /// See [`printWidth`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#printwidth) on GitHub
    pub print_width: usize,

    #[cfg_attr(feature = "config_serde", serde(alias = "useTabs"))]
    /// See [`useTabs`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#usetabs) on GitHub
    pub use_tabs: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "indentWidth"))]
    /// See [`indentWidth`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#indentwidth) on GitHub
    pub indent_width: usize,

    #[cfg_attr(
        feature = "config_serde",
        serde(alias = "lineBreak", alias = "linebreak")
    )]
    /// See [`lineBreak`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#linebreak) on GitHub
    pub line_break: LineBreak,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            print_width: 80,
            use_tabs: false,
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
    /// See [`quotes`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#quotes) on GitHub
    pub quotes: Quotes,

    #[cfg_attr(feature = "config_serde", serde(alias = "trailingComma"))]
    /// See [`trailingComma`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#trailingcomma) on GitHub
    pub trailing_comma: bool,

    #[cfg_attr(feature = "config_serde", serde(alias = "formatComments"))]
    /// See [`formatComments`](https://github.com/g-plane/pretty_yaml/blob/main/docs/config.md#formatcomments) on GitHub
    pub format_comments: bool,
}

impl Default for LanguageOptions {
    fn default() -> Self {
        LanguageOptions {
            quotes: Quotes::default(),
            trailing_comma: true,
            format_comments: false,
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
}