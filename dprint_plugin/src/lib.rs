use crate::config::resolve_config;
use anyhow::Result;
#[cfg(target_arch = "wasm32")]
use dprint_core::generate_plugin_code;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult},
    plugins::{FileMatchingInfo, PluginInfo, SyncPluginHandler, SyncPluginInfo},
};
use pretty_yaml::{config::FormatOptions, format_text};
use std::path::Path;

mod config;

#[cfg(target_arch = "wasm32")]
type Configuration = FormatOptions;

pub struct PrettyYamlPluginHandler;

impl SyncPluginHandler<FormatOptions> for PrettyYamlPluginHandler {
    fn plugin_info(&mut self) -> SyncPluginInfo {
        let version = env!("CARGO_PKG_VERSION").to_string();
        SyncPluginInfo {
            info: PluginInfo {
                name: env!("CARGO_PKG_NAME").into(),
                version: version.clone(),
                config_key: "yaml".into(),
                help_url: "https://github.com/g-plane/pretty_yaml".into(),
                config_schema_url: format!(
                    "https://plugins.dprint.dev/g-plane/pretty_yaml/v{}/schema.json",
                    version
                ),
                update_url: Some(
                    "https://plugins.dprint.dev/g-plane/pretty_yaml/latest.json".into(),
                ),
            },
            file_matching: FileMatchingInfo {
                file_extensions: ["yaml", "yml"].into_iter().map(String::from).collect(),
                file_names: vec![],
            },
        }
    }

    fn license_text(&mut self) -> String {
        include_str!("../../LICENSE").into()
    }

    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        global_config: &GlobalConfiguration,
    ) -> ResolveConfigurationResult<FormatOptions> {
        resolve_config(config, global_config)
    }

    fn format(
        &mut self,
        _: &Path,
        file_text: Vec<u8>,
        config: &FormatOptions,
        _: impl FnMut(&Path, Vec<u8>, &ConfigKeyMap) -> Result<Option<Vec<u8>>>,
    ) -> Result<Option<Vec<u8>>> {
        let format_result = format_text(std::str::from_utf8(&file_text)?, config);
        match format_result {
            Ok(code) => Ok(Some(code.into_bytes())),
            Err(err) => Err(err.into()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
generate_plugin_code!(PrettyYamlPluginHandler, PrettyYamlPluginHandler);
