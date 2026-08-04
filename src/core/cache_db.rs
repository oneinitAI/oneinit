//! SQLite cache for resolved versions and checksums (non-exact-match recipe
//! system). Tables live in the same `~/.oneinit/db/oneinit.db` as the install
//! manifest, with their own tables.

use super::{Result, db_dir};

/// Open the shared SQLite DB and ensure the cache tables exist.
fn open() -> Result<rusqlite::Connection> {
    let db_path = db_dir().join("oneinit.db");
    let conn = rusqlite::Connection::open(&db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS version_cache (
            recipe TEXT NOT NULL,
            version TEXT NOT NULL,
            source TEXT NOT NULL,
            fetched_at INTEGER NOT NULL,
            PRIMARY KEY (recipe, version)
        );
        CREATE TABLE IF NOT EXISTS checksum_cache (
            recipe TEXT NOT NULL,
            version TEXT NOT NULL,
            platform TEXT NOT NULL,
            filename TEXT NOT NULL,
            checksum TEXT NOT NULL,
            source TEXT,
            fetched_at INTEGER NOT NULL,
            PRIMARY KEY (recipe, version, platform, filename)
        );
        PRAGMA journal_mode=WAL;",
    )?;
    Ok(conn)
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Insert a resolved version into the cache.
pub fn cache_version(recipe: &str, version: &str, source: &str) -> Result<()> {
    let conn = open()?;
    conn.execute(
        "INSERT OR REPLACE INTO version_cache (recipe, version, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![recipe, version, source, now()],
    )?;
    Ok(())
}

/// All cached resolved versions for a recipe, newest first.
pub fn cached_versions(recipe: &str) -> Result<Vec<String>> {
    let conn = open()?;
    let mut stmt = conn
        .prepare("SELECT version FROM version_cache WHERE recipe = ?1 ORDER BY fetched_at DESC")?;
    let rows = stmt.query_map([recipe], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Cache a resolved checksum.
pub fn cache_checksum(
    recipe: &str,
    version: &str,
    platform: &str,
    filename: &str,
    checksum: &str,
    source: &str,
) -> Result<()> {
    let conn = open()?;
    conn.execute(
        "INSERT OR REPLACE INTO checksum_cache
            (recipe, version, platform, filename, checksum, source, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![recipe, version, platform, filename, checksum, source, now()],
    )?;
    Ok(())
}

/// Look up a cached checksum.
pub fn cached_checksum(
    recipe: &str,
    version: &str,
    platform: &str,
    filename: &str,
) -> Option<String> {
    let conn = open().ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT checksum FROM checksum_cache
             WHERE recipe = ?1 AND version = ?2 AND platform = ?3 AND filename = ?4",
        )
        .ok()?;
    stmt.query_row(
        rusqlite::params![recipe, version, platform, filename],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

/// Drop cached entries for a recipe (used by --refresh).
pub fn invalidate(recipe: &str) -> Result<()> {
    let conn = open()?;
    conn.execute("DELETE FROM version_cache WHERE recipe = ?1", [recipe])?;
    conn.execute("DELETE FROM checksum_cache WHERE recipe = ?1", [recipe])?;
    Ok(())
}
