use std::sync::OnceLock;

use keyring_core::{Entry, Error};
use reqwest::Url;
use serde::{Deserialize, Serialize};

const SERVICE_PREFIX: &str = "com.betteraimaira.app";
const IDENTITY_ACCOUNT: &str = "last-used";
static KEYRING_STATUS: OnceLock<Result<(), String>> = OnceLock::new();

pub struct SavedCredentials {
    pub portal_url: Url,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
struct SavedIdentity {
    portal_url: String,
    username: String,
}

pub fn save_password(portal_url: &Url, username: &str, password: &str) -> Result<(), String> {
    initialize_keyring()?;
    password_entry(portal_url, username)?
        .set_password(password)
        .map_err(|error| error.to_string())?;

    let identity = SavedIdentity {
        portal_url: portal_url.to_string(),
        username: username.to_owned(),
    };
    let payload = serde_json::to_string(&identity).map_err(|error| error.to_string())?;
    identity_entry()?
        .set_password(&payload)
        .map_err(|error| error.to_string())
}

pub fn load_credentials() -> Result<Option<SavedCredentials>, String> {
    let Some((identity, portal_url)) = read_saved_identity()? else {
        return Ok(None);
    };
    let Some(password) = get_entry_password(&password_entry(&portal_url, &identity.username)?)?
    else {
        return Ok(None);
    };

    Ok(Some(SavedCredentials {
        portal_url,
        username: identity.username,
        password,
    }))
}

/// The identity alone, without touching the password entry: the startup screen
/// needs to know whether a session will be restored before the slow parts run.
pub fn load_identity() -> Result<Option<(Url, String)>, String> {
    Ok(read_saved_identity()?.map(|(identity, portal_url)| (portal_url, identity.username)))
}

/// Forgets the saved password without forgetting whose it was.
///
/// A portal answering `invalid_credentials` proves the password is wrong; it
/// proves nothing about the account. Keeping the identity is what lets
/// `resolve_account_key` still name the rows already stored on disk, so an
/// offline cold start has something to show, and what lets the login form open
/// pre-filled with the portal and the username.
///
/// The consequence is deliberate: the next cold start finds an identity but no
/// password, so `load_credentials` answers `None` and `restore_session` reports
/// `no_credentials` instead of `credentials_rejected`. A day later, a pre-filled
/// form asking for the password is more honest than replaying a rejection the
/// user has already seen. The run that saw it is unaffected — it set its own
/// status before returning.
///
/// A no-op when nothing is saved.
pub fn clear_saved_password() -> Result<(), String> {
    let Some((identity, portal_url)) = read_saved_identity()? else {
        return Ok(());
    };
    delete_entry_if_exists(&password_entry(&portal_url, &identity.username)?)
}

/// The full sign-out: the password and the identity that names it.
///
/// Dropping the identity orphans every snapshot stored under that account key,
/// which is what the user asked for here and the wrong answer to a rejected
/// password — see [`clear_saved_password`].
pub fn clear_saved_credentials() -> Result<(), String> {
    clear_saved_password()?;
    delete_entry_if_exists(&identity_entry()?)
}

fn read_saved_identity() -> Result<Option<(SavedIdentity, Url)>, String> {
    initialize_keyring()?;
    let Some(raw_identity) = get_entry_password(&identity_entry()?)? else {
        return Ok(None);
    };
    let identity: SavedIdentity =
        serde_json::from_str(&raw_identity).map_err(|error| error.to_string())?;
    let portal_url = Url::parse(&identity.portal_url).map_err(|error| error.to_string())?;
    Ok(Some((identity, portal_url)))
}

fn get_entry_password(entry: &Entry) -> Result<Option<String>, String> {
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn delete_entry_if_exists(entry: &Entry) -> Result<(), String> {
    match entry.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn password_entry(portal_url: &Url, username: &str) -> Result<Entry, String> {
    let host = portal_url
        .host_str()
        .ok_or_else(|| "Portal host is missing".to_owned())?;
    let service = format!("{SERVICE_PREFIX}:{host}");
    Entry::new(&service, username).map_err(|error| error.to_string())
}

fn identity_entry() -> Result<Entry, String> {
    initialize_keyring()?;
    Entry::new(SERVICE_PREFIX, IDENTITY_ACCOUNT).map_err(|error| error.to_string())
}

fn initialize_keyring() -> Result<(), String> {
    KEYRING_STATUS.get_or_init(install_default_store).clone()
}

/// Unit tests run against an in-memory store instead of the platform one.
///
/// The service names below are the same in a test binary as in the shipped
/// app, so letting the tests reach the real credential manager would read and
/// delete the developer's own saved password. Swapping the store here rather
/// than at the call sites keeps the production path free of test branches.
#[cfg(test)]
fn install_default_store() -> Result<(), String> {
    keyring_core::mock::Store::new()
        .map(|store| keyring_core::set_default_store(store))
        .map_err(|error| error.to_string())
}

#[cfg(not(test))]
fn install_default_store() -> Result<(), String> {
    #[cfg(target_os = "android")]
    let result = android_native_keyring_store::Store::new()
        .map(|store| keyring_core::set_default_store(store));
    #[cfg(target_os = "ios")]
    let result = apple_native_keyring_store::protected::Store::new()
        .map(|store| keyring_core::set_default_store(store));
    #[cfg(target_os = "macos")]
    let result = apple_native_keyring_store::keychain::Store::new()
        .map(|store| keyring_core::set_default_store(store));
    #[cfg(target_os = "windows")]
    let result = windows_native_keyring_store::Store::new()
        .map(|store| keyring_core::set_default_store(store));
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    let result = zbus_secret_service_keyring_store::Store::new()
        .map(|store| keyring_core::set_default_store(store));
    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    let result = Err(keyring_core::Error::NotSupportedByStore(
        "No native credential store is available".to_owned(),
    ));

    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        clear_saved_credentials, clear_saved_password, load_credentials, load_identity,
        save_password,
    };
    use reqwest::Url;

    /// One test, not two: the identity entry is a single well-known slot in the
    /// process-wide store, so two tests touching it would race each other.
    #[test]
    fn clearing_the_password_keeps_the_identity_that_names_the_stored_snapshots() {
        let portal_url = Url::parse("https://portal.example.test/").expect("a valid portal URL");
        save_password(&portal_url, "student", "correct horse").expect("the password is saved");

        clear_saved_password().expect("the password is cleared");

        // The password is gone, so the next cold start will not burn a login
        // round-trip on it...
        assert!(load_credentials().expect("the store answers").is_none());
        // ...but the account is still named, so the snapshots on disk are still
        // reachable and the login form still opens pre-filled.
        assert_eq!(
            load_identity().expect("the store answers"),
            Some((portal_url.clone(), "student".to_owned()))
        );

        // A second clear on a password that is already gone is a no-op.
        clear_saved_password().expect("clearing twice is harmless");
        assert!(load_identity().expect("the store answers").is_some());

        // The full sign-out forgets the account itself.
        save_password(&portal_url, "student", "correct horse").expect("the password is saved");
        clear_saved_credentials().expect("the sign-out succeeds");
        assert!(load_identity().expect("the store answers").is_none());
        assert!(load_credentials().expect("the store answers").is_none());
    }
}
