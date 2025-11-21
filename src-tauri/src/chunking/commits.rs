use super::storage::{calculate_content_hash, upsert_chunk};
use super::types::{Chunk, ChunkType, CommitMetadata};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Time};
use rusqlite::Connection;

/// Genera chunks de commit history
pub fn generate_commit_chunks(
    conn: &Connection,
    project_path: &str,
    max_commits: Option<usize>,
) -> Result<usize> {
    let repo = Repository::open(project_path).context("Failed to open git repository")?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut chunks_created = 0;
    let limit = max_commits.unwrap_or(100);

    for (idx, oid) in revwalk.enumerate() {
        if idx >= limit {
            break;
        }

        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author();
        let time = commit.time();

        // Obtener archivos modificados
        let mut files_modified = Vec::new();
        let tree = commit.tree()?;

        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        files_modified.push(path.to_string_lossy().to_string());
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }

        // Crear representación del commit
        let mut commit_repr = String::new();
        commit_repr.push_str(&format!("Commit: {}\n", oid));
        commit_repr.push_str(&format!("Author: {} <{}>\n", author.name().unwrap_or(""), author.email().unwrap_or("")));
        commit_repr.push_str(&format!("Date: {}\n\n", time_to_datetime(time)));
        commit_repr.push_str(&format!("Message:\n{}\n\n", message));
        commit_repr.push_str(&format!("Files Modified ({}):\n", files_modified.len()));
        for file in &files_modified {
            commit_repr.push_str(&format!("  - {}\n", file));
        }

        let content_hash = calculate_content_hash(&commit_repr);

        let metadata = CommitMetadata {
            commit_hash: oid.to_string(),
            author: author.name().unwrap_or("").to_string(),
            author_email: author.email().unwrap_or("").to_string(),
            commit_date: time_to_datetime(time),
            files_modified: files_modified.clone(),
            insertions: 0, // Podríamos calcular esto con diff stats
            deletions: 0,
        };

        let chunk = Chunk {
            id: None,
            project_path: project_path.to_string(),
            chunk_type: ChunkType::CommitHistory,
            file_path: None,
            entity_name: Some(oid.to_string()),
            content: commit_repr,
            content_hash,
            metadata: Some(serde_json::to_string(&metadata)?),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        upsert_chunk(conn, &chunk, None)?;
        chunks_created += 1;
    }

    Ok(chunks_created)
}

/// Convierte git2::Time a DateTime<Utc>
fn time_to_datetime(time: Time) -> DateTime<Utc> {
    DateTime::from_timestamp(time.seconds(), 0).unwrap_or_else(Utc::now)
}
