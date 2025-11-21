use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{Chunk, ChunkType};
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;

/// Genera chunks de metadata del proyecto
pub fn generate_metadata_chunks(
    conn: &Connection,
    project_path: &str,
    file_path: &str,
    content: &str,
) -> Result<usize> {
    if !is_metadata_file(file_path) {
        return Ok(0);
    }

    let content_hash = calculate_content_hash(content);

    let chunk = Chunk {
        id: None,
        project_path: project_path.to_string(),
        chunk_type: ChunkType::ProjectMetadata,
        file_path: Some(file_path.to_string()),
        entity_name: None,
        content: content.to_string(),
        content_hash,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_chunk(conn, &chunk)?;
    Ok(1)
}

/// Detecta si un archivo es de metadata del proyecto
fn is_metadata_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    matches!(
        filename,
        "package.json"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "Cargo.toml"
            | "Cargo.lock"
            | "pyproject.toml"
            | "requirements.txt"
            | "Pipfile"
            | "Pipfile.lock"
            | "go.mod"
            | "go.sum"
            | "build.gradle"
            | "pom.xml"
            | "composer.json"
            | "composer.lock"
            | "Gemfile"
            | "Gemfile.lock"
    )
}
