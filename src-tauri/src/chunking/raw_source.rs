use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{Chunk, ChunkType};
use anyhow::Result;
use chrono::Utc;
use ignore::WalkBuilder;
use rusqlite::Connection;
use std::path::Path;

/// Genera chunks de código fuente RAW (archivo completo)
pub fn generate_raw_source_chunks(
    conn: &Connection,
    project_path: &str,
    ignore_patterns: &[String],
) -> Result<usize> {
    let mut chunks_created = 0;

    // Construir walker que respeta .gitignore
    let walker = WalkBuilder::new(project_path)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .hidden(false)
        .build();

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        // Solo procesar archivos
        if !path.is_file() {
            continue;
        }

        // Verificar que es un archivo de código
        if !is_code_file(path) {
            continue;
        }

        // Verificar patrones de ignore personalizados
        let rel_path = path
            .strip_prefix(project_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if should_ignore(&rel_path, ignore_patterns) {
            continue;
        }

        // Leer contenido del archivo
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let content_hash = calculate_content_hash(&content);

                let chunk = Chunk {
                    id: None,
                    project_path: project_path.to_string(),
                    chunk_type: ChunkType::RawSource,
                    file_path: Some(rel_path),
                    entity_name: None,
                    content,
                    content_hash,
                    metadata: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                match upsert_chunk(conn, &chunk) {
                    Ok(_) => chunks_created += 1,
                    Err(e) => {
                        eprintln!("Failed to insert chunk for {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read file {}: {}", path.display(), e);
            }
        }
    }

    Ok(chunks_created)
}

/// Verifica si un archivo es un archivo de código
fn is_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(
            ext_str.as_str(),
            "rs" | "js"
                | "jsx"
                | "ts"
                | "tsx"
                | "py"
                | "java"
                | "cpp"
                | "c"
                | "h"
                | "hpp"
                | "cs"
                | "go"
                | "rb"
                | "php"
                | "swift"
                | "kt"
                | "scala"
                | "r"
                | "m"
                | "mm"
                | "vue"
                | "svelte"
                | "dart"
                | "lua"
                | "sh"
                | "bash"
                | "zsh"
                | "fish"
                | "sql"
                | "graphql"
                | "proto"
                | "toml"
                | "yaml"
                | "yml"
                | "json"
                | "xml"
                | "html"
                | "css"
                | "scss"
                | "sass"
                | "less"
        )
    } else {
        false
    }
}

/// Verifica si un path debe ser ignorado según los patrones
fn should_ignore(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        // Simplificado: verificar si el path contiene el patrón
        let pattern_clean = pattern.replace("**", "").replace("*", "");
        if path.contains(pattern_clean.trim_matches('/')) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file(Path::new("src/main.rs")));
        assert!(is_code_file(Path::new("app.tsx")));
        assert!(is_code_file(Path::new("utils.py")));
        assert!(!is_code_file(Path::new("image.png")));
        assert!(!is_code_file(Path::new("README.md")));
    }

    #[test]
    fn test_should_ignore() {
        let patterns = vec!["node_modules/**".to_string(), "dist/**".to_string()];
        assert!(should_ignore("node_modules/package/index.js", &patterns));
        assert!(should_ignore("dist/bundle.js", &patterns));
        assert!(!should_ignore("src/index.js", &patterns));
    }
}
