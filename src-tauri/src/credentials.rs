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
    let Some(password) = get_entry_password(&password_entry(&portal_url, &identity.username)?)? else {
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

pub fn clear_saved_credentials() -> Result<(), String> {
    if let Some((identity, portal_url)) = read_saved_identity()? {
        delete_entry_if_exists(&password_entry(&portal_url, &identity.username)?)?;
    }
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
    KEYRING_STATUS
        .get_or_init(|| {
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
        })
        .clone()
}
