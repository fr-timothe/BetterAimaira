//! The single owner of the local SQLite file.
//!
//! Every store in the app (grade snapshots, portal snapshots) shares one
//! database, so connection handling and schema migration live here rather than
//! being duplicated — and repeated — per store.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;

/// The app makes short writes from `spawn_blocking` tasks that can overlap, and
/// SQLite's default is to fail the loser instantly. Waiting a few seconds turns
/// a hard `database is locked` error into the queueing the caller expected.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// One entry per schema version, applied in order, each one upgrading the
/// database by exactly one step. `PRAGMA user_version` records how many have
/// run, so a new migration means appending a batch to this list and nothing
/// else.
///
/// `migrate()` replays this list with `.skip(applied)`, which pins every entry
/// to its index: the list may only ever be appended to, never shortened or
/// reordered, and an already released entry may never be edited — a client
/// that ran the old text would simply never see the new one. Version 1 is the
/// single exception, and only while it holds: the counter was introduced on
/// this branch and has not shipped, so no install has ever reached
/// `user_version = 1` and the batch below has never run anywhere.
const MIGRATIONS: &[&str] = &[VERSION_1];

/// Uses `CREATE TABLE IF NOT EXISTS` because installs that predate the
/// migration counter sit at `user_version = 0` with the grade tables already
/// there. The two `DROP TABLE` statements clear the new-grade detection those
/// same installs carry from `0.1.1-beta.14`; on a fresh install they are a
/// no-op and the tables are simply never created.
const VERSION_1: &str = "
    DROP TABLE IF EXISTS grade_alerts;
    DROP TABLE IF EXISTS grade_sync_accounts;
    CREATE TABLE IF NOT EXISTS grade_snapshots (
        account_key TEXT NOT NULL,
        grade_id TEXT NOT NULL,
        grade_json TEXT NOT NULL,
        PRIMARY KEY (account_key, grade_id)
    );
    CREATE TABLE IF NOT EXISTS portal_snapshots (
        account_key TEXT NOT NULL,
        resource TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        fetched_at INTEGER NOT NULL,
        PRIMARY KEY (account_key, resource)
    );
    CREATE TABLE IF NOT EXISTS schedule_snapshots (
        account_key TEXT NOT NULL,
        range_key TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        fetched_at INTEGER NOT NULL,
        PRIMARY KEY (account_key, range_key)
    );
";

#[derive(Clone)]
pub struct Storage {
    database_path: PathBuf,
    /// Set once the migrations have run through. Only success is remembered: an
    /// earlier version cached the failure too, so a single unwritable data
    /// directory at startup left every store dead for the rest of the process
    /// even after the cause was gone.
    schema_ready: Arc<Mutex<bool>>,
}

impl Storage {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            schema_ready: Arc::new(Mutex::new(false)),
        }
    }

    /// Opens a migrated connection. Blocking, so callers on the async runtime
    /// must reach it through `spawn_blocking`.
    pub fn open_connection(&self) -> Result<Connection, String> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut connection =
            Connection::open(&self.database_path).map_err(|error| error.to_string())?;
        connection
            .busy_timeout(BUSY_TIMEOUT)
            .map_err(|error| error.to_string())?;
        self.ensure_schema(&mut connection)?;
        Ok(connection)
    }

    fn ensure_schema(&self, connection: &mut Connection) -> Result<(), String> {
        let mut schema_ready = self
            .schema_ready
            .lock()
            .map_err(|_| "Schema state is poisoned".to_owned())?;
        if *schema_ready {
            return Ok(());
        }
        migrate(connection)?;
        *schema_ready = true;
        Ok(())
    }
}

fn migrate(connection: &mut Connection) -> Result<(), String> {
    let applied = read_user_version(connection)?;
    for (index, statements) in MIGRATIONS.iter().enumerate().skip(applied) {
        let target = index + 1;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute_batch(statements)
            .map_err(|error| error.to_string())?;
        // `user_version` is transactional in SQLite, so a step that fails
        // halfway leaves the counter untouched and is retried on the next open.
        transaction
            .pragma_update(None, "user_version", target as i64)
            .map_err(|error| error.to_string())?;
        transaction.commit().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn read_user_version(connection: &Connection) -> Result<usize, String> {
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    Ok(version.max(0) as usize)
}

#[cfg(test)]
pub(crate) fn temporary_database_path(label: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("betteraimaira-{label}-{nonce}.sqlite"))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{temporary_database_path, Storage, MIGRATIONS};

    fn table_exists(connection: &Connection, name: &str) -> bool {
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                [name],
                |row| row.get::<_, bool>(0),
            )
            .unwrap()
    }

    fn user_version(connection: &Connection) -> i64 {
        connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn migrates_an_existing_grade_only_database_without_losing_its_rows() {
        let database_path = temporary_database_path("storage-migration");
        let legacy = Connection::open(&database_path).unwrap();
        legacy
            .execute_batch(
                "
                CREATE TABLE grade_sync_accounts (account_key TEXT PRIMARY KEY);
                CREATE TABLE grade_snapshots (
                    account_key TEXT NOT NULL,
                    grade_id TEXT NOT NULL,
                    grade_json TEXT NOT NULL,
                    PRIMARY KEY (account_key, grade_id)
                );
                CREATE TABLE grade_alerts (
                    account_key TEXT NOT NULL,
                    grade_id TEXT NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                    read_at INTEGER,
                    PRIMARY KEY (account_key, grade_id)
                );
                INSERT INTO grade_snapshots VALUES ('account', 'grade', '{}');
                ",
            )
            .unwrap();
        // Installs that predate the counter sit at version 0 with the tables
        // already there, which is exactly the case version 1 has to survive.
        assert_eq!(user_version(&legacy), 0);
        drop(legacy);

        let storage = Storage::new(database_path);
        let connection = storage.open_connection().unwrap();

        assert_eq!(user_version(&connection), MIGRATIONS.len() as i64);
        assert!(table_exists(&connection, "portal_snapshots"));
        assert!(table_exists(&connection, "schedule_snapshots"));
        // New-grade detection is gone, so its two tables have to leave with it
        // rather than linger on every install that already shipped them.
        assert!(!table_exists(&connection, "grade_alerts"));
        assert!(!table_exists(&connection, "grade_sync_accounts"));
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM grade_snapshots", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn reopening_a_migrated_database_is_a_no_op() {
        let database_path = temporary_database_path("storage-reopen");
        Storage::new(database_path.clone())
            .open_connection()
            .unwrap();

        // A separate `Storage` does not share the cached schema flag, so it
        // runs the migration loop again and proves it is idempotent.
        let connection = Storage::new(database_path).open_connection().unwrap();
        assert_eq!(user_version(&connection), MIGRATIONS.len() as i64);
    }
}
