//! Local SQLite database: connection setup and migrations.

use std::{path::Path, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

pub const DB_FILE_NAME: &str = "paideia.db";

/// Migrations are compiled into the binary, so a fresh install needs no setup.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Open (or create) the database in `data_dir` and bring its schema up to date.
pub async fn open(data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(data_dir.join(DB_FILE_NAME))
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new().max_connections(4).connect_with(options).await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// A private in-memory database, for tests.
#[cfg(test)]
pub async fn open_in_memory() -> SqlitePool {
    use std::str::FromStr;
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("valid url")
        .foreign_keys(true);
    // One connection: every connection to `:memory:` is a separate database.
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("open in-memory database");
    MIGRATOR.run(&pool).await.expect("run migrations");
    pool
}
