use super::storage::{get_business_rules, upsert_business_rule};
use super::types::BusinessRule;
use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

/// Crea una regla de negocio propuesta (pendiente de validación)
pub fn propose_business_rule(
    conn: &Connection,
    project_path: &str,
    entity_name: &str,
    file_path: &str,
    ai_interpretation: &str,
) -> Result<i64> {
    let rule = BusinessRule {
        id: None,
        project_path: project_path.to_string(),
        entity_name: entity_name.to_string(),
        file_path: file_path.to_string(),
        rule_description: String::new(), // Se llenará con la validación del usuario
        ai_interpretation: ai_interpretation.to_string(),
        user_correction: None,
        is_validated: false,
        validation_date: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    upsert_business_rule(conn, &rule)
}

/// Valida una regla de negocio con corrección del usuario
pub fn validate_business_rule(
    conn: &Connection,
    rule_id: i64,
    rule_description: &str,
    user_correction: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE business_rules SET rule_description = ?1, user_correction = ?2, is_validated = 1, validation_date = ?3, updated_at = ?4 WHERE id = ?5",
        rusqlite::params![
            rule_description,
            user_correction,
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339(),
            rule_id
        ],
    )?;
    Ok(())
}

/// Obtiene reglas de negocio pendientes de validación
pub fn get_pending_rules(conn: &Connection, project_path: &str) -> Result<Vec<BusinessRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_path, entity_name, file_path, rule_description, ai_interpretation, user_correction, is_validated, validation_date, created_at, updated_at
         FROM business_rules WHERE project_path = ?1 AND is_validated = 0 ORDER BY created_at",
    )?;

    let rules = stmt
        .query_map(rusqlite::params![project_path], |row| {
            let created_at_str: String = row.get(9)?;
            let updated_at_str: String = row.get(10)?;
            let validation_date_str: Option<String> = row.get(8)?;

            Ok(BusinessRule {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                entity_name: row.get(2)?,
                file_path: row.get(3)?,
                rule_description: row.get(4)?,
                ai_interpretation: row.get(5)?,
                user_correction: row.get(6)?,
                is_validated: row.get(7)?,
                validation_date: validation_date_str.and_then(|s| s.parse().ok()),
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rules)
}
