pub mod ast;
pub mod business_rules;
pub mod callgraph;
pub mod commits;
pub mod config;
pub mod errors;
pub mod metadata;
pub mod raw_source;
pub mod snapshots;
pub mod storage;
pub mod tests;
pub mod types;

use anyhow::Result;
use chrono::Utc;
use ignore::WalkBuilder;
use rusqlite::Connection;
use std::path::Path;

use storage::init_chunk_database;
use types::{ChunkingOptions, ChunkingResult, ChunkType};

/// Orquestador principal del sistema de chunking
pub struct ChunkingOrchestrator {
    pub conn: Connection,
}

impl ChunkingOrchestrator {
    /// Crea una nueva instancia del orquestador
    pub fn new(conn: Connection) -> Result<Self> {
        init_chunk_database(&conn)?;
        Ok(Self { conn })
    }

    /// Procesa un proyecto completo generando todos los tipos de chunks configurados
    pub fn process_project(
        &self,
        project_path: &str,
        options: &ChunkingOptions,
    ) -> Result<ChunkingResult> {
        let started_at = Utc::now();
        let mut chunks_created = 0;
        let mut chunks_updated = 0;
        let mut relationships_created = 0;
        let mut errors = Vec::new();

        // 1. Raw Source Chunks
        if options.chunk_types.contains(&ChunkType::RawSource) {
            match raw_source::generate_raw_source_chunks(
                &self.conn,
                project_path,
                &options.ignore_patterns,
            ) {
                Ok(count) => {
                    chunks_created += count;
                    log::info!("Created {} raw source chunks", count);
                }
                Err(e) => {
                    let err_msg = format!("Failed to generate raw source chunks: {}", e);
                    log::error!("{}", err_msg);
                    errors.push(err_msg);
                }
            }
        }

        // 2. AST Chunks + 3. Callgraph + 4. Tests + 5. Config + 6. Metadata
        // Los procesamos en un solo pass del filesystem
        let walker = WalkBuilder::new(project_path)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .hidden(false)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let rel_path = match path.strip_prefix(project_path) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            // Leer contenido una sola vez
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // AST Chunks
            if options.chunk_types.contains(&ChunkType::Ast) {
                if let Err(e) = ast::generate_ast_chunks(&self.conn, project_path, &rel_path, &content)
                {
                    log::debug!("Skipped AST for {}: {}", rel_path, e);
                } else {
                    chunks_created += 1;
                }
            }

            // Callgraph Chunks
            if options.chunk_types.contains(&ChunkType::Callgraph) {
                if let Err(e) =
                    callgraph::generate_callgraph_chunks(&self.conn, project_path, &rel_path, &content)
                {
                    log::debug!("Skipped callgraph for {}: {}", rel_path, e);
                } else {
                    chunks_created += 1;
                }
            }

            // Test Chunks
            if options.chunk_types.contains(&ChunkType::Tests) {
                match tests::generate_test_chunks(&self.conn, project_path, &rel_path, &content) {
                    Ok(count) => chunks_created += count,
                    Err(e) => log::debug!("Skipped tests for {}: {}", rel_path, e),
                }
            }

            // Config Chunks
            if options.chunk_types.contains(&ChunkType::StateConfig) {
                match config::generate_config_chunks(&self.conn, project_path, &rel_path, &content) {
                    Ok(count) => chunks_created += count,
                    Err(e) => log::debug!("Skipped config for {}: {}", rel_path, e),
                }
            }

            // Metadata Chunks
            if options.chunk_types.contains(&ChunkType::ProjectMetadata) {
                match metadata::generate_metadata_chunks(&self.conn, project_path, &rel_path, &content)
                {
                    Ok(count) => chunks_created += count,
                    Err(e) => log::debug!("Skipped metadata for {}: {}", rel_path, e),
                }
            }
        }

        // 5. Commit History Chunks
        if options.chunk_types.contains(&ChunkType::CommitHistory) {
            match commits::generate_commit_chunks(&self.conn, project_path, options.max_commits) {
                Ok(count) => {
                    chunks_created += count;
                    log::info!("Created {} commit history chunks", count);
                }
                Err(e) => {
                    let err_msg = format!("Failed to generate commit chunks: {}", e);
                    log::warn!("{}", err_msg);
                    errors.push(err_msg);
                }
            }
        }

        let completed_at = Utc::now();

        Ok(ChunkingResult {
            project_path: project_path.to_string(),
            chunks_created,
            chunks_updated,
            relationships_created,
            errors,
            started_at,
            completed_at,
        })
    }

    /// Reindexación incremental: solo procesa los archivos modificados
    /// Se ejecuta automáticamente después de crear snapshots
    pub fn reindex_changed_files(
        &self,
        project_path: &str,
        changed_files: &[String],
        snapshot_id: Option<i64>,
    ) -> Result<ChunkingResult> {
        let started_at = Utc::now();
        let mut chunks_created = 0;
        let mut chunks_updated = 0;
        let mut relationships_created = 0;
        let mut errors = Vec::new();

        println!(
            "[Chunking] Incremental reindex: {} files changed in project {}",
            changed_files.len(),
            project_path
        );

        // Procesar solo los archivos que cambiaron
        for file_path in changed_files {
            let full_path = Path::new(project_path).join(file_path);

            // Skip if file doesn't exist (deleted files)
            if !full_path.exists() {
                println!("[Chunking] Skipping deleted file: {}", file_path);
                continue;
            }

            // Read file content
            match std::fs::read_to_string(&full_path) {
                Ok(content) => {
                    // Generate all chunk types for this file
                    // RawSource chunk
                    if let Ok(chunk) = raw_source::create_raw_source_chunk(&full_path, &content) {
                        match storage::upsert_chunk(&self.conn, &chunk, snapshot_id) {
                            Ok(created) => {
                                if created {
                                    chunks_created += 1;
                                } else {
                                    chunks_updated += 1;
                                }
                            }
                            Err(e) => errors.push(e.to_string()),
                        }
                    }

                    // AST chunks
                    if let Ok(ast_chunks) = ast::create_ast_chunks(&full_path, &content) {
                        for chunk in ast_chunks {
                            match storage::upsert_chunk(&self.conn, &chunk, snapshot_id) {
                                Ok(created) => {
                                    if created {
                                        chunks_created += 1;
                                    } else {
                                        chunks_updated += 1;
                                    }
                                }
                                Err(e) => errors.push(e.to_string()),
                            }
                        }
                    }

                    // Other chunk types as needed...
                }
                Err(e) => {
                    errors.push(format!("Failed to read {}: {}", file_path, e));
                }
            }
        }

        let completed_at = Utc::now();

        println!(
            "[Chunking] Incremental reindex completed: {} created, {} updated, {} errors",
            chunks_created, chunks_updated, errors.len()
        );

        Ok(ChunkingResult {
            project_path: project_path.to_string(),
            chunks_created,
            chunks_updated,
            relationships_created,
            errors,
            started_at,
            completed_at,
        })
    }

    /// Crea un snapshot master del estado actual del proyecto
    /// Automáticamente reindexingdex los archivos modificados
    pub fn create_user_snapshot(
        &self,
        project_path: &str,
        user_message: &str,
        _changed_files: &[String],
        _parent_snapshot_id: Option<i64>,
    ) -> Result<i64> {
        // Crear snapshot y obtener archivos modificados desde Git
        let snapshot_id = snapshots::create_master_snapshot_with_git(
            &self.conn,
            project_path,
            user_message,
        )?;

        // Obtener archivos modificados del snapshot recién creado
        let changed_files: Vec<String> = self.conn
            .query_row(
                "SELECT changed_files FROM snapshots WHERE id = ?1",
                rusqlite::params![snapshot_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        // Reindexar archivos modificados automáticamente
        if !changed_files.is_empty() {
            println!(
                "[Chunking] Auto-reindexing {} changed files after master snapshot",
                changed_files.len()
            );

            match self.reindex_changed_files(project_path, &changed_files, Some(snapshot_id)) {
                Ok(result) => {
                    println!(
                        "[Chunking] Auto-reindex complete: {} created, {} updated",
                        result.chunks_created, result.chunks_updated
                    );
                }
                Err(e) => {
                    log::warn!("Auto-reindex failed: {}", e);
                }
            }
        }

        Ok(snapshot_id)
    }

    /// Crea un snapshot agent de un cambio realizado por el agente
    /// Automáticamente reindexingdex los archivos modificados
    pub fn create_agent_snapshot(
        &self,
        project_path: &str,
        message: &str,
        changed_files: &[String],
        master_snapshot_id: i64,
    ) -> Result<i64> {
        // Crear snapshot agent
        let snapshot_id = snapshots::create_agent_snapshot_with_git(
            &self.conn,
            project_path,
            master_snapshot_id,
            message,
            Some(changed_files.to_vec()),
        )?;

        // Reindexar archivos modificados automáticamente
        if !changed_files.is_empty() {
            println!(
                "[Chunking] Auto-reindexing {} changed files after agent snapshot",
                changed_files.len()
            );

            match self.reindex_changed_files(project_path, changed_files, Some(snapshot_id)) {
                Ok(result) => {
                    println!(
                        "[Chunking] Auto-reindex complete: {} created, {} updated",
                        result.chunks_created, result.chunks_updated
                    );
                }
                Err(e) => {
                    log::warn!("Auto-reindex failed: {}", e);
                }
            }
        }

        Ok(snapshot_id)
    }

    /// Propone una regla de negocio para validación del usuario
    pub fn propose_business_rule(
        &self,
        project_path: &str,
        entity_name: &str,
        file_path: &str,
        ai_interpretation: &str,
    ) -> Result<i64> {
        business_rules::propose_business_rule(
            &self.conn,
            project_path,
            entity_name,
            file_path,
            ai_interpretation,
        )
    }

    /// Registra un error en el sistema
    pub fn log_error(
        &self,
        project_path: &str,
        error_type: &str,
        message: &str,
        file_path: Option<&str>,
        stacktrace: Option<&str>,
    ) -> Result<i64> {
        errors::log_error(
            &self.conn,
            project_path,
            error_type,
            message,
            file_path,
            None,
            stacktrace,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_chunking_orchestrator_creation() {
        let conn = Connection::open_in_memory().unwrap();
        let orchestrator = ChunkingOrchestrator::new(conn);
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_default_chunking_options() {
        let options = ChunkingOptions::default();
        assert!(options.chunk_types.contains(&ChunkType::RawSource));
        assert!(options.chunk_types.contains(&ChunkType::Ast));
        assert_eq!(options.max_commits, Some(100));
    }
}
