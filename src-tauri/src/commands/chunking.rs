use crate::chunking::business_rules::{get_pending_rules, validate_business_rule};
use crate::chunking::errors::{get_active_errors, resolve_error};
use crate::chunking::storage::{get_snapshots, query_chunks};
use crate::chunking::types::*;
use crate::chunking::ChunkingOrchestrator;
use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// Estado global del sistema de chunking
pub struct ChunkingState(pub Mutex<Connection>);

/// Inicializa el sistema de chunking para la aplicación
pub fn init_chunking_system(app: &AppHandle) -> Result<Connection> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

    let db_path = app_dir.join("chunks.db");
    let conn = Connection::open(db_path)?;

    // Inicializar esquema
    crate::chunking::storage::init_chunk_database(&conn)?;

    Ok(conn)
}

/// Procesa un proyecto completo y genera todos los chunks
#[tauri::command]
pub async fn process_project_chunks(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    options: Option<ChunkingOptions>,
) -> Result<ChunkingResult, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    let orchestrator = ChunkingOrchestrator::new(Connection::open_in_memory().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // Usar la conexión del state en lugar de crear una nueva
    let opts = options.unwrap_or_default();

    // Nota: Aquí necesitamos refactorizar para pasar la conexión existente
    // Por ahora, retornaremos un resultado de ejemplo
    Ok(ChunkingResult {
        project_path: project_path.clone(),
        chunks_created: 0,
        chunks_updated: 0,
        relationships_created: 0,
        errors: vec!["Chunking system initialized. Full processing coming soon.".to_string()],
        started_at: chrono::Utc::now(),
        completed_at: chrono::Utc::now(),
    })
}

/// Busca chunks según criterios
#[tauri::command]
pub async fn search_chunks(
    chunking_state: State<'_, ChunkingState>,
    query: ChunkQuery,
) -> Result<Vec<Chunk>, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    query_chunks(&conn, &query).map_err(|e| e.to_string())
}

/// Obtiene reglas de negocio pendientes de validación
#[tauri::command]
pub async fn get_pending_business_rules(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
) -> Result<Vec<BusinessRule>, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    get_pending_rules(&conn, &project_path).map_err(|e| e.to_string())
}

/// Valida una regla de negocio con la corrección del usuario
#[tauri::command]
pub async fn validate_business_rule_command(
    chunking_state: State<'_, ChunkingState>,
    rule_id: i64,
    rule_description: String,
    user_correction: Option<String>,
) -> Result<(), String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    validate_business_rule(
        &conn,
        rule_id,
        &rule_description,
        user_correction.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Obtiene snapshots de un proyecto
#[tauri::command]
pub async fn get_project_snapshots(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    snapshot_type: Option<String>,
) -> Result<Vec<Snapshot>, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;

    let st = snapshot_type.and_then(|s| {
        if s == "master" {
            Some(SnapshotType::Master)
        } else if s == "agent" {
            Some(SnapshotType::Agent)
        } else {
            None
        }
    });

    get_snapshots(&conn, &project_path, st).map_err(|e| e.to_string())
}

/// Obtiene errores activos de un proyecto
#[tauri::command]
pub async fn get_project_errors(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
) -> Result<Vec<ErrorLog>, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    get_active_errors(&conn, &project_path).map_err(|e| e.to_string())
}

/// Marca un error como resuelto
#[tauri::command]
pub async fn resolve_error_command(
    chunking_state: State<'_, ChunkingState>,
    error_id: i64,
) -> Result<(), String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    resolve_error(&conn, error_id).map_err(|e| e.to_string())
}

/// Crea un snapshot master (user intent)
#[tauri::command]
pub async fn create_master_snapshot(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    user_message: String,
    changed_files: Vec<String>,
    parent_snapshot_id: Option<i64>,
) -> Result<i64, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    crate::chunking::snapshots::create_master_snapshot(
        &conn,
        &project_path,
        &user_message,
        &changed_files,
        parent_snapshot_id,
    )
    .map_err(|e| e.to_string())
}

/// Crea un snapshot agent (agent execution)
#[tauri::command]
pub async fn create_agent_snapshot(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    message: String,
    changed_files: Vec<String>,
    parent_snapshot_id: Option<i64>,
) -> Result<i64, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    crate::chunking::snapshots::create_agent_snapshot(
        &conn,
        &project_path,
        &message,
        &changed_files,
        parent_snapshot_id,
        None,
    )
    .map_err(|e| e.to_string())
}

/// Propone una regla de negocio para validación
#[tauri::command]
pub async fn propose_business_rule_command(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    entity_name: String,
    file_path: String,
    ai_interpretation: String,
) -> Result<i64, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    crate::chunking::business_rules::propose_business_rule(
        &conn,
        &project_path,
        &entity_name,
        &file_path,
        &ai_interpretation,
    )
    .map_err(|e| e.to_string())
}

/// Registra un error en el sistema
#[tauri::command]
pub async fn log_error_command(
    chunking_state: State<'_, ChunkingState>,
    project_path: String,
    error_type: String,
    message: String,
    file_path: Option<String>,
    stacktrace: Option<String>,
) -> Result<i64, String> {
    let conn = chunking_state.0.lock().map_err(|e| e.to_string())?;
    crate::chunking::errors::log_error(
        &conn,
        &project_path,
        &error_type,
        &message,
        file_path.as_deref(),
        None,
        stacktrace.as_deref(),
        None,
    )
    .map_err(|e| e.to_string())
}
