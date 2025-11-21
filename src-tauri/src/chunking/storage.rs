use super::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, Result as SqliteResult};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

/// Database connection wrapper para chunks
pub struct ChunkDb(pub Mutex<Connection>);

/// Inicializa la base de datos de chunks
pub fn init_chunk_database(conn: &Connection) -> SqliteResult<()> {
    // Tabla principal de chunks
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_path TEXT NOT NULL,
            chunk_type TEXT NOT NULL,
            file_path TEXT,
            entity_name TEXT,
            content TEXT NOT NULL,
            content_hash TEXT NOT NULL UNIQUE,
            metadata TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    // Índices para búsqueda eficiente
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_project ON chunks(project_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_type ON chunks(chunk_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_file ON chunks(file_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_entity ON chunks(entity_name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_hash ON chunks(content_hash)",
        [],
    )?;

    // Tabla de relaciones entre chunks
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunk_relationships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_chunk_id INTEGER NOT NULL,
            to_chunk_id INTEGER NOT NULL,
            relationship_type TEXT NOT NULL,
            metadata TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (from_chunk_id) REFERENCES chunks(id) ON DELETE CASCADE,
            FOREIGN KEY (to_chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_relationships_from ON chunk_relationships(from_chunk_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_relationships_to ON chunk_relationships(to_chunk_id)",
        [],
    )?;

    // Tabla de reglas de negocio
    conn.execute(
        "CREATE TABLE IF NOT EXISTS business_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_path TEXT NOT NULL,
            entity_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            rule_description TEXT NOT NULL,
            ai_interpretation TEXT NOT NULL,
            user_correction TEXT,
            is_validated BOOLEAN NOT NULL DEFAULT 0,
            validation_date TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_business_rules_project ON business_rules(project_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_business_rules_entity ON business_rules(entity_name)",
        [],
    )?;

    // Tabla de snapshots (git real con versionado)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_path TEXT NOT NULL,
            snapshot_type TEXT NOT NULL,
            parent_snapshot_id INTEGER,
            message TEXT NOT NULL,
            user_message TEXT,
            changed_files TEXT NOT NULL,
            diff_summary TEXT,
            metadata TEXT,
            git_commit_hash TEXT,
            git_tag TEXT,
            git_branch TEXT,
            version_major INTEGER NOT NULL DEFAULT 1,
            version_minor INTEGER,
            created_at TEXT NOT NULL,
            FOREIGN KEY (parent_snapshot_id) REFERENCES snapshots(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // Migrations: Add Git fields if they don't exist (for existing databases)
    let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN git_commit_hash TEXT", []);
    let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN git_tag TEXT", []);
    let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN git_branch TEXT", []);
    let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN version_major INTEGER NOT NULL DEFAULT 1", []);
    let _ = conn.execute("ALTER TABLE snapshots ADD COLUMN version_minor INTEGER", []);

    // Migration: Add snapshot_id to chunks table for linking chunks with snapshots
    let _ = conn.execute("ALTER TABLE chunks ADD COLUMN snapshot_id INTEGER", []);

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_chunks_snapshot ON chunks(snapshot_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_project ON snapshots(project_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_type ON snapshots(snapshot_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_parent ON snapshots(parent_snapshot_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_version ON snapshots(version_major, version_minor)",
        [],
    )?;

    // Tabla de errores/logs
    conn.execute(
        "CREATE TABLE IF NOT EXISTS error_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_path TEXT NOT NULL,
            snapshot_id INTEGER,
            file_path TEXT,
            entity_name TEXT,
            error_type TEXT NOT NULL,
            message TEXT NOT NULL,
            stacktrace TEXT,
            occurrence_count INTEGER NOT NULL DEFAULT 1,
            first_seen TEXT NOT NULL,
            last_seen TEXT NOT NULL,
            is_resolved BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (snapshot_id) REFERENCES snapshots(id) ON DELETE SET NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_error_logs_project ON error_logs(project_path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_error_logs_snapshot ON error_logs(snapshot_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_error_logs_file ON error_logs(file_path)",
        [],
    )?;

    Ok(())
}

/// Calcula el hash SHA256 del contenido
pub fn calculate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Inserta o actualiza un chunk
/// Retorna (created: bool) - true si se creó nuevo, false si se actualizó existente
pub fn upsert_chunk(conn: &Connection, chunk: &Chunk, snapshot_id: Option<i64>) -> Result<bool> {
    let chunk_type_str = chunk.chunk_type.as_str();
    let now = Utc::now().to_rfc3339();

    // Check if chunk already exists
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM chunks WHERE content_hash = ?1",
            params![&chunk.content_hash],
            |row| row.get(0),
        )
        .ok();

    if let Some(_id) = existing {
        // Update existing chunk
        conn.execute(
            "UPDATE chunks SET updated_at = ?1, metadata = ?2, snapshot_id = ?3 WHERE content_hash = ?4",
            params![&now, &chunk.metadata, snapshot_id, &chunk.content_hash],
        )?;
        Ok(false) // Updated, not created
    } else {
        // Insert new chunk
        conn.execute(
            "INSERT INTO chunks (project_path, chunk_type, file_path, entity_name, content, content_hash, metadata, snapshot_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &chunk.project_path,
                chunk_type_str,
                &chunk.file_path,
                &chunk.entity_name,
                &chunk.content,
                &chunk.content_hash,
                &chunk.metadata,
                snapshot_id,
                &now,
                &now,
            ],
        )?;
        Ok(true) // Created new
    }
}

/// Obtiene chunks según criterios de búsqueda
pub fn query_chunks(conn: &Connection, query: &ChunkQuery) -> Result<Vec<Chunk>> {
    let mut sql = "SELECT id, project_path, chunk_type, file_path, entity_name, content, content_hash, metadata, created_at, updated_at FROM chunks WHERE 1=1".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(project_path) = &query.project_path {
        sql.push_str(" AND project_path = ?");
        params_vec.push(Box::new(project_path.clone()));
    }

    if let Some(chunk_types) = &query.chunk_types {
        let placeholders: Vec<String> = chunk_types.iter().map(|_| "?".to_string()).collect();
        sql.push_str(&format!(" AND chunk_type IN ({})", placeholders.join(",")));
        for ct in chunk_types {
            params_vec.push(Box::new(ct.as_str().to_string()));
        }
    }

    if let Some(file_path) = &query.file_path {
        sql.push_str(" AND file_path = ?");
        params_vec.push(Box::new(file_path.clone()));
    }

    if let Some(entity_name) = &query.entity_name {
        sql.push_str(" AND entity_name = ?");
        params_vec.push(Box::new(entity_name.clone()));
    }

    sql.push_str(" ORDER BY updated_at DESC");

    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = query.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let chunks = stmt
        .query_map(param_refs.as_slice(), |row| {
            let chunk_type_str: String = row.get(2)?;
            let chunk_type = ChunkType::from_str(&chunk_type_str)
                .ok_or_else(|| rusqlite::Error::InvalidQuery)?;

            let created_at_str: String = row.get(8)?;
            let updated_at_str: String = row.get(9)?;

            Ok(Chunk {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                chunk_type,
                file_path: row.get(3)?,
                entity_name: row.get(4)?,
                content: row.get(5)?,
                content_hash: row.get(6)?,
                metadata: row.get(7)?,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(chunks)
}

/// Inserta una relación entre chunks
pub fn insert_relationship(conn: &Connection, rel: &ChunkRelationship) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO chunk_relationships (from_chunk_id, to_chunk_id, relationship_type, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            rel.from_chunk_id,
            rel.to_chunk_id,
            rel.relationship_type.as_str(),
            &rel.metadata,
            &now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Obtiene relaciones de un chunk
pub fn get_relationships(
    conn: &Connection,
    chunk_id: i64,
    outgoing: bool,
) -> Result<Vec<ChunkRelationship>> {
    let sql = if outgoing {
        "SELECT id, from_chunk_id, to_chunk_id, relationship_type, metadata, created_at
         FROM chunk_relationships WHERE from_chunk_id = ?1"
    } else {
        "SELECT id, from_chunk_id, to_chunk_id, relationship_type, metadata, created_at
         FROM chunk_relationships WHERE to_chunk_id = ?1"
    };

    let mut stmt = conn.prepare(sql)?;
    let rels = stmt
        .query_map(params![chunk_id], |row| {
            let rel_type_str: String = row.get(3)?;
            let created_at_str: String = row.get(5)?;

            Ok(ChunkRelationship {
                id: Some(row.get(0)?),
                from_chunk_id: row.get(1)?,
                to_chunk_id: row.get(2)?,
                relationship_type: match rel_type_str.as_str() {
                    "depends_on" => RelationshipType::DependsOn,
                    "calls" => RelationshipType::Calls,
                    "tested_by" => RelationshipType::TestedBy,
                    "implements_rule" => RelationshipType::ImplementsRule,
                    "modified_with" => RelationshipType::ModifiedWith,
                    "associated_with_error" => RelationshipType::AssociatedWithError,
                    "configures_for" => RelationshipType::ConfiguresFor,
                    _ => RelationshipType::DependsOn,
                },
                metadata: row.get(4)?,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(rels)
}

/// Inserta o actualiza una regla de negocio
pub fn upsert_business_rule(conn: &Connection, rule: &BusinessRule) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO business_rules (project_path, entity_name, file_path, rule_description, ai_interpretation, user_correction, is_validated, validation_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(rowid) DO UPDATE SET
            rule_description = ?4,
            ai_interpretation = ?5,
            user_correction = ?6,
            is_validated = ?7,
            validation_date = ?8,
            updated_at = ?10",
        params![
            &rule.project_path,
            &rule.entity_name,
            &rule.file_path,
            &rule.rule_description,
            &rule.ai_interpretation,
            &rule.user_correction,
            rule.is_validated,
            rule.validation_date.as_ref().map(|d| d.to_rfc3339()),
            &now,
            &now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Obtiene reglas de negocio para un proyecto
pub fn get_business_rules(conn: &Connection, project_path: &str) -> Result<Vec<BusinessRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_path, entity_name, file_path, rule_description, ai_interpretation, user_correction, is_validated, validation_date, created_at, updated_at
         FROM business_rules WHERE project_path = ?1 ORDER BY entity_name",
    )?;

    let rules = stmt
        .query_map(params![project_path], |row| {
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;
            let validation_date_str: Option<String> = row.get(8)?;

            Ok(BusinessRule {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                entity_name: row.get(2)?,
                file_path: row.get(3)?,
                rule_description: row.get(4)?,
                ai_interpretation: row.get(5)?,
                user_correction: row.get(6)?,
                is_validated: row.get(7)?,
                validation_date: validation_date_str.and_then(|s| s.parse().ok()),
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(rules)
}

/// Crea un snapshot con información Git
pub fn create_snapshot(conn: &Connection, snapshot: &Snapshot) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO snapshots (project_path, snapshot_type, parent_snapshot_id, message, user_message, changed_files, diff_summary, metadata, git_commit_hash, git_tag, git_branch, version_major, version_minor, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            &snapshot.project_path,
            snapshot.snapshot_type.as_str(),
            snapshot.parent_snapshot_id,
            &snapshot.message,
            &snapshot.user_message,
            &snapshot.changed_files,
            &snapshot.diff_summary,
            &snapshot.metadata,
            &snapshot.git_commit_hash,
            &snapshot.git_tag,
            &snapshot.git_branch,
            snapshot.version_major,
            snapshot.version_minor,
            &now,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Obtiene snapshots de un proyecto
pub fn get_snapshots(
    conn: &Connection,
    project_path: &str,
    snapshot_type: Option<SnapshotType>,
) -> Result<Vec<Snapshot>> {
    let (sql, has_type_filter) = if let Some(st) = snapshot_type {
        (
            "SELECT id, project_path, snapshot_type, parent_snapshot_id, message, user_message, changed_files, diff_summary, metadata, git_commit_hash, git_tag, git_branch, version_major, version_minor, created_at
             FROM snapshots WHERE project_path = ?1 AND snapshot_type = ?2 ORDER BY version_major DESC, version_minor DESC, created_at DESC",
            Some(st),
        )
    } else {
        (
            "SELECT id, project_path, snapshot_type, parent_snapshot_id, message, user_message, changed_files, diff_summary, metadata, git_commit_hash, git_tag, git_branch, version_major, version_minor, created_at
             FROM snapshots WHERE project_path = ?1 ORDER BY version_major DESC, version_minor DESC, created_at DESC",
            None,
        )
    };

    let mut stmt = conn.prepare(sql)?;
    let snapshots = if let Some(st) = has_type_filter {
        stmt.query_map(params![project_path, st.as_str()], |row| {
            parse_snapshot_row(row)
        })?
        .collect::<SqliteResult<Vec<_>>>()?
    } else {
        stmt.query_map(params![project_path], |row| parse_snapshot_row(row))?
            .collect::<SqliteResult<Vec<_>>>()?
    };

    Ok(snapshots)
}

fn parse_snapshot_row(row: &rusqlite::Row) -> SqliteResult<Snapshot> {
    let snapshot_type_str: String = row.get(2)?;
    let created_at_str: String = row.get(14)?;

    Ok(Snapshot {
        id: Some(row.get(0)?),
        project_path: row.get(1)?,
        snapshot_type: if snapshot_type_str == "master" {
            SnapshotType::Master
        } else {
            SnapshotType::Agent
        },
        parent_snapshot_id: row.get(3)?,
        message: row.get(4)?,
        user_message: row.get(5)?,
        changed_files: row.get(6)?,
        diff_summary: row.get(7)?,
        metadata: row.get(8)?,
        git_commit_hash: row.get(9)?,
        git_tag: row.get(10)?,
        git_branch: row.get(11)?,
        version_major: row.get(12)?,
        version_minor: row.get(13)?,
        created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
    })
}

/// Inserta o actualiza un error log
pub fn upsert_error_log(conn: &Connection, error: &ErrorLog) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    // Intentar encontrar error similar existente
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM error_logs WHERE project_path = ?1 AND error_type = ?2 AND message = ?3 AND is_resolved = 0",
            params![&error.project_path, &error.error_type, &error.message],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Actualizar contador y last_seen
        conn.execute(
            "UPDATE error_logs SET occurrence_count = occurrence_count + 1, last_seen = ?1 WHERE id = ?2",
            params![&now, id],
        )?;
        Ok(id)
    } else {
        // Insertar nuevo error
        conn.execute(
            "INSERT INTO error_logs (project_path, snapshot_id, file_path, entity_name, error_type, message, stacktrace, occurrence_count, first_seen, last_seen, is_resolved)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &error.project_path,
                error.snapshot_id,
                &error.file_path,
                &error.entity_name,
                &error.error_type,
                &error.message,
                &error.stacktrace,
                1,
                &now,
                &now,
                false,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// Obtiene error logs de un proyecto
pub fn get_error_logs(conn: &Connection, project_path: &str, include_resolved: bool) -> Result<Vec<ErrorLog>> {
    let sql = if include_resolved {
        "SELECT id, project_path, snapshot_id, file_path, entity_name, error_type, message, stacktrace, occurrence_count, first_seen, last_seen, is_resolved
         FROM error_logs WHERE project_path = ?1 ORDER BY last_seen DESC"
    } else {
        "SELECT id, project_path, snapshot_id, file_path, entity_name, error_type, message, stacktrace, occurrence_count, first_seen, last_seen, is_resolved
         FROM error_logs WHERE project_path = ?1 AND is_resolved = 0 ORDER BY last_seen DESC"
    };

    let mut stmt = conn.prepare(sql)?;
    let errors = stmt
        .query_map(params![project_path], |row| {
            let first_seen_str: String = row.get(9)?;
            let last_seen_str: String = row.get(10)?;

            Ok(ErrorLog {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                snapshot_id: row.get(2)?,
                file_path: row.get(3)?,
                entity_name: row.get(4)?,
                error_type: row.get(5)?,
                message: row.get(6)?,
                stacktrace: row.get(7)?,
                occurrence_count: row.get(8)?,
                first_seen: first_seen_str.parse().unwrap_or_else(|_| Utc::now()),
                last_seen: last_seen_str.parse().unwrap_or_else(|_| Utc::now()),
                is_resolved: row.get(11)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(errors)
}

/// Elimina todos los chunks de un proyecto
pub fn delete_project_chunks(conn: &Connection, project_path: &str) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM chunks WHERE project_path = ?1",
        params![project_path],
    )?;
    Ok(count)
}
