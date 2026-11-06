//! EMI Acess customizations, kept in one module to simplify upstream merges.
//!
//! `APP_NAME` stays "RustDesk" because it also names the service, pipes, config
//! directory and install path, and it does not allow spaces.

pub const DISPLAY_NAME: &str = "EMI Acess";
#[cfg(windows)]
pub const PUBLISHER: &str = "Empresa Mineira de Informatica";
// `is_custom_client()` is false with the stock `APP_NAME`, so without this the client
// would offer official RustDesk releases, which would replace this build.
pub const SOFTWARE_UPDATE_DISABLED: bool = true;
