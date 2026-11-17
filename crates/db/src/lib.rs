//! Database initialization and the single authoritative migration history.
use sqlx::SqlitePool;
use std::path::Path;

/// Embedded migrations, also used by isolated application tests.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Errors opening or migrating the persistent database.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Open the existing SQLite database, apply migrations and restrict its permissions.
pub async fn open(directory: &Path) -> Result<SqlitePool, Error> {
    let path = directory.join("oxevu.sqlite");
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true)
                .foreign_keys(true),
        )
        .await?;
    MIGRATOR.run(&pool).await?;
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    Ok(pool)
}
