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

use anyhow::{Context, Result};
use chrono::Utc;
use ignore::WalkBuilder;
use rusqlite::Connection;
use std::path::Path;

use storage::{calculate_content_hash, init_chunk_database, upsert_chunk};
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

    /// Crea un snapshot master del estado actual del proyecto
    pub fn create_user_snapshot(
        &self,
        project_path: &str,
        user_message: &str,
        _changed_files: &[String],
        _parent_snapshot_id: Option<i64>,
    ) -> Result<i64> {
        snapshots::create_master_snapshot_with_git(
            &self.conn,
            project_path,
            user_message,
        )
    }

    /// Crea un snapshot agent de un cambio realizado por el agente
    pub fn create_agent_snapshot(
        &self,
        project_path: &str,
        message: &str,
        changed_files: &[String],
        master_snapshot_id: i64,
    ) -> Result<i64> {
        snapshots::create_agent_snapshot_with_git(
            &self.conn,
            project_path,
            master_snapshot_id,
            message,
            Some(changed_files.to_vec()),
        )
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
