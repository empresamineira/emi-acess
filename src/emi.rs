//! EMI Acess customizations, kept in one module to simplify upstream merges.
//!
//! `APP_NAME` stays "RustDesk" because it also names the service, pipes, config
//! directory and install path, and it does not allow spaces.

use base::config::keys;
use hbb_common::{
    config::{self, LocalConfig},
    sha2::{Digest, Sha256},
};
use std::collections::HashMap;

pub const DISPLAY_NAME: &str = "EMI Acess";
#[cfg(windows)]
pub const PUBLISHER: &str = "Empresa Mineira de Informatica";
// `is_custom_client()` is false with the stock `APP_NAME`, so without this the client
// would offer official RustDesk releases, which would replace this build.
pub const SOFTWARE_UPDATE_DISABLED: bool = true;

const MAX_PASSWORD_ATTEMPTS: u32 = 5;
const PASSWORD_LOCKOUT_SECS: u64 = 300;
const PASSWORD_LOCKOUT_OPTION: &str = "emi-server-settings-lockout";

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PasswordLockout {
    failures: u32,
    locked_until: u64,
}

impl PasswordLockout {
    fn parse(stored: &str) -> Self {
        let (failures, locked_until) = stored.split_once(':').unwrap_or_default();
        Self {
            failures: failures.parse().unwrap_or_default(),
            locked_until: locked_until.parse().unwrap_or_default(),
        }
    }

    fn encode(&self) -> String {
        format!("{}:{}", self.failures, self.locked_until)
    }
}

#[derive(Debug, PartialEq)]
struct PasswordCheck {
    ok: bool,
    configured: bool,
    retry_after: u64,
}

fn denied(retry_after: u64) -> PasswordCheck {
    PasswordCheck {
        ok: false,
        configured: true,
        retry_after,
    }
}

fn is_valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn sha256_hex(password: &str) -> String {
    Sha256::digest(password.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn check_password(
    candidate: &str,
    expected_hash: Option<&str>,
    lockout: PasswordLockout,
    now: u64,
) -> (PasswordCheck, PasswordLockout) {
    let Some(expected_hash) = expected_hash.filter(|hash| is_valid_hash(hash)) else {
        let check = PasswordCheck {
            ok: false,
            configured: false,
            retry_after: 0,
        };
        return (check, lockout);
    };
    if lockout.locked_until > now {
        // Capped so a clock moved backwards never reports more than one lockout period.
        let retry_after = (lockout.locked_until - now).min(PASSWORD_LOCKOUT_SECS);
        return (denied(retry_after), lockout);
    }
    if constant_time_eq(sha256_hex(candidate).as_bytes(), expected_hash.as_bytes()) {
        let check = PasswordCheck {
            ok: true,
            configured: true,
            retry_after: 0,
        };
        return (check, PasswordLockout::default());
    }
    let failures = lockout.failures + 1;
    if failures >= MAX_PASSWORD_ATTEMPTS {
        let lockout = PasswordLockout {
            failures: 0,
            locked_until: now + PASSWORD_LOCKOUT_SECS,
        };
        return (denied(PASSWORD_LOCKOUT_SECS), lockout);
    }
    let lockout = PasswordLockout {
        failures,
        locked_until: 0,
    };
    (denied(0), lockout)
}

/// The hash stays on this side of the bridge, and only the attempt count is stored.
pub fn verify_server_settings_password(password: &str) -> String {
    let lockout = PasswordLockout::parse(&LocalConfig::get_option(PASSWORD_LOCKOUT_OPTION));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (check, next) = check_password(password, option_env!("EMI_SETTINGS_HASH"), lockout, now);
    if next != lockout {
        LocalConfig::set_option(PASSWORD_LOCKOUT_OPTION.to_owned(), next.encode());
    }
    serde_json::json!({
        "ok": check.ok,
        "configured": check.configured,
        "retry_after": check.retry_after,
    })
    .to_string()
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

    // NIST test vector for SHA-256("abc").
    const ABC_HASH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn only_the_matching_password_unlocks() {
        let lockout = PasswordLockout::default();
        assert!(check_password("abc", Some(ABC_HASH), lockout, 0).0.ok);
        assert!(!check_password("abd", Some(ABC_HASH), lockout, 0).0.ok);
    }

    #[test]
    fn missing_or_malformed_hash_stays_locked() {
        for hash in [None, Some(""), Some("abc"), Some(&ABC_HASH.to_uppercase()[..])] {
            let (check, _) = check_password("abc", hash, PasswordLockout::default(), 0);
            assert!(!check.ok);
            assert!(!check.configured);
        }
    }

    #[test]
    fn five_failures_lock_for_five_minutes() {
        let mut lockout = PasswordLockout::default();
        for attempt in 1..=MAX_PASSWORD_ATTEMPTS {
            let (check, next) = check_password("wrong", Some(ABC_HASH), lockout, 1000);
            let expected = if attempt == MAX_PASSWORD_ATTEMPTS {
                PASSWORD_LOCKOUT_SECS
            } else {
                0
            };
            assert_eq!(check.retry_after, expected);
            lockout = next;
        }

        let lockout = PasswordLockout::parse(&lockout.encode());
        let (check, _) = check_password("abc", Some(ABC_HASH), lockout, 1299);
        assert!(!check.ok);
        assert_eq!(check.retry_after, 1);
        assert!(check_password("abc", Some(ABC_HASH), lockout, 1300).0.ok);
    }
}
