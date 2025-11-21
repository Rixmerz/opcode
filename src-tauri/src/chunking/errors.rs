use super::storage::{get_error_logs, upsert_error_log};
use super::types::ErrorLog;
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

/// Registra un error/log
pub fn log_error(
    conn: &Connection,
    project_path: &str,
    error_type: &str,
    message: &str,
    file_path: Option<&str>,
    entity_name: Option<&str>,
    stacktrace: Option<&str>,
    snapshot_id: Option<i64>,
) -> Result<i64> {
    let error = ErrorLog {
        id: None,
        project_path: project_path.to_string(),
        snapshot_id,
        file_path: file_path.map(|s| s.to_string()),
        entity_name: entity_name.map(|s| s.to_string()),
        error_type: error_type.to_string(),
        message: message.to_string(),
        stacktrace: stacktrace.map(|s| s.to_string()),
        occurrence_count: 1,
        first_seen: Utc::now(),
        last_seen: Utc::now(),
        is_resolved: false,
    };

    upsert_error_log(conn, &error)
}

/// Marca un error como resuelto
pub fn resolve_error(conn: &Connection, error_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE error_logs SET is_resolved = 1 WHERE id = ?1",
        rusqlite::params![error_id],
    )?;
    Ok(())
}

/// Obtiene errores activos (no resueltos) del proyecto
pub fn get_active_errors(conn: &Connection, project_path: &str) -> Result<Vec<ErrorLog>> {
    get_error_logs(conn, project_path, false)
}
