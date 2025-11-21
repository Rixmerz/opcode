use super::storage::create_snapshot;
use super::types::{Snapshot, SnapshotType};
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

/// Crea un snapshot MASTER (user intent timeline)
pub fn create_master_snapshot(
    conn: &Connection,
    project_path: &str,
    user_message: &str,
    changed_files: &[String],
    parent_snapshot_id: Option<i64>,
) -> Result<i64> {
    let snapshot = Snapshot {
        id: None,
        project_path: project_path.to_string(),
        snapshot_type: SnapshotType::Master,
        parent_snapshot_id,
        message: format!("User intent: {}", user_message),
        user_message: Some(user_message.to_string()),
        changed_files: serde_json::to_string(&changed_files)?,
        diff_summary: None,
        metadata: None,
        created_at: Utc::now(),
    };

    create_snapshot(conn, &snapshot)
}

/// Crea un snapshot AGENT (agent execution timeline)
pub fn create_agent_snapshot(
    conn: &Connection,
    project_path: &str,
    message: &str,
    changed_files: &[String],
    parent_snapshot_id: Option<i64>,
    metadata: Option<&str>,
) -> Result<i64> {
    let snapshot = Snapshot {
        id: None,
        project_path: project_path.to_string(),
        snapshot_type: SnapshotType::Agent,
        parent_snapshot_id,
        message: message.to_string(),
        user_message: None,
        changed_files: serde_json::to_string(&changed_files)?,
        diff_summary: None,
        metadata: metadata.map(|s| s.to_string()),
        created_at: Utc::now(),
    };

    create_snapshot(conn, &snapshot)
}

/// Reescribe la rama master retrocediendo a un snapshot anterior
pub fn rewind_master_to_snapshot(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<()> {
    // Eliminar todos los snapshots master posteriores al snapshot especificado
    conn.execute(
        "DELETE FROM snapshots WHERE snapshot_type = 'master' AND id > ?1",
        rusqlite::params![snapshot_id],
    )?;
    Ok(())
}
