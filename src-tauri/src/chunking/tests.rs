use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{Chunk, ChunkType};
use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rusqlite::Connection;

/// Genera chunks de tests por archivo
pub fn generate_test_chunks(
    conn: &Connection,
    project_path: &str,
    file_path: &str,
    content: &str,
) -> Result<usize> {
    // Detectar si es un archivo de tests
    if !is_test_file(file_path, content) {
        return Ok(0);
    }

    // Extraer información de tests
    let test_functions = extract_test_functions(content, file_path);
    let expectations = extract_expectations(content);

    // Crear representación del chunk de tests
    let mut test_repr = String::new();
    test_repr.push_str(&format!("# Test File: {}\n", file_path));
    test_repr.push_str(&format!("# Test Functions: {}\n\n", test_functions.len()));

    for (idx, test_func) in test_functions.iter().enumerate() {
        test_repr.push_str(&format!("{}. {}\n", idx + 1, test_func));
    }

    test_repr.push_str(&format!("\n# Expectations: {}\n", expectations.len()));
    for exp in &expectations {
        test_repr.push_str(&format!("- {}\n", exp));
    }

    let content_hash = calculate_content_hash(&test_repr);

    let chunk = Chunk {
        id: None,
        project_path: project_path.to_string(),
        chunk_type: ChunkType::Tests,
        file_path: Some(file_path.to_string()),
        entity_name: None,
        content: test_repr,
        content_hash,
        metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_chunk(conn, &chunk)?;
    Ok(1)
}

/// Detecta si un archivo es un archivo de tests
fn is_test_file(file_path: &str, content: &str) -> bool {
    // Por nombre de archivo
    if file_path.contains("test") || file_path.contains("spec") || file_path.ends_with("_test.rs") {
        return true;
    }

    // Por contenido (buscar keywords de testing)
    content.contains("#[test]")
        || content.contains("describe(")
        || content.contains("it(")
        || content.contains("test(")
        || content.contains("def test_")
        || content.contains("class Test")
}

/// Extrae nombres de funciones de test
fn extract_test_functions(content: &str, file_path: &str) -> Vec<String> {
    let mut tests = Vec::new();

    if file_path.ends_with(".rs") {
        // Rust: #[test] fn test_name()
        let re = Regex::new(r"#\[test\]\s*fn\s+([a-zA-Z0-9_]+)").unwrap();
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                tests.push(name.as_str().to_string());
            }
        }
    } else if file_path.ends_with(".js") || file_path.ends_with(".ts") {
        // JS/TS: it('test name') o test('test name')
        let re = Regex::new(r#"(?:it|test)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                tests.push(name.as_str().to_string());
            }
        }
    } else if file_path.ends_with(".py") {
        // Python: def test_name()
        let re = Regex::new(r"def\s+(test_[a-zA-Z0-9_]+)").unwrap();
        for cap in re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                tests.push(name.as_str().to_string());
            }
        }
    }

    tests
}

/// Extrae expectations/assertions de los tests
fn extract_expectations(content: &str) -> Vec<String> {
    let mut expectations = Vec::new();

    // Patrones comunes de assertions
    let patterns = [
        r"assert[_!]?\s*\(",
        r"expect\s*\(",
        r"\.to[A-Z][a-zA-Z]*\(",
        r"should\.",
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            for mat in re.find_iter(content) {
                // Extraer contexto alrededor del assertion
                let start = mat.start().saturating_sub(20);
                let end = (mat.end() + 50).min(content.len());
                let context = &content[start..end];
                expectations.push(context.replace('\n', " ").trim().to_string());
            }
        }
    }

    expectations
}
