use dprint_core::configuration::{
    get_unknown_property_diagnostics, get_value, ConfigKeyMap, ConfigurationDiagnostic,
    GlobalConfiguration, NewLineKind, ResolveConfigurationResult,
};
use pretty_yaml::config::*;

pub(crate) fn resolve_config(
    mut config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<FormatOptions> {
    let mut diagnostics = Vec::new();
    let pretty_yaml_config = FormatOptions {
        layout: LayoutOptions {
            print_width: get_value(
                &mut config,
                "printWidth",
                global_config.line_width.unwrap_or(80),
                &mut diagnostics,
            ) as usize,
            indent_width: get_value(
                &mut config,
                "indentWidth",
                global_config.indent_width.unwrap_or(2),
                &mut diagnostics,
            ) as usize,
            line_break: match &*get_value(
                &mut config,
                "lineBreak",
                match global_config.new_line_kind {
                    Some(NewLineKind::LineFeed) => "lf",
                    Some(NewLineKind::CarriageReturnLineFeed) => "crlf",
                    _ => "lf",
                }
                .to_string(),
                &mut diagnostics,
            ) {
                "lf" => LineBreak::Lf,
                "crlf" => LineBreak::Crlf,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "lineBreak".into(),
                        message: "invalid value for config `lineBreak`".into(),
                    });
                    LineBreak::Lf
                }
            },
        },
        language: LanguageOptions {
            quotes: match &*get_value(
                &mut config,
                "quotes",
                "preferDouble".to_string(),
                &mut diagnostics,
            ) {
                "preferDouble" => Quotes::PreferDouble,
                "preferSingle" => Quotes::PreferSingle,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "quotes".into(),
                        message: "invalid value for config `quotes`".into(),
                    });
                    Default::default()
                }
            },
            trailing_comma: get_value(&mut config, "trailingComma", true, &mut diagnostics),
            format_comments: get_value(&mut config, "formatComments", false, &mut diagnostics),
            indent_block_sequence_in_map: get_value(
                &mut config,
                "indentBlockSequenceInMap",
                true,
                &mut diagnostics,
            ),
            brace_spacing: get_value(&mut config, "braceSpacing", true, &mut diagnostics),
            bracket_spacing: get_value(&mut config, "bracketSpacing", false, &mut diagnostics),
            dash_spacing: match &*get_value(
                &mut config,
                "dashSpacing",
                "oneSpace".to_string(),
                &mut diagnostics,
            ) {
                "oneSpace" => DashSpacing::OneSpace,
                "indent" => DashSpacing::Indent,
                _ => {
                    diagnostics.push(ConfigurationDiagnostic {
                        property_name: "dashSpacing".into(),
                        message: "invalid value for config `dashSpacing`".into(),
                    });
                    Default::default()
                }
            },
            trim_trailing_whitespaces: get_value(
                &mut config,
                "trimTrailingWhitespaces",
                true,
                &mut diagnostics,
            ),
            trim_trailing_zero: get_value(&mut config, "trimTrailingZero", false, &mut diagnostics),
            ignore_comment_directive: get_value(
                &mut config,
                "ignoreCommentDirective",
                "pretty-yaml-ignore".into(),
                &mut diagnostics,
            ),
        },
    };

    diagnostics.extend(get_unknown_property_diagnostics(config));

    ResolveConfigurationResult {
        config: pretty_yaml_config,
        diagnostics,
    }
}
