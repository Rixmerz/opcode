use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representa el tipo de chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    /// Chunk 1: Raw source code - archivo completo
    RawSource,
    /// Chunk 2: AST comprimido por archivo
    Ast,
    /// Chunk 3: Callgraph y dependencias (estático + dinámico)
    Callgraph,
    /// Chunk 4: Tests - pruebas unitarias/integrales
    Tests,
    /// Chunk 5: Commit history - documentación técnica real
    CommitHistory,
    /// Chunk 6: Estado/configuración - ENV, flags, settings
    StateConfig,
    /// Chunk 7: Metadata del proyecto - paquetes, versiones, deps
    ProjectMetadata,
    /// Chunk 8: Reglas de negocio validadas por humanos
    BusinessRules,
    /// Chunk 9: History/Snapshots - Git virtual interno
    Snapshot,
    /// Chunk 10: Errores/logs - stacktraces, crashes
    ErrorLog,
}

impl ChunkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkType::RawSource => "raw_source",
            ChunkType::Ast => "ast",
            ChunkType::Callgraph => "callgraph",
            ChunkType::Tests => "tests",
            ChunkType::CommitHistory => "commit_history",
            ChunkType::StateConfig => "state_config",
            ChunkType::ProjectMetadata => "project_metadata",
            ChunkType::BusinessRules => "business_rules",
            ChunkType::Snapshot => "snapshot",
            ChunkType::ErrorLog => "error_log",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "raw_source" => Some(ChunkType::RawSource),
            "ast" => Some(ChunkType::Ast),
            "callgraph" => Some(ChunkType::Callgraph),
            "tests" => Some(ChunkType::Tests),
            "commit_history" => Some(ChunkType::CommitHistory),
            "state_config" => Some(ChunkType::StateConfig),
            "project_metadata" => Some(ChunkType::ProjectMetadata),
            "business_rules" => Some(ChunkType::BusinessRules),
            "snapshot" => Some(ChunkType::Snapshot),
            "error_log" => Some(ChunkType::ErrorLog),
            _ => None,
        }
    }
}

/// Representa un chunk de código/información
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Option<i64>,
    pub project_path: String,
    pub chunk_type: ChunkType,
    pub file_path: Option<String>, // Path relativo al proyecto
    pub entity_name: Option<String>, // Nombre de clase/función/módulo si aplica
    pub content: String,
    pub content_hash: String, // SHA256 del contenido
    pub metadata: Option<String>, // JSON con metadata adicional
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Representa una relación entre chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRelationship {
    pub id: Option<i64>,
    pub from_chunk_id: i64,
    pub to_chunk_id: i64,
    pub relationship_type: RelationshipType,
    pub metadata: Option<String>, // JSON con metadata adicional
    pub created_at: DateTime<Utc>,
}

/// Tipos de relaciones entre chunks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Importa/depende de
    DependsOn,
    /// Llama a
    Calls,
    /// Es testeado por
    TestedBy,
    /// Implementa regla de negocio
    ImplementsRule,
    /// Modificado en el mismo commit
    ModifiedWith,
    /// Asociado con error
    AssociatedWithError,
    /// Contiene configuración para
    ConfiguresFor,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::DependsOn => "depends_on",
            RelationshipType::Calls => "calls",
            RelationshipType::TestedBy => "tested_by",
            RelationshipType::ImplementsRule => "implements_rule",
            RelationshipType::ModifiedWith => "modified_with",
            RelationshipType::AssociatedWithError => "associated_with_error",
            RelationshipType::ConfiguresFor => "configures_for",
        }
    }
}

/// Regla de negocio validada por humanos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: Option<i64>,
    pub project_path: String,
    pub entity_name: String, // Clase/función/módulo
    pub file_path: String,
    pub rule_description: String,
    pub ai_interpretation: String,
    pub user_correction: Option<String>,
    pub is_validated: bool,
    pub validation_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Snapshot del proyecto (Git real con versionado)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Option<i64>,
    pub project_path: String,
    pub snapshot_type: SnapshotType,
    pub parent_snapshot_id: Option<i64>,
    pub message: String,
    pub user_message: Option<String>, // Mensaje original del usuario (solo para master)
    pub changed_files: String,        // JSON array de archivos modificados
    pub diff_summary: Option<String>, // Resumen de cambios
    pub metadata: Option<String>,     // JSON con metadata adicional

    // Git integration fields
    pub git_commit_hash: Option<String>, // Hash del commit real de Git
    pub git_tag: Option<String>,         // Tag de versión (V1, V2, V3 o V1.1, V1.2)
    pub git_branch: Option<String>,      // Rama (main para master, agent/v1.1 para agent)
    pub version_major: i32,              // Número de versión principal (1, 2, 3...)
    pub version_minor: Option<i32>,      // Número de versión secundaria (solo para agent: 1, 2, 3...)

    pub created_at: DateTime<Utc>,
}

/// Tipo de snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotType {
    /// Snapshot en rama MASTER (user intent timeline)
    Master,
    /// Snapshot en rama AGENT (agent execution timeline)
    Agent,
}

impl SnapshotType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotType::Master => "master",
            SnapshotType::Agent => "agent",
        }
    }
}

/// Error/log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub id: Option<i64>,
    pub project_path: String,
    pub snapshot_id: Option<i64>,
    pub file_path: Option<String>,
    pub entity_name: Option<String>, // Función/método donde ocurrió
    pub error_type: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub occurrence_count: i32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_resolved: bool,
}

/// Metadata del chunk de AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstMetadata {
    pub language: String,
    pub node_count: usize,
    pub max_depth: usize,
    pub has_syntax_errors: bool,
}

/// Metadata del chunk de callgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallgraphMetadata {
    pub is_static: bool,    // true = análisis estático, false = runtime tracking
    pub entry_points: Vec<String>,
    pub external_calls: Vec<String>,
    pub call_count: usize,
}

/// Metadata del chunk de commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub commit_hash: String,
    pub author: String,
    pub author_email: String,
    pub commit_date: DateTime<Utc>,
    pub files_modified: Vec<String>,
    pub insertions: usize,
    pub deletions: usize,
}

/// Resultado de procesamiento de chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingResult {
    pub project_path: String,
    pub chunks_created: usize,
    pub chunks_updated: usize,
    pub relationships_created: usize,
    pub errors: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

/// Opciones de configuración para el chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingOptions {
    /// Tipos de chunks a generar
    pub chunk_types: Vec<ChunkType>,
    /// Máxima profundidad del AST
    pub max_ast_depth: Option<usize>,
    /// Incluir análisis de callgraph dinámico (requiere runtime tracking)
    pub include_dynamic_callgraph: bool,
    /// Número máximo de commits a analizar
    pub max_commits: Option<usize>,
    /// Patrones de archivos a ignorar
    pub ignore_patterns: Vec<String>,
}

impl Default for ChunkingOptions {
    fn default() -> Self {
        Self {
            chunk_types: vec![
                ChunkType::RawSource,
                ChunkType::Ast,
                ChunkType::Callgraph,
                ChunkType::Tests,
                ChunkType::CommitHistory,
                ChunkType::StateConfig,
                ChunkType::ProjectMetadata,
            ],
            max_ast_depth: None,
            include_dynamic_callgraph: false,
            max_commits: Some(100),
            ignore_patterns: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
                ".git/**".to_string(),
            ],
        }
    }
}

/// Query para búsqueda de chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkQuery {
    pub project_path: Option<String>,
    pub chunk_types: Option<Vec<ChunkType>>,
    pub file_path: Option<String>,
    pub entity_name: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
