//! Opt-in anonymous usage reporting.
//!
//! Three properties make this defensible in a client that holds student
//! credentials, and each one is enforced here rather than promised in a doc:
//!
//! - **No identity.** The `distinct_id` is a UUID minted at process start and
//!   never written to disk, so two runs of the app cannot be tied together by
//!   anyone, including us. Retention and returning-user counts are therefore
//!   impossible by construction, which is the point.
//! - **No free-form payload.** The interface may send an event name from a
//!   fixed list plus one short token; it can never hand this module a string it
//!   picked up from the portal.
//! - **Nothing before consent.** Every capture reads the stored answer first, so
//!   an interface that forgets to ask reports nothing rather than everything.
//!
//! Consent is stored on the Rust side and checked on every capture: the
//! interface cannot report anything by forgetting to ask.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use crate::error::CommandError;

/// PostHog EU region: the data of French students has no reason to cross the
/// Atlantic on the way to a usage counter.
const POSTHOG_HOST: &str = "https://eu.i.posthog.com";

/// The project key. A PostHog project key is a public credential by design — it
/// ships inside every client that reports, so hiding it here would buy nothing.
/// It grants writes only, and to one project that holds no student data.
const POSTHOG_KEY: &str = "phc_mFaV9Ws5xkBT9UfQEkyJivv6b2voGin2RgNoutp4sFEB";

const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);

/// Every event this build can emit. An unknown name is refused, so the set of
/// things the app reports can be read here instead of hunted through the
/// interface.
const ALLOWED_EVENTS: &[&str] = &[
    "consent_accepted",
    "app_launched",
    "login_succeeded",
    "login_failed",
];

/// The longest token the interface may attach to an event, and the only
/// character set allowed in it. Wide enough for the stable error codes the
/// frontend already switches on, far too narrow for a name, an address or a
/// portal URL to survive.
const MAX_VARIANT_LEN: usize = 32;

fn variant_is_safe(variant: &str) -> bool {
    !variant.is_empty()
        && variant.len() <= MAX_VARIANT_LEN
        && variant
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[derive(Serialize, Deserialize)]
struct StoredConsent {
    enabled: bool,
}

/// What the onboarding step needs to know before it decides whether to render.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsStatus {
    /// Whether reporting exists on this platform at all. False collapses the
    /// whole step, the same way an empty permission list does — which is what
    /// the browser preview, with no Rust side to answer, reports for itself.
    pub available: bool,
    /// Whether the reader has already answered. A missing file means never
    /// asked, which is distinct from having said no.
    pub decided: bool,
    pub enabled: bool,
}

pub struct AnalyticsStore {
    consent_path: PathBuf,
    /// `None` until the file has been read once; the inner value is the answer,
    /// itself `None` when the reader has not been asked yet.
    consent: Mutex<Option<Option<bool>>>,
    /// Minted once per process and deliberately not persisted.
    run_id: String,
}

impl AnalyticsStore {
    pub fn new(consent_path: PathBuf) -> Self {
        Self {
            consent_path,
            consent: Mutex::new(None),
            run_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    fn read_consent(&self) -> Result<Option<bool>, CommandError> {
        let mut cache = self
            .consent
            .lock()
            .map_err(|_| CommandError::new("internal_error"))?;
        if let Some(known) = *cache {
            return Ok(known);
        }
        // An unreadable or malformed file reads as "never asked": the reader is
        // asked again rather than silently reported on.
        let stored = fs::read_to_string(&self.consent_path)
            .ok()
            .and_then(|body| serde_json::from_str::<StoredConsent>(&body).ok())
            .map(|stored| stored.enabled);
        *cache = Some(stored);
        Ok(stored)
    }

    async fn write_consent(&self, enabled: bool) -> Result<(), CommandError> {
        let body = serde_json::to_string(&StoredConsent { enabled })
            .map_err(|_| CommandError::new("analytics_store_failed"))?;
        let path = self.consent_path.clone();
        // Creating the directory and writing the file are blocking calls, and
        // the async executor they used to run on is the one serving every other
        // command. Withdrawing consent must not stall a schedule refresh.
        tauri::async_runtime::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, body)
        })
        .await
        .map_err(|_| CommandError::new("analytics_store_failed"))?
        .map_err(|_| CommandError::new("analytics_store_failed"))?;

        // The memo is updated only once the file is on disk, and the guard is
        // taken after the await rather than around it: a `std` guard held across
        // an await point would make this future non-`Send`, and would also keep
        // every reader of the answer waiting on the filesystem.
        let mut cache = self
            .consent
            .lock()
            .map_err(|_| CommandError::new("internal_error"))?;
        *cache = Some(Some(enabled));
        Ok(())
    }

    fn status(&self) -> Result<AnalyticsStatus, CommandError> {
        let consent = self.read_consent()?;
        Ok(AnalyticsStatus {
            available: true,
            decided: consent.is_some(),
            enabled: consent.unwrap_or(false),
        })
    }
}

/// Sends one event, or does nothing at all. Failures are dropped on purpose: a
/// usage counter that cannot be reached is not a problem the reader should ever
/// be shown, and retrying would mean queueing their activity on disk.
async fn send(key: &str, run_id: String, event: String, variant: Option<String>) {
    let mut properties = json!({
        // Server-side captures carry no session of their own, and one run of a
        // desktop app is the closest honest equivalent to a session here.
        "$session_id": run_id,
        // No person profile is created, so nothing accumulates across runs and
        // the event stays in the cheaper anonymous tier.
        "$process_person_profile": false,
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
    });
    if let Some(variant) = variant {
        properties["variant"] = json!(variant);
    }

    let body = json!({
        "api_key": key,
        "event": event,
        "distinct_id": run_id,
        "properties": properties,
    });

    // Serialized here rather than through `reqwest`'s `json` feature: the shared
    // client is built without it, and one counter is no reason to widen it.
    let Ok(payload) = serde_json::to_vec(&body) else {
        return;
    };
    let Ok(client) = reqwest::Client::builder().timeout(CAPTURE_TIMEOUT).build() else {
        return;
    };
    let result = client
        .post(format!("{POSTHOG_HOST}/i/v0/e/"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload)
        .send()
        .await;

    // Visible while developing, silent in a release bundle: the reader gets no
    // diagnostics about a service they may not even have agreed to talk to.
    #[cfg(debug_assertions)]
    match result {
        Ok(response) if !response.status().is_success() => {
            eprintln!("analytics: capture refused with {}", response.status());
        }
        Err(error) => eprintln!("analytics: capture failed: {error}"),
        Ok(_) => {}
    }
    #[cfg(not(debug_assertions))]
    let _ = result;
}

/// Whether to ask, and what was answered last time.
#[tauri::command]
pub fn analytics_status(store: State<'_, AnalyticsStore>) -> Result<AnalyticsStatus, CommandError> {
    store.status()
}

/// Records the answer, whether it is given on the onboarding step or changed
/// later from the settings.
///
/// Accepting is itself reported, because that is the one moment consent is
/// certain. Refusing reports nothing — sending "this reader said no" would be
/// the exact act being refused — so acceptance is compared against the download
/// counts of the release instead.
#[tauri::command]
pub async fn set_analytics_consent(
    store: State<'_, AnalyticsStore>,
    enabled: bool,
) -> Result<AnalyticsStatus, CommandError> {
    store.write_consent(enabled).await?;
    if enabled {
        // The interface never waits on a usage counter. Awaiting the capture
        // here used to hold the answer back for as long as `CAPTURE_TIMEOUT`,
        // which turns a settings switch into a five second freeze on a bad
        // network — for the one control a reader uses to say stop.
        let run_id = store.run_id.clone();
        tauri::async_runtime::spawn(async move {
            send(POSTHOG_KEY, run_id, "consent_accepted".into(), None).await
        });
    }
    store.status()
}

/// Whether a capture reaches the network, refusing outright the ones that must
/// never leave the machine.
///
/// Split out of the command so the guarantee can be asserted on its own: the
/// command needs a Tauri `State`, which a unit test has no way to build, and
/// "emits nothing without consent" is the one property of this module worth
/// more than the plumbing around it. The command below is the only caller.
fn capture_decision(
    store: &AnalyticsStore,
    event: &str,
    variant: Option<&str>,
) -> Result<bool, CommandError> {
    if !ALLOWED_EVENTS.contains(&event) {
        return Err(CommandError::new("analytics_event_unknown"));
    }
    if let Some(variant) = variant {
        if !variant_is_safe(variant) {
            return Err(CommandError::new("analytics_variant_rejected"));
        }
    }
    Ok(store.read_consent()? == Some(true))
}

/// Reports `event`, if this build can report and the reader agreed to it.
#[tauri::command]
pub fn capture_analytics_event(
    store: State<'_, AnalyticsStore>,
    event: String,
    variant: Option<String>,
) -> Result<(), CommandError> {
    if !capture_decision(&store, &event, variant.as_deref())? {
        return Ok(());
    }
    // The interface never waits on a usage counter.
    let run_id = store.run_id.clone();
    tauri::async_runtime::spawn(async move { send(POSTHOG_KEY, run_id, event, variant).await });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{
        capture_decision, variant_is_safe, AnalyticsStore, StoredConsent, MAX_VARIANT_LEN,
    };

    /// A path of its own per test: the store memoises the answer it reads, so
    /// two tests sharing a file would also share whichever one wrote first.
    fn temporary_consent_path(label: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("betteraimaira-{label}-{nonce}.json"))
    }

    fn store_answering(label: &str, stored: Option<bool>) -> AnalyticsStore {
        let path = temporary_consent_path(label);
        if let Some(enabled) = stored {
            fs::write(
                &path,
                serde_json::to_string(&StoredConsent { enabled }).unwrap(),
            )
            .unwrap();
        }
        AnalyticsStore::new(path)
    }

    #[test]
    fn a_reader_who_was_never_asked_is_never_reported_on() {
        let store = store_answering("analytics-unasked", None);

        // Not an error: an interface that forgot the consent step still gets a
        // working command, it just reports nothing.
        assert!(!capture_decision(&store, "app_launched", None).unwrap());
    }

    #[test]
    fn a_refusal_keeps_every_allowed_event_off_the_wire() {
        let store = store_answering("analytics-refused", Some(false));

        for event in ["consent_accepted", "app_launched", "login_succeeded"] {
            assert!(!capture_decision(&store, event, None).unwrap(), "{event}");
        }
    }

    #[test]
    fn only_a_stored_yes_lets_an_event_through() {
        let store = store_answering("analytics-accepted", Some(true));

        assert!(capture_decision(&store, "app_launched", None).unwrap());
        assert!(capture_decision(&store, "login_failed", Some("bad_credentials")).unwrap());
    }

    #[test]
    fn a_consent_file_that_cannot_be_understood_reads_as_never_asked() {
        let path = temporary_consent_path("analytics-corrupt");
        fs::write(&path, "{ this is not the file we wrote }").unwrap();
        let store = AnalyticsStore::new(path);

        // Silence, not a guess: a damaged file must never be read as a yes.
        assert!(!capture_decision(&store, "app_launched", None).unwrap());
    }

    #[test]
    fn an_event_the_build_cannot_emit_is_refused_before_consent_is_even_read() {
        // Consent is granted, so a refusal here can only come from the name.
        let store = store_answering("analytics-unknown-event", Some(true));

        let refusal = capture_decision(&store, "grade_viewed", None).unwrap_err();
        assert_eq!(refusal.code, "analytics_event_unknown");
    }

    #[test]
    fn a_variant_wide_enough_to_carry_portal_content_is_refused() {
        let store = store_answering("analytics-variant", Some(true));

        // A grade, a name, an address, a portal URL: none of these survive the
        // shape the variant is allowed to have.
        let too_long = "a".repeat(MAX_VARIANT_LEN + 1);
        for smuggled in [
            "16/20",
            "DUCHEMIN Loïc",
            "https://portal.example.test/",
            "student@example.test",
            "bad credentials",
            "Bad_Credentials",
            "",
            too_long.as_str(),
        ] {
            let refusal = capture_decision(&store, "login_failed", Some(smuggled))
                .expect_err(&format!("{smuggled:?} must not be reportable"));
            assert_eq!(refusal.code, "analytics_variant_rejected", "{smuggled:?}");
        }
    }

    #[test]
    fn a_variant_is_a_short_lowercase_token_and_nothing_else() {
        assert!(variant_is_safe("portal_unreachable"));
        assert!(variant_is_safe("http_502"));
        assert!(variant_is_safe(&"a".repeat(MAX_VARIANT_LEN)));

        assert!(!variant_is_safe(&"a".repeat(MAX_VARIANT_LEN + 1)));
        assert!(!variant_is_safe(
            "
"
        ));
        assert!(!variant_is_safe(""));
        assert!(!variant_is_safe("has space"));
        assert!(!variant_is_safe("UPPER"));
        assert!(!variant_is_safe("dashed-token"));
        assert!(!variant_is_safe("é"));
    }
}
