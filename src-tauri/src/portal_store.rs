//! On-disk copies of what the portal last answered.
//!
//! The session cache in `state.rs` dies with the process, so a cold start with
//! no network never reached the main screen. These snapshots are the second
//! tier: slower than memory, but they survive a restart and let the app open
//! offline with the last known content marked as stale.

use rusqlite::{params, Connection};

use crate::aimaira::{CalendarEvent, PortalPage, PortalResource};
use crate::storage::Storage;

#[derive(Clone)]
pub struct PortalStore {
    storage: Storage,
}

pub struct StoredSchedule {
    pub events: Vec<CalendarEvent>,
    pub fetched_at: u64,
}

/// Identifies a schedule request the way the portal itself does: the two form
/// fields it is sent, joined verbatim. Hashing them would buy nothing and cost
/// readability of the stored rows, and `duration` is a bounded number so the
/// separator can never be ambiguous.
pub fn schedule_range_key(start: &str, duration: u8) -> String {
    format!("{}_{duration}", start.trim())
}

impl PortalStore {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn save_portal_page(&self, account_key: &str, page: &PortalPage) -> Result<(), String> {
        let payload = serde_json::to_string(page).map_err(|error| error.to_string())?;
        self.storage
            .open_connection()?
            .execute(
                "INSERT OR REPLACE INTO portal_snapshots (account_key, resource, payload_json, fetched_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![account_key, page.resource.key(), payload, page.fetched_at as i64],
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    /// Answers the stored page with `stale` raised and the timestamp of the
    /// fetch it came from, never the current time: the interface tells the
    /// reader how old the content is from that field alone.
    pub fn load_portal_page(
        &self,
        account_key: &str,
        resource: PortalResource,
    ) -> Result<Option<PortalPage>, String> {
        let connection = self.storage.open_connection()?;
        let Some((payload, fetched_at)) = query_snapshot(
            &connection,
            "SELECT payload_json, fetched_at FROM portal_snapshots
             WHERE account_key = ?1 AND resource = ?2",
            account_key,
            resource.key(),
        )?
        else {
            return Ok(None);
        };

        let mut page: PortalPage =
            serde_json::from_str(&payload).map_err(|error| error.to_string())?;
        page.fetched_at = fetched_at;
        page.stale = true;
        Ok(Some(page))
    }

    pub fn save_schedule(
        &self,
        account_key: &str,
        range_key: &str,
        events: &[CalendarEvent],
        fetched_at: u64,
    ) -> Result<(), String> {
        let payload = serde_json::to_string(events).map_err(|error| error.to_string())?;
        self.storage
            .open_connection()?
            .execute(
                "INSERT OR REPLACE INTO schedule_snapshots (account_key, range_key, payload_json, fetched_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![account_key, range_key, payload, fetched_at as i64],
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    pub fn load_schedule(
        &self,
        account_key: &str,
        range_key: &str,
    ) -> Result<Option<StoredSchedule>, String> {
        let connection = self.storage.open_connection()?;
        let Some((payload, fetched_at)) = query_snapshot(
            &connection,
            "SELECT payload_json, fetched_at FROM schedule_snapshots
             WHERE account_key = ?1 AND range_key = ?2",
            account_key,
            range_key,
        )?
        else {
            return Ok(None);
        };

        Ok(Some(StoredSchedule {
            events: serde_json::from_str(&payload).map_err(|error| error.to_string())?,
            fetched_at,
        }))
    }

    /// Whether the account has anything at all to show offline. The login
    /// screen uses this to open the app instead of blocking on a portal it
    /// cannot reach.
    pub fn has_snapshots(&self, account_key: &str) -> Result<bool, String> {
        self.storage
            .open_connection()?
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM portal_snapshots WHERE account_key = ?1
                     UNION ALL
                     SELECT 1 FROM schedule_snapshots WHERE account_key = ?1
                     UNION ALL
                     SELECT 1 FROM grade_snapshots WHERE account_key = ?1
                 )",
                [account_key],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| error.to_string())
    }
}

fn query_snapshot(
    connection: &Connection,
    statement: &str,
    account_key: &str,
    key: &str,
) -> Result<Option<(String, u64)>, String> {
    let mut prepared = connection
        .prepare(statement)
        .map_err(|error| error.to_string())?;
    let mut rows = prepared
        .query_map(params![account_key, key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|error| error.to_string())?;
    match rows.next() {
        None => Ok(None),
        Some(row) => {
            let (payload, fetched_at) = row.map_err(|error| error.to_string())?;
            Ok(Some((payload, fetched_at.max(0) as u64)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{schedule_range_key, PortalStore};
    use crate::aimaira::{PortalPage, PortalResource};
    use crate::storage::{temporary_database_path, Storage};

    fn test_store(label: &str) -> PortalStore {
        PortalStore::new(Storage::new(temporary_database_path(label)))
    }

    fn portal_page(resource: PortalResource, fetched_at: u64) -> PortalPage {
        PortalPage {
            resource,
            fetched_at,
            stale: false,
            title: "Mes notes".to_owned(),
            headings: vec!["Année 2025-2026".to_owned()],
            tables: Vec::new(),
            fields: Vec::new(),
            documents: Vec::new(),
            grade_periods: Vec::new(),
            absence_periods: Vec::new(),
            questionnaires: Vec::new(),
            markup_recognized: true,
        }
    }

    #[test]
    fn portal_snapshot_round_trips_and_comes_back_stale() {
        let store = test_store("portal-snapshot");
        assert!(!store.has_snapshots("account").unwrap());
        assert!(store
            .load_portal_page("account", PortalResource::Grades)
            .unwrap()
            .is_none());

        store
            .save_portal_page("account", &portal_page(PortalResource::Grades, 1_700_000))
            .unwrap();

        let restored = store
            .load_portal_page("account", PortalResource::Grades)
            .unwrap()
            .unwrap();
        assert_eq!(restored.resource, PortalResource::Grades);
        assert_eq!(restored.title, "Mes notes");
        assert_eq!(restored.headings, vec!["Année 2025-2026".to_owned()]);
        assert!(restored.markup_recognized);
        // The age the interface shows must be the age of the fetch, not of the read.
        assert_eq!(restored.fetched_at, 1_700_000);
        assert!(restored.stale);

        assert!(store.has_snapshots("account").unwrap());
        assert!(!store.has_snapshots("other-account").unwrap());
        // Another resource of the same account is a separate row.
        assert!(store
            .load_portal_page("account", PortalResource::Absences)
            .unwrap()
            .is_none());

        store
            .save_portal_page("account", &portal_page(PortalResource::Grades, 1_800_000))
            .unwrap();
        assert_eq!(
            store
                .load_portal_page("account", PortalResource::Grades)
                .unwrap()
                .unwrap()
                .fetched_at,
            1_800_000
        );
    }

    #[test]
    fn schedule_snapshot_round_trips_for_its_own_range() {
        let store = test_store("schedule-snapshot");
        let range = schedule_range_key("2026-08-24", 7);
        store
            .save_schedule("account", &range, &[], 1_700_000)
            .unwrap();

        let restored = store.load_schedule("account", &range).unwrap().unwrap();
        assert!(restored.events.is_empty());
        assert_eq!(restored.fetched_at, 1_700_000);
        assert!(store
            .load_schedule("account", &schedule_range_key("2026-08-31", 7))
            .unwrap()
            .is_none());
    }

    #[test]
    fn schedule_range_key_is_stable_and_distinguishes_ranges() {
        assert_eq!(schedule_range_key("2026-08-24", 7), "2026-08-24_7");
        assert_eq!(schedule_range_key("  2026-08-24  ", 7), "2026-08-24_7");
        assert_eq!(
            schedule_range_key("2026-08-24", 7),
            schedule_range_key("2026-08-24", 7)
        );
        assert_ne!(
            schedule_range_key("2026-08-24", 7),
            schedule_range_key("2026-08-24", 14)
        );
        assert_ne!(
            schedule_range_key("2026-08-24", 7),
            schedule_range_key("2026-08-31", 7)
        );
    }
}
