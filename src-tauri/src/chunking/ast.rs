use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{AstMetadata, Chunk, ChunkType};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::Connection;
use std::path::Path;
use tree_sitter::{Language, Parser, TreeCursor};

/// Genera chunks de AST comprimido por archivo
pub fn generate_ast_chunks(
    conn: &Connection,
    project_path: &str,
    file_path: &str,
    content: &str,
) -> Result<usize> {
    let language = detect_language(file_path)?;
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .context("Failed to set language")?;

    let tree = parser
        .parse(content, None)
        .context("Failed to parse file")?;

    let root = tree.root_node();

    // Generar representación comprimida del AST
    let mut ast_repr = String::new();
    let mut max_depth = 0;
    let mut node_count = 0;
    let has_syntax_errors = root.has_error();

    serialize_ast_node(&root, &mut ast_repr, 0, &mut max_depth, &mut node_count);

    let content_hash = calculate_content_hash(&ast_repr);

    let metadata = AstMetadata {
        language: get_language_name(&language),
        node_count,
        max_depth,
        has_syntax_errors,
    };

    let chunk = Chunk {
        id: None,
        project_path: project_path.to_string(),
        chunk_type: ChunkType::Ast,
        file_path: Some(file_path.to_string()),
        entity_name: None,
        content: ast_repr,
        content_hash,
        metadata: Some(serde_json::to_string(&metadata)?),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_chunk(conn, &chunk)?;
    Ok(1)
}

/// Serializa un nodo del AST de forma comprimida
fn serialize_ast_node(
    node: &tree_sitter::Node,
    output: &mut String,
    depth: usize,
    max_depth: &mut usize,
    node_count: &mut usize,
) {
    *node_count += 1;
    if depth > *max_depth {
        *max_depth = depth;
    }

    // Formato comprimido: tipo:inicio-fin
    output.push_str(&format!(
        "{}{}:{}-{}",
        "  ".repeat(depth),
        node.kind(),
        node.start_position().row,
        node.end_position().row
    ));

    // Si el nodo tiene un identificador o literal, incluirlo
    if node.child_count() == 0 && node.byte_range().len() < 100 {
        // Solo para nodos hoja pequeños
        output.push_str(&format!(" [{}]", node.kind()));
    }

    output.push('\n');

    // Recursivamente serializar hijos (limitado a profundidad razonable)
    if depth < 50 {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                serialize_ast_node(&child, output, depth + 1, max_depth, node_count);
            }
        }
    }
}

/// Detecta el lenguaje basado en la extensión del archivo
fn detect_language(file_path: &str) -> Result<Language> {
    let path = Path::new(file_path);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .context("No file extension")?;

    match ext {
        "rs" => Ok(tree_sitter_rust::language()),
        "js" | "jsx" | "mjs" | "cjs" => Ok(tree_sitter_javascript::language()),
        "ts" | "tsx" | "mts" | "cts" => {
            Ok(tree_sitter_typescript::language_typescript())
        }
        "py" => Ok(tree_sitter_python::language()),
        _ => Err(anyhow::anyhow!("Unsupported language: {}", ext)),
    }
}

/// Obtiene el nombre del lenguaje
fn get_language_name(language: &Language) -> String {
    // Esto es una simplificación, idealmente debería mantener un mapeo
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert!(detect_language("test.rs").is_ok());
        assert!(detect_language("test.js").is_ok());
        assert!(detect_language("test.ts").is_ok());
        assert!(detect_language("test.py").is_ok());
        assert!(detect_language("test.unknown").is_err());
    }
}
