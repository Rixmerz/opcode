use super::storage::{calculate_content_hash, insert_relationship, upsert_chunk};
use super::types::{CallgraphMetadata, Chunk, ChunkRelationship, ChunkType, RelationshipType};
use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

/// Genera chunks de callgraph estático por análisis de imports/requires
pub fn generate_callgraph_chunks(
    conn: &Connection,
    project_path: &str,
    file_path: &str,
    content: &str,
) -> Result<usize> {
    let language = detect_language_by_extension(file_path);

    // Extraer imports/requires según el lenguaje
    let dependencies = extract_dependencies(content, &language);
    let function_calls = extract_function_calls(content, &language);

    // Crear metadata
    let metadata = CallgraphMetadata {
        is_static: true,
        entry_points: vec![],
        external_calls: dependencies.clone(),
        call_count: function_calls.len(),
    };

    // Serializar el callgraph
    let mut callgraph_repr = String::new();
    callgraph_repr.push_str(&format!("# Dependencies ({})\n", dependencies.len()));
    for dep in &dependencies {
        callgraph_repr.push_str(&format!("import: {}\n", dep));
    }

    callgraph_repr.push_str(&format!("\n# Function Calls ({})\n", function_calls.len()));
    for call in &function_calls {
        callgraph_repr.push_str(&format!("call: {}\n", call));
    }

    let content_hash = calculate_content_hash(&callgraph_repr);

    let chunk = Chunk {
        id: None,
        project_path: project_path.to_string(),
        chunk_type: ChunkType::Callgraph,
        file_path: Some(file_path.to_string()),
        entity_name: None,
        content: callgraph_repr,
        content_hash,
        metadata: Some(serde_json::to_string(&metadata)?),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_chunk(conn, &chunk)?;
    Ok(1)
}

/// Detecta el lenguaje por extensión de archivo
fn detect_language_by_extension(file_path: &str) -> String {
    if file_path.ends_with(".rs") {
        "rust".to_string()
    } else if file_path.ends_with(".js") || file_path.ends_with(".jsx") {
        "javascript".to_string()
    } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        "typescript".to_string()
    } else if file_path.ends_with(".py") {
        "python".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Extrae dependencias (imports) del código
fn extract_dependencies(content: &str, language: &str) -> Vec<String> {
    let mut deps = HashSet::new();

    match language {
        "rust" => {
            // use statements
            let re = Regex::new(r"use\s+([a-zA-Z0-9_:]+)").unwrap();
            for cap in re.captures_iter(content) {
                if let Some(dep) = cap.get(1) {
                    deps.insert(dep.as_str().to_string());
                }
            }
        }
        "javascript" | "typescript" => {
            // import statements
            let re_import = Regex::new(r#"import\s+.*?from\s+['"]([^'"]+)['"]"#).unwrap();
            for cap in re_import.captures_iter(content) {
                if let Some(dep) = cap.get(1) {
                    deps.insert(dep.as_str().to_string());
                }
            }

            // require statements
            let re_require = Regex::new(r#"require\(['"]([^'"]+)['"]\)"#).unwrap();
            for cap in re_require.captures_iter(content) {
                if let Some(dep) = cap.get(1) {
                    deps.insert(dep.as_str().to_string());
                }
            }
        }
        "python" => {
            // import statements
            let re_import = Regex::new(r"import\s+([a-zA-Z0-9_.]+)").unwrap();
            for cap in re_import.captures_iter(content) {
                if let Some(dep) = cap.get(1) {
                    deps.insert(dep.as_str().to_string());
                }
            }

            // from import
            let re_from = Regex::new(r"from\s+([a-zA-Z0-9_.]+)\s+import").unwrap();
            for cap in re_from.captures_iter(content) {
                if let Some(dep) = cap.get(1) {
                    deps.insert(dep.as_str().to_string());
                }
            }
        }
        _ => {}
    }

    deps.into_iter().collect()
}

/// Extrae llamadas a funciones del código
fn extract_function_calls(content: &str, language: &str) -> Vec<String> {
    let mut calls = HashSet::new();

    // Pattern genérico para llamadas a función
    let re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    for cap in re.captures_iter(content) {
        if let Some(func) = cap.get(1) {
            let func_name = func.as_str();
            // Filtrar keywords comunes
            if !is_keyword(func_name, language) {
                calls.insert(func_name.to_string());
            }
        }
    }

    calls.into_iter().collect()
}

/// Verifica si una palabra es una keyword del lenguaje
fn is_keyword(word: &str, language: &str) -> bool {
    match language {
        "rust" => matches!(
            word,
            "if" | "else" | "while" | "for" | "loop" | "match" | "return" | "break" | "continue"
        ),
        "javascript" | "typescript" => matches!(
            word,
            "if" | "else"
                | "while"
                | "for"
                | "switch"
                | "case"
                | "return"
                | "break"
                | "continue"
                | "function"
                | "class"
        ),
        "python" => matches!(
            word,
            "if" | "elif"
                | "else"
                | "while"
                | "for"
                | "return"
                | "break"
                | "continue"
                | "def"
                | "class"
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_dependencies_rust() {
        let code = "use std::collections::HashMap;\nuse serde::Serialize;";
        let deps = extract_dependencies(code, "rust");
        assert!(deps.contains(&"std::collections::HashMap".to_string()));
        assert!(deps.contains(&"serde::Serialize".to_string()));
    }

    #[test]
    fn test_extract_dependencies_js() {
        let code = r#"import React from 'react';\nconst fs = require('fs');"#;
        let deps = extract_dependencies(code, "javascript");
        assert!(deps.contains(&"react".to_string()));
        assert!(deps.contains(&"fs".to_string()));
    }

    #[test]
    fn test_extract_function_calls() {
        let code = "console.log('test');\nconst result = calculate(10);";
        let calls = extract_function_calls(code, "javascript");
        assert!(calls.contains(&"log".to_string()));
        assert!(calls.contains(&"calculate".to_string()));
    }
}
