use crate::config::resolve_config;
use anyhow::Result;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration},
    plugins::{
        CheckConfigUpdatesMessage, ConfigChange, FormatError, FormatResult, PluginInfo,
        PluginResolveConfigurationResult, SyncFormatRequest, SyncHostFormatRequest,
        SyncPluginHandler,
    },
};
use pretty_yaml::{config::FormatOptions, format_text};

mod config;

pub struct PrettyYamlPluginHandler;

impl SyncPluginHandler<FormatOptions> for PrettyYamlPluginHandler {
    fn plugin_info(&mut self) -> PluginInfo {
        let version = env!("CARGO_PKG_VERSION").to_string();
        PluginInfo {
            name: env!("CARGO_PKG_NAME").into(),
            version: version.clone(),
            config_key: "yaml".into(),
            help_url: "https://github.com/g-plane/pretty_yaml".into(),
            config_schema_url: format!(
                "https://plugins.dprint.dev/g-plane/pretty_yaml/v{}/schema.json",
                version
            ),
            update_url: Some("https://plugins.dprint.dev/g-plane/pretty_yaml/latest.json".into()),
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").into()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<FormatOptions> {
        resolve_config(config, global_config)
    }

    fn check_config_updates(
        &self,
        _: CheckConfigUpdatesMessage,
    ) -> Result<Vec<ConfigChange>, FormatError> {
        Ok(Vec::new())
    }

    fn format(
        &mut self,
        request: SyncFormatRequest<FormatOptions>,
        _: impl FnMut(SyncHostFormatRequest) -> FormatResult,
    ) -> FormatResult {
        let format_result = format_text(std::str::from_utf8(&request.file_bytes)?, request.config);
        match format_result {
            Ok(code) => Ok(Some(code.into_bytes())),
            Err(err) => Err(FormatError::new(err)),
        }
    }
}

#[cfg(target_arch = "wasm32")]
dprint_core::generate_plugin_code!(
    PrettyYamlPluginHandler,
    PrettyYamlPluginHandler,
    FormatOptions
);
