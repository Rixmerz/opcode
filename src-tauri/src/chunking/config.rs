use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{Chunk, ChunkType};
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;

/// Genera chunks de configuración/estado
pub fn generate_config_chunks(
    conn: &Connection,
    project_path: &str,
    file_path: &str,
    content: &str,
) -> Result<usize> {
    if !is_config_file(file_path) {
        return Ok(0);
    }

    let content_hash = calculate_content_hash(content);

    let chunk = Chunk {
        id: None,
        project_path: project_path.to_string(),
        chunk_type: ChunkType::StateConfig,
        file_path: Some(file_path.to_string()),
        entity_name: None,
        content: content.to_string(),
        content_hash,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_chunk(conn, &chunk, None)?;
    Ok(1)
}

/// Detecta si un archivo es de configuración
fn is_config_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    matches!(
        filename,
        ".env"
            | ".env.local"
            | ".env.development"
            | ".env.production"
            | "config.json"
            | "config.yaml"
            | "config.yml"
            | "settings.json"
            | "settings.yaml"
            | "appsettings.json"
    ) || file_path.ends_with(".config.js")
        || file_path.ends_with(".config.ts")
        || file_path.ends_with("rc.json")
}
