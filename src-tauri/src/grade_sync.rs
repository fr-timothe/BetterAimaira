use rusqlite::{params, Connection};
use serde::Serialize;

use crate::aimaira::Grade;
use crate::storage::Storage;

#[derive(Clone)]
pub struct GradeSyncStore {
    storage: Storage,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeSyncResult {
    pub grades: Vec<Grade>,
    /// Raised when the grades come from the stored snapshots because the portal
    /// could not be reached.
    pub stale: bool,
}

impl GradeSyncStore {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Records the grades the portal just returned and hands them straight
    /// back, so the next offline read replays exactly what the reader saw
    /// online. The stored rows are a mirror, not a history: everything the
    /// account had is cleared first, otherwise grades the portal has stopped
    /// listing — an older school year, a mark the school withdrew — would keep
    /// surfacing offline long after they left the online view.
    pub fn persist(
        &self,
        account_key: &str,
        grades: Vec<Grade>,
    ) -> Result<GradeSyncResult, String> {
        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "DELETE FROM grade_snapshots WHERE account_key = ?1",
                [account_key],
            )
            .map_err(|error| error.to_string())?;

        for grade in &grades {
            let grade_json = serde_json::to_string(grade).map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "INSERT OR REPLACE INTO grade_snapshots (account_key, grade_id, grade_json) VALUES (?1, ?2, ?3)",
                    params![account_key, grade.id, grade_json],
                )
                .map_err(|error| error.to_string())?;
        }

        transaction.commit().map_err(|error| error.to_string())?;

        Ok(GradeSyncResult {
            grades,
            stale: false,
        })
    }

    /// The last grades stored for the account, for when the portal is out of
    /// reach. Nothing is written here: a page the app could not refresh is no
    /// evidence of what the portal holds now.
    pub fn stored_snapshot(&self, account_key: &str) -> Result<GradeSyncResult, String> {
        let connection = self.storage.open_connection()?;
        Ok(GradeSyncResult {
            grades: stored_grades(&connection, account_key)?,
            stale: true,
        })
    }

    fn open_connection(&self) -> Result<Connection, String> {
        self.storage.open_connection()
    }
}

fn stored_grades(connection: &Connection, account_key: &str) -> Result<Vec<Grade>, String> {
    let mut statement = connection
        .prepare("SELECT grade_json FROM grade_snapshots WHERE account_key = ?1 ORDER BY grade_id")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([account_key], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    rows.map(|row| {
        let value = row.map_err(|error| error.to_string())?;
        serde_json::from_str(&value).map_err(|error| error.to_string())
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::GradeSyncStore;
    use crate::aimaira::Grade;
    use crate::storage::{temporary_database_path, Storage};

    fn test_store() -> GradeSyncStore {
        GradeSyncStore::new(Storage::new(temporary_database_path("grade-sync")))
    }

    fn grade(id: &str) -> Grade {
        Grade {
            id: id.to_owned(),
            subject: "Mathématiques".to_owned(),
            label: "Partiel".to_owned(),
            score: "16".to_owned(),
            scale: Some("20".to_owned()),
            coefficient: None,
            average: None,
        }
    }

    #[test]
    fn stored_snapshot_replays_the_grades() {
        let store = test_store();
        store.persist("account", vec![grade("first")]).unwrap();

        let offline = store.stored_snapshot("account").unwrap();
        assert!(offline.stale);
        assert_eq!(offline.grades.len(), 1);
        assert_eq!(offline.grades[0].id, "first");
    }

    #[test]
    fn a_grade_the_portal_stopped_listing_leaves_the_snapshot() {
        let store = test_store();
        store
            .persist("account", vec![grade("first"), grade("second")])
            .unwrap();
        store.persist("account", vec![grade("second")]).unwrap();

        let offline = store.stored_snapshot("account").unwrap();
        assert_eq!(offline.grades.len(), 1);
        assert_eq!(offline.grades[0].id, "second");
    }
}
