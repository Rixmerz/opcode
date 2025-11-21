# Sistema de Chunking - Documentación Técnica

## Visión General

Sistema completo de análisis y chunking de proyectos implementado para opcode. Este sistema procesa automáticamente proyectos al abrirlos, generando 10 tipos diferentes de chunks que representan diferentes aspectos del código y su contexto.

## Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│  Frontend (React/TypeScript)                             │
│  • ChunkExplorer - Visualizar chunks                    │
│  • BusinessRuleEditor - Confirmar reglas de negocio     │
│  • TimelineNavigator - Navegación temporal              │
└──────────────────┬──────────────────────────────────────┘
                   │ Tauri IPC
┌──────────────────▼──────────────────────────────────────┐
│  Backend (Rust) - src-tauri/src/chunking/               │
│  ┌────────────────────────────────────────────────────┐ │
│  │ mod.rs - ChunkingOrchestrator (orquestador)        │ │
│  │ types.rs - Tipos de datos y estructuras            │ │
│  │ storage.rs - Persistencia SQLite                   │ │
│  ├────────────────────────────────────────────────────┤ │
│  │ Generadores de Chunks:                             │ │
│  │  • raw_source.rs - Archivos completos (RAW)        │ │
│  │  • ast.rs - AST comprimido (tree-sitter)           │ │
│  │  • callgraph.rs - Dependencias y llamadas          │ │
│  │  • tests.rs - Tests unitarios/integrales           │ │
│  │  • commits.rs - Historia de commits (git2)         │ │
│  │  • config.rs - Archivos de configuración           │ │
│  │  • metadata.rs - Manifiestos y deps                │ │
│  │  • business_rules.rs - Reglas de negocio           │ │
│  │  • snapshots.rs - Git virtual interno              │ │
│  │  • errors.rs - Errores y logs                      │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│  SQLite Database (chunks.db)                             │
│  • chunks - Tabla principal de chunks                   │
│  • chunk_relationships - Relaciones entre chunks        │
│  • business_rules - Reglas validadas por humanos       │
│  • snapshots - Timeline master/agent                    │
│  • error_logs - Errores y crashes                       │
└─────────────────────────────────────────────────────────┘
```

## 10 Tipos de Chunks Implementados

### 1. Raw Source (Código Fuente Completo)
**Ubicación:** `raw_source.rs`

- Contiene el archivo completo, no fragmentado
- Máxima fidelidad para mejor grounding
- Permite reconstrucción exacta del contexto
- Soporta múltiples lenguajes de programación

**Características:**
- Respeta .gitignore
- Filtra por extensiones de código conocidas
- Calcula hash SHA256 para detectar cambios

### 2. AST (Abstract Syntax Tree)
**Ubicación:** `ast.rs`

- AST comprimido por archivo usando tree-sitter
- Útil para análisis estructural y refactoring seguro
- Incluye metadata: nodos, profundidad, errores de sintaxis

**Lenguajes soportados:**
- Rust
- JavaScript/JSX
- TypeScript/TSX
- Python

### 3. Callgraph / Dependencias
**Ubicación:** `callgraph.rs`

- Análisis estático de imports/requires
- Extrae llamadas a funciones
- Identifica dependencias externas
- Basis para análisis de impacto de cambios

**Captura:**
- Imports/use statements
- Requires (Node.js)
- Llamadas a funciones
- Dependencias externas

### 4. Tests
**Ubicación:** `tests.rs`

- Extrae pruebas unitarias e integrales
- Identifica expectations y assertions
- Documenta flujos de testing
- Captura reglas de negocio implícitas

**Detecta:**
- Funciones de test por convención
- Assertions y expectations
- Describe/it blocks (JS/TS)
- #[test] annotations (Rust)
- def test_ functions (Python)

### 5. Commit History (Documentación Técnica Real)
**Ubicación:** `commits.rs`

- Reemplaza README/ADRs inexistentes
- Historia completa de cambios usando git2
- Documenta evolución del sistema

**Información capturada:**
- Mensaje de commit
- Autor y fecha
- Archivos modificados
- Hash del commit

### 6. State / Configuration
**Ubicación:** `config.rs`

- ENV files, flags, settings
- Feature toggles
- Configuraciones de runtime
- Ayuda a entender comportamiento variable

**Archivos reconocidos:**
- .env y variantes
- config.json/yaml
- settings files
- *.config.js/ts

### 7. Project Metadata
**Ubicación:** `metadata.rs`

- Package managers manifests
- Versiones de dependencias
- Build configurations

**Archivos reconocidos:**
- package.json (Node.js)
- Cargo.toml (Rust)
- pyproject.toml (Python)
- go.mod (Go)
- pom.xml (Java)
- Y más...

### 8. Business Rules (Interactivas)
**Ubicación:** `business_rules.rs`

**Proceso interactivo:**
1. IA lee clase/método/función
2. Deduce reglas de negocio
3. Pregunta al usuario si es correcta
4. Usuario corrige/valida
5. IA reformula
6. Cuando usuario confirma → chunk de negocio

**Características:**
- No se genera automáticamente
- Requiere validación humana
- "Verdades fundamentales" del dominio
- Enlazadas a entidades específicas

### 9. History / Snapshots (Git Virtual Interno)
**Ubicación:** `snapshots.rs`

**Dos ramas paralelas:**

**Rama MASTER (User Intent Timeline):**
- Cada mensaje del usuario → snapshot maestro
- Retroceso permite reescribir historia
- Navegación temporal del proyecto

**Rama AGENT (Agent Execution Timeline):**
- Cada cambio del agente → commit paralelo
- Memoria técnica del agente
- Qué intentó, qué falló, qué se descartó

### 10. Errors / Logs
**Ubicación:** `errors.rs`

- Stacktraces y crashes
- Errores recurrentes
- Enlazados a snapshots, commits, archivos
- Tracking de ocurrencias
- Estado de resolución

**Relaciones:**
- Con callgraph
- Con historia
- Con reglas de negocio
- Con archivos específicos

## Base de Datos

### Tabla: chunks
```sql
CREATE TABLE chunks (
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
);
```

### Tabla: chunk_relationships
```sql
CREATE TABLE chunk_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_chunk_id INTEGER NOT NULL,
    to_chunk_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL,
    metadata TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (from_chunk_id) REFERENCES chunks(id),
    FOREIGN KEY (to_chunk_id) REFERENCES chunks(id)
);
```

**Tipos de relaciones:**
- `depends_on` - Depende de
- `calls` - Llama a
- `tested_by` - Es testeado por
- `implements_rule` - Implementa regla
- `modified_with` - Modificado en mismo commit
- `associated_with_error` - Asociado con error
- `configures_for` - Configura para

### Tabla: business_rules
```sql
CREATE TABLE business_rules (
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
);
```

### Tabla: snapshots
```sql
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    snapshot_type TEXT NOT NULL,  -- 'master' o 'agent'
    parent_snapshot_id INTEGER,
    message TEXT NOT NULL,
    user_message TEXT,
    changed_files TEXT NOT NULL,  -- JSON array
    diff_summary TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (parent_snapshot_id) REFERENCES snapshots(id)
);
```

### Tabla: error_logs
```sql
CREATE TABLE error_logs (
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
    FOREIGN KEY (snapshot_id) REFERENCES snapshots(id)
);
```

## Comandos Tauri Disponibles

### Procesamiento de Proyecto
```rust
process_project_chunks(project_path: String, options: Option<ChunkingOptions>) -> ChunkingResult
```

### Búsqueda de Chunks
```rust
search_chunks(query: ChunkQuery) -> Vec<Chunk>
```

### Reglas de Negocio
```rust
get_pending_business_rules(project_path: String) -> Vec<BusinessRule>
validate_business_rule_command(rule_id: i64, rule_description: String, user_correction: Option<String>) -> ()
propose_business_rule_command(project_path: String, entity_name: String, file_path: String, ai_interpretation: String) -> i64
```

### Snapshots
```rust
get_project_snapshots(project_path: String, snapshot_type: Option<String>) -> Vec<Snapshot>
create_master_snapshot(project_path: String, user_message: String, changed_files: Vec<String>, parent_snapshot_id: Option<i64>) -> i64
create_agent_snapshot(project_path: String, message: String, changed_files: Vec<String>, parent_snapshot_id: Option<i64>) -> i64
```

### Errores
```rust
get_project_errors(project_path: String) -> Vec<ErrorLog>
log_error_command(project_path: String, error_type: String, message: String, file_path: Option<String>, stacktrace: Option<String>) -> i64
resolve_error_command(error_id: i64) -> ()
```

## Dependencias Agregadas

```toml
[dependencies]
# Chunking system dependencies
tree-sitter = "0.22"
tree-sitter-rust = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-python = "0.21"
git2 = "0.19"
ignore = "0.4"
sha2 = "0.10"  # Ya existía
```

## Uso del Sistema

### 1. Procesamiento Inicial al Abrir Proyecto

```typescript
// Llamada desde el frontend
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('process_project_chunks', {
  projectPath: '/ruta/al/proyecto',
  options: {
    chunk_types: [
      'raw_source',
      'ast',
      'callgraph',
      'tests',
      'commit_history',
      'state_config',
      'project_metadata'
    ],
    max_commits: 100,
    include_dynamic_callgraph: false,
    ignore_patterns: ['node_modules/**', 'target/**', 'dist/**']
  }
});

console.log(`Chunks creados: ${result.chunks_created}`);
```

### 2. Búsqueda de Chunks

```typescript
const chunks = await invoke('search_chunks', {
  query: {
    project_path: '/ruta/al/proyecto',
    chunk_types: ['raw_source'],
    file_path: 'src/main.rs',
    limit: 10
  }
});
```

### 3. Proponer y Validar Reglas de Negocio

```typescript
// Proponer regla
const ruleId = await invoke('propose_business_rule_command', {
  projectPath: '/ruta/al/proyecto',
  entityName: 'UserAuth::login',
  filePath: 'src/auth.rs',
  aiInterpretation: 'El login requiere email válido y contraseña de mínimo 8 caracteres'
});

// Usuario valida/corrige
await invoke('validate_business_rule_command', {
  ruleId: ruleId,
  ruleDescription: 'El login requiere email válido y contraseña de mínimo 12 caracteres',
  userCorrection: 'La longitud mínima es 12, no 8'
});
```

### 4. Crear Snapshots

```typescript
// Snapshot del usuario
const snapshotId = await invoke('create_master_snapshot', {
  projectPath: '/ruta/al/proyecto',
  userMessage: 'Implementar autenticación OAuth',
  changedFiles: ['src/auth.rs', 'src/oauth.rs'],
  parentSnapshotId: null
});

// Snapshot del agente
await invoke('create_agent_snapshot', {
  projectPath: '/ruta/al/proyecto',
  message: 'Intentó refactorizar la función login() - fallido por tests',
  changedFiles: ['src/auth.rs'],
  parentSnapshotId: snapshotId
});
```

### 5. Gestión de Errores

```typescript
// Registrar error
const errorId = await invoke('log_error_command', {
  projectPath: '/ruta/al/proyecto',
  errorType: 'NullPointerException',
  message: 'Cannot read property "user" of undefined',
  filePath: 'src/components/UserProfile.tsx',
  stacktrace: '...'
});

// Obtener errores activos
const errors = await invoke('get_project_errors', {
  projectPath: '/ruta/al/proyecto'
});

// Marcar como resuelto
await invoke('resolve_error_command', { errorId });
```

## Próximos Pasos

### Frontend (Pendiente de Implementar)

1. **ChunkExplorer Component**
   - Visualizar chunks por tipo
   - Búsqueda y filtrado
   - Vista de relaciones entre chunks

2. **BusinessRuleEditor Component**
   - Interfaz para validar reglas propuestas
   - Editar y corregir interpretaciones de IA
   - Historial de reglas validadas

3. **TimelineNavigator Component**
   - Visualización de snapshots master y agent
   - Navegación temporal
   - Comparación de snapshots

4. **ErrorDashboard Component**
   - Lista de errores activos
   - Tracking de ocurrencias
   - Vinculación con código fuente

### Mejoras Técnicas

1. **Análisis de Callgraph Dinámico**
   - Runtime tracking mediante instrumentación
   - Mapeo de llamadas reales vs estáticas

2. **Optimización de Índices**
   - Full-text search en contenido de chunks
   - Índices compuestos para queries complejas

3. **Compresión de Contenido**
   - Compresión zstd para chunks grandes
   - Delta encoding para versiones

4. **Integración con Claude Code**
   - Auto-procesamiento al abrir proyecto
   - Sugerencias inteligentes basadas en chunks
   - Context injection automático

## Beneficios

1. **Máxima Fidelidad**: Chunks de raw source mantienen el código completo
2. **Análisis Profundo**: AST y callgraph permiten entender estructura
3. **Documentación Automática**: Commit history como documentación técnica
4. **Validación Humana**: Business rules aseguran correctitud del dominio
5. **Navegación Temporal**: Snapshots permiten explorar evolución
6. **Detección de Patrones**: Error logs identifican problemas recurrentes
7. **Context Completo**: Múltiples vistas del mismo código

## Archivos Creados/Modificados

### Nuevos Módulos Rust
- `src-tauri/src/chunking/mod.rs`
- `src-tauri/src/chunking/types.rs`
- `src-tauri/src/chunking/storage.rs`
- `src-tauri/src/chunking/raw_source.rs`
- `src-tauri/src/chunking/ast.rs`
- `src-tauri/src/chunking/callgraph.rs`
- `src-tauri/src/chunking/tests.rs`
- `src-tauri/src/chunking/commits.rs`
- `src-tauri/src/chunking/config.rs`
- `src-tauri/src/chunking/metadata.rs`
- `src-tauri/src/chunking/business_rules.rs`
- `src-tauri/src/chunking/snapshots.rs`
- `src-tauri/src/chunking/errors.rs`
- `src-tauri/src/commands/chunking.rs`

### Archivos Modificados
- `src-tauri/Cargo.toml` - Nuevas dependencias
- `src-tauri/src/lib.rs` - Módulo chunking
- `src-tauri/src/commands/mod.rs` - Módulo chunking commands
- `src-tauri/src/main.rs` - Inicialización y comandos Tauri

## Estado de Implementación

✅ **Completado:**
- Arquitectura del sistema
- 10 tipos de chunks implementados
- Base de datos SQLite con esquema completo
- Comandos Tauri para todas las operaciones
- Integración con la aplicación principal
- Sistema de relaciones entre chunks
- Sistema de snapshots (git virtual)
- Sistema de business rules interactivas
- Sistema de error logging

⏳ **Pendiente:**
- Componentes React para visualización
- Integración automática al abrir proyecto
- Tests unitarios del sistema
- Callgraph dinámico (runtime tracking)
- Documentación de API para frontend
