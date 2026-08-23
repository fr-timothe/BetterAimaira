use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::aimaira::Grade;

#[derive(Clone)]
pub struct GradeSyncStore {
    database_path: PathBuf,
    schema_status: Arc<OnceLock<Result<(), String>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeSyncResult {
    pub grades: Vec<Grade>,
    pub unread_alerts: Vec<Grade>,
    pub initialized: bool,
}

impl GradeSyncStore {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            schema_status: Arc::new(OnceLock::new()),
        }
    }

    pub fn sync(&self, account_key: &str, grades: Vec<Grade>) -> Result<GradeSyncResult, String> {
        let mut connection = self.open_connection()?;
        let initialized = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM grade_sync_accounts WHERE account_key = ?1)",
                [account_key],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| error.to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        let known_ids = known_grade_ids(&transaction, account_key)?;

        for grade in &grades {
            let grade_json = serde_json::to_string(grade).map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "INSERT OR REPLACE INTO grade_snapshots (account_key, grade_id, grade_json) VALUES (?1, ?2, ?3)",
                    params![account_key, grade.id, grade_json],
                )
                .map_err(|error| error.to_string())?;
            if initialized && !known_ids.contains(&grade.id) {
                transaction
                    .execute(
                        "INSERT OR IGNORE INTO grade_alerts (account_key, grade_id) VALUES (?1, ?2)",
                        params![account_key, grade.id],
                    )
                    .map_err(|error| error.to_string())?;
            }
        }

        transaction
            .execute(
                "INSERT OR IGNORE INTO grade_sync_accounts (account_key) VALUES (?1)",
                [account_key],
            )
            .map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| error.to_string())?;

        Ok(GradeSyncResult {
            unread_alerts: unread_alerts(&connection, account_key)?,
            grades,
            initialized,
        })
    }

    pub fn mark_alerts_read(&self, account_key: &str) -> Result<(), String> {
        let connection = self.open_connection()?;
        connection
            .execute(
                "UPDATE grade_alerts SET read_at = unixepoch() WHERE account_key = ?1 AND read_at IS NULL",
                [account_key],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn open_connection(&self) -> Result<Connection, String> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let connection = Connection::open(&self.database_path).map_err(|error| error.to_string())?;
        self.ensure_schema(&connection)?;
        Ok(connection)
    }

    fn ensure_schema(&self, connection: &Connection) -> Result<(), String> {
        self.schema_status
            .get_or_init(|| create_schema(connection))
            .clone()
    }
}

fn create_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS grade_sync_accounts (
                account_key TEXT PRIMARY KEY
            );
            CREATE TABLE IF NOT EXISTS grade_snapshots (
                account_key TEXT NOT NULL,
                grade_id TEXT NOT NULL,
                grade_json TEXT NOT NULL,
                PRIMARY KEY (account_key, grade_id)
            );
            CREATE TABLE IF NOT EXISTS grade_alerts (
                account_key TEXT NOT NULL,
                grade_id TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                read_at INTEGER,
                PRIMARY KEY (account_key, grade_id)
            );
            ",
        )
        .map_err(|error| error.to_string())
}

fn known_grade_ids(connection: &Connection, account_key: &str) -> Result<HashSet<String>, String> {
    let mut statement = connection
        .prepare("SELECT grade_id FROM grade_snapshots WHERE account_key = ?1")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([account_key], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())
}

fn unread_alerts(connection: &Connection, account_key: &str) -> Result<Vec<Grade>, String> {
    let mut statement = connection
        .prepare(
            "SELECT snapshot.grade_json
             FROM grade_alerts AS alert
             INNER JOIN grade_snapshots AS snapshot
               ON snapshot.account_key = alert.account_key AND snapshot.grade_id = alert.grade_id
             WHERE alert.account_key = ?1 AND alert.read_at IS NULL
             ORDER BY alert.created_at DESC",
        )
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
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::GradeSyncStore;
    use crate::aimaira::Grade;

    fn test_store() -> GradeSyncStore {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        GradeSyncStore::new(
            std::env::temp_dir().join(format!("betteraimaira-grade-sync-{nonce}.sqlite")),
        )
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
    fn first_sync_is_silent_and_later_grade_is_alerted_once() {
        let store = test_store();
        let first = store.sync("account", vec![grade("first")]).unwrap();
        assert!(!first.initialized);
        assert!(first.unread_alerts.is_empty());

        let second = store
            .sync("account", vec![grade("first"), grade("second")])
            .unwrap();
        assert!(second.initialized);
        assert_eq!(second.unread_alerts.len(), 1);
        assert_eq!(second.unread_alerts[0].id, "second");

        let third = store
            .sync("account", vec![grade("first"), grade("second")])
            .unwrap();
        assert_eq!(third.unread_alerts.len(), 1);
        store.mark_alerts_read("account").unwrap();
        assert!(store
            .sync("account", vec![grade("first"), grade("second")])
            .unwrap()
            .unread_alerts
            .is_empty());
    }
}
