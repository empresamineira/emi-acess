//! EMI Acess customizations, kept in one module to simplify upstream merges.
//!
//! `APP_NAME` stays "RustDesk" because it also names the service, pipes, config
//! directory and install path, and it does not allow spaces.

use base::config::keys;
use hbb_common::config;
use std::collections::HashMap;

pub const DISPLAY_NAME: &str = "EMI Acess";
#[cfg(windows)]
pub const PUBLISHER: &str = "Empresa Mineira de Informatica";
// `is_custom_client()` is false with the stock `APP_NAME`, so without this the client
// would offer official RustDesk releases, which would replace this build.
pub const SOFTWARE_UPDATE_DISABLED: bool = true;

fn server_defaults(server: Option<&str>, key: Option<&str>) -> HashMap<String, String> {
    [
        (keys::OPTION_CUSTOM_RENDEZVOUS_SERVER, server),
        (keys::OPTION_KEY, key),
    ]
    .into_iter()
    .filter_map(|(option, value)| {
        let value = value?.trim();
        (!value.is_empty()).then(|| (option.to_owned(), value.to_owned()))
    })
    .collect()
}

/// Upstream dropped `option_env!` from `hbb_common`, so the build values are applied here.
/// `DEFAULT_SETTINGS` rather than `OVERWRITE_SETTINGS`, so the server can still be changed
/// after unlocking the settings.
pub fn apply_build_defaults() {
    let defaults = server_defaults(option_env!("RENDEZVOUS_SERVER"), option_env!("RS_PUB_KEY"));
    config::DEFAULT_SETTINGS.write().unwrap().extend(defaults);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_values_become_server_defaults() {
        let defaults = server_defaults(Some("hbbs.example.com"), Some("abc="));
        assert_eq!(
            defaults.get(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER).map(String::as_str),
            Some("hbbs.example.com")
        );
        assert_eq!(defaults.get(keys::OPTION_KEY).map(String::as_str), Some("abc="));
    }

    #[test]
    fn missing_build_values_keep_stock_behavior() {
        assert!(server_defaults(None, Some("")).is_empty());
    }
}
