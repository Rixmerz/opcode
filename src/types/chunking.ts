/**
 * TypeScript types for the Chunking System
 * Mirrors Rust types from src-tauri/src/chunking/types.rs
 */

export type ChunkType =
  | 'raw_source'
  | 'ast'
  | 'callgraph'
  | 'tests'
  | 'commit_history'
  | 'state_config'
  | 'project_metadata'
  | 'business_rules'
  | 'snapshot'
  | 'error_log';

export interface Chunk {
  id?: number;
  project_path: string;
  chunk_type: ChunkType;
  file_path?: string;
  entity_name?: string;
  content: string;
  content_hash: string;
  metadata?: string;
  created_at: string;
  updated_at: string;
}

export type RelationshipType =
  | 'depends_on'
  | 'calls'
  | 'tested_by'
  | 'implements_rule'
  | 'modified_with'
  | 'associated_with_error'
  | 'configures_for';

export interface ChunkRelationship {
  id?: number;
  from_chunk_id: number;
  to_chunk_id: number;
  relationship_type: RelationshipType;
  metadata?: string;
  created_at: string;
}

export interface BusinessRule {
  id?: number;
  project_path: string;
  entity_name: string;
  file_path: string;
  rule_description: string;
  ai_interpretation: string;
  user_correction?: string;
  is_validated: boolean;
  validation_date?: string;
  created_at: string;
  updated_at: string;
}

export type SnapshotType = 'master' | 'agent';

export interface Snapshot {
  id?: number;
  project_path: string;
  snapshot_type: SnapshotType;
  parent_snapshot_id?: number;
  message: string;
  user_message?: string;
  changed_files: string; // JSON array
  diff_summary?: string;
  metadata?: string;

  // Git integration fields
  git_commit_hash?: string; // Hash del commit real de Git
  git_tag?: string;         // Tag de versión (V1, V2, V3 o V1.1, V1.2)
  git_branch?: string;      // Rama (main para master, agent/v1.1 para agent)
  version_major: number;    // Número de versión principal (1, 2, 3...)
  version_minor?: number;   // Número de versión secundaria (solo para agent: 1, 2, 3...)

  created_at: string;
}

export interface ErrorLog {
  id?: number;
  project_path: string;
  snapshot_id?: number;
  file_path?: string;
  entity_name?: string;
  error_type: string;
  message: string;
  stacktrace?: string;
  occurrence_count: number;
  first_seen: string;
  last_seen: string;
  is_resolved: boolean;
}

export interface AstMetadata {
  language: string;
  node_count: number;
  max_depth: number;
  has_syntax_errors: boolean;
}

export interface CallgraphMetadata {
  is_static: boolean;
  entry_points: string[];
  external_calls: string[];
  call_count: number;
}

export interface CommitMetadata {
  commit_hash: string;
  author: string;
  author_email: string;
  commit_date: string;
  files_modified: string[];
  insertions: number;
  deletions: number;
}

export interface ChunkingResult {
  project_path: string;
  chunks_created: number;
  chunks_updated: number;
  relationships_created: number;
  errors: string[];
  started_at: string;
  completed_at: string;
}

export interface ChunkingOptions {
  chunk_types: ChunkType[];
  max_ast_depth?: number;
  include_dynamic_callgraph: boolean;
  max_commits?: number;
  ignore_patterns: string[];
}

export interface ChunkQuery {
  project_path?: string;
  chunk_types?: ChunkType[];
  file_path?: string;
  entity_name?: string;
  limit?: number;
  offset?: number;
}

// UI-specific types
export interface ChunkWithMetadata extends Chunk {
  parsedMetadata?: AstMetadata | CallgraphMetadata | CommitMetadata;
  relationships?: ChunkRelationship[];
}

export interface ChunkFilterOptions {
  chunkTypes: ChunkType[];
  searchQuery: string;
  filePath?: string;
  dateRange?: {
    start: Date;
    end: Date;
  };
}

export interface ChunkStats {
  totalChunks: number;
  chunksByType: Record<ChunkType, number>;
  lastProcessed?: string;
  processingTime?: number;
}
