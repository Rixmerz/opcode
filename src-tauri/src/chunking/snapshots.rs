use super::storage::create_snapshot;
use super::types::{Snapshot, SnapshotType};
use anyhow::{Context, Result};
use chrono::Utc;
use git2::{Repository, Signature, IndexAddOption, Oid};
use rusqlite::Connection;
use std::path::Path;

/// Asegura que el proyecto tenga Git inicializado
/// Si no existe .git, lo inicializa y hace un commit inicial
pub fn ensure_git_initialized(project_path: &str) -> Result<Repository> {
    let path = Path::new(project_path);
    let git_path = path.join(".git");

    if git_path.exists() {
        // Ya tiene Git, simplemente abrirlo
        Repository::open(path).context("Failed to open existing Git repository")
    } else {
        // Inicializar nuevo repositorio
        let repo = Repository::init(path).context("Failed to initialize Git repository")?;

        // Crear commit inicial vacío
        let sig = Signature::now("Opcode Agent", "agent@opcode.local")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };

        {
            let tree = repo.find_tree(tree_id)?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "chore: initialize opcode chunking system",
                &tree,
                &[],
            )?;
        }

        println!("[Chunking] Initialized Git repository at: {}", project_path);
        Ok(repo)
    }
}

/// Obtiene el siguiente número de versión master para un proyecto
fn get_next_master_version(conn: &Connection, project_path: &str) -> Result<i32> {
    let max_version: Option<i32> = conn
        .query_row(
            "SELECT MAX(version_major) FROM snapshots WHERE project_path = ?1 AND snapshot_type = 'master'",
            rusqlite::params![project_path],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    Ok(max_version.unwrap_or(0) + 1)
}

/// Obtiene el siguiente número de versión agent para un snapshot master dado
fn get_next_agent_version(conn: &Connection, project_path: &str, master_version: i32) -> Result<i32> {
    let max_minor: Option<i32> = conn
        .query_row(
            "SELECT MAX(version_minor) FROM snapshots WHERE project_path = ?1 AND snapshot_type = 'agent' AND version_major = ?2",
            rusqlite::params![project_path, master_version],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    Ok(max_minor.unwrap_or(0) + 1)
}

/// Obtiene la lista de archivos modificados desde Git
fn get_changed_files_from_repo(repo: &Repository) -> Result<Vec<String>> {
    let mut changed_files = Vec::new();

    // Obtener HEAD
    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;

    // Comparar con working directory
    let diff = repo.diff_tree_to_workdir_with_index(Some(&head_tree), None)?;

    diff.foreach(
        &mut |delta, _progress| {
            if let Some(path) = delta.new_file().path() {
                if let Some(path_str) = path.to_str() {
                    changed_files.push(path_str.to_string());
                }
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(changed_files)
}

/// Crea un snapshot MASTER con commit y tag de Git
/// Versión: V1, V2, V3, etc.
/// Se ejecuta ANTES de enviar un mensaje al agente
pub fn create_master_snapshot_with_git(
    conn: &Connection,
    project_path: &str,
    user_message: &str,
) -> Result<i64> {
    // Asegurar que Git esté inicializado
    let repo = ensure_git_initialized(project_path)?;

    // Obtener la versión siguiente
    let version = get_next_master_version(conn, project_path)?;
    let tag_name = format!("v{}", version);

    // Hacer commit de todos los cambios actuales
    let sig = Signature::now("Opcode User", "user@opcode.local")?;

    // Stage todos los archivos (git add -A)
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Obtener archivos modificados antes del commit
    let changed_files = get_changed_files_from_repo(&repo).unwrap_or_default();

    let commit_message = format!("Master snapshot V{}: {}", version, user_message);

    // Crear commit
    let parent_commit = repo.head()?.peel_to_commit()?;
    let commit_oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &commit_message,
        &tree,
        &[&parent_commit],
    )?;

    // Crear tag
    let commit = repo.find_commit(commit_oid)?;
    repo.tag_lightweight(&tag_name, commit.as_object(), false)?;

    println!(
        "[Chunking] Created master snapshot V{} with commit {} and tag {}",
        version,
        commit_oid,
        tag_name
    );

    // Guardar en la base de datos
    let snapshot = Snapshot {
        id: None,
        project_path: project_path.to_string(),
        snapshot_type: SnapshotType::Master,
        parent_snapshot_id: None,
        message: commit_message.clone(),
        user_message: Some(user_message.to_string()),
        changed_files: serde_json::to_string(&changed_files)?,
        diff_summary: Some(format!("{} files changed", changed_files.len())),
        metadata: None,
        git_commit_hash: Some(commit_oid.to_string()),
        git_tag: Some(tag_name.clone()),
        git_branch: Some("main".to_string()),
        version_major: version,
        version_minor: None,
        created_at: Utc::now(),
    };

    create_snapshot(conn, &snapshot)
}

/// Crea un snapshot AGENT en rama paralela con commit y tag
/// Versión: V{master_version}.{minor} (ej: V1.1, V1.2, V2.1)
/// Se ejecuta DESPUÉS de que el agente completa una ejecución
pub fn create_agent_snapshot_with_git(
    conn: &Connection,
    project_path: &str,
    master_snapshot_id: i64,
    message: &str,
    changed_files_override: Option<Vec<String>>,
) -> Result<i64> {
    // Asegurar que Git esté inicializado
    let repo = ensure_git_initialized(project_path)?;

    // Obtener el snapshot master padre
    let master_snapshot: Snapshot = conn.query_row(
        "SELECT id, project_path, snapshot_type, parent_snapshot_id, message, user_message, changed_files, diff_summary, metadata, git_commit_hash, git_tag, git_branch, version_major, version_minor, created_at
         FROM snapshots WHERE id = ?1",
        rusqlite::params![master_snapshot_id],
        |row| {
            let snapshot_type_str: String = row.get(2)?;
            let created_at_str: String = row.get(14)?;
            Ok(Snapshot {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                snapshot_type: if snapshot_type_str == "master" {
                    SnapshotType::Master
                } else {
                    SnapshotType::Agent
                },
                parent_snapshot_id: row.get(3)?,
                message: row.get(4)?,
                user_message: row.get(5)?,
                changed_files: row.get(6)?,
                diff_summary: row.get(7)?,
                metadata: row.get(8)?,
                git_commit_hash: row.get(9)?,
                git_tag: row.get(10)?,
                git_branch: row.get(11)?,
                version_major: row.get(12)?,
                version_minor: row.get(13)?,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        },
    )?;

    let master_version = master_snapshot.version_major;
    let agent_version = get_next_agent_version(conn, project_path, master_version)?;
    let tag_name = format!("v{}.{}", master_version, agent_version);
    let branch_name = format!("agent/v{}.{}", master_version, agent_version);

    // Obtener el commit hash del master snapshot
    let master_commit_hash = master_snapshot.git_commit_hash
        .context("Master snapshot does not have git_commit_hash")?;
    let master_oid = Oid::from_str(&master_commit_hash)?;
    let master_commit = repo.find_commit(master_oid)?;

    // Crear rama desde el commit master
    repo.branch(&branch_name, &master_commit, false)?;

    // Cambiar a la nueva rama
    let branch_ref = format!("refs/heads/{}", branch_name);
    repo.set_head(&branch_ref)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

    // Stage todos los archivos modificados por el agente
    let sig = Signature::now("Opcode Agent", "agent@opcode.local")?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Obtener archivos modificados
    let changed_files = changed_files_override.unwrap_or_else(|| {
        get_changed_files_from_repo(&repo).unwrap_or_default()
    });

    let commit_message = format!("Agent snapshot V{}.{}: {}", master_version, agent_version, message);

    // Crear commit en la rama agent
    let commit_oid = repo.commit(
        Some(&branch_ref),
        &sig,
        &sig,
        &commit_message,
        &tree,
        &[&master_commit],
    )?;

    // Crear tag
    let commit = repo.find_commit(commit_oid)?;
    repo.tag_lightweight(&tag_name, commit.as_object(), false)?;

    println!(
        "[Chunking] Created agent snapshot V{}.{} on branch {} with commit {} and tag {}",
        master_version,
        agent_version,
        branch_name,
        commit_oid,
        tag_name
    );

    // Volver a la rama main
    repo.set_head("refs/heads/main")?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

    // Guardar en la base de datos
    let snapshot = Snapshot {
        id: None,
        project_path: project_path.to_string(),
        snapshot_type: SnapshotType::Agent,
        parent_snapshot_id: Some(master_snapshot_id),
        message: commit_message.clone(),
        user_message: None,
        changed_files: serde_json::to_string(&changed_files)?,
        diff_summary: Some(format!("{} files changed", changed_files.len())),
        metadata: Some(serde_json::json!({
            "master_version": master_version,
            "agent_version": agent_version,
        }).to_string()),
        git_commit_hash: Some(commit_oid.to_string()),
        git_tag: Some(tag_name.clone()),
        git_branch: Some(branch_name.clone()),
        version_major: master_version,
        version_minor: Some(agent_version),
        created_at: Utc::now(),
    };

    create_snapshot(conn, &snapshot)
}

/// Retrocede la rama master a un snapshot anterior
/// Usa push force para reescribir el historial
/// Elimina snapshots master posteriores de la DB
/// Preserva las ramas agent paralelas
pub fn rewind_master_to_snapshot_with_git(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<()> {
    // Obtener el snapshot
    let snapshot: Snapshot = conn.query_row(
        "SELECT id, project_path, snapshot_type, parent_snapshot_id, message, user_message, changed_files, diff_summary, metadata, git_commit_hash, git_tag, git_branch, version_major, version_minor, created_at
         FROM snapshots WHERE id = ?1",
        rusqlite::params![snapshot_id],
        |row| {
            let snapshot_type_str: String = row.get(2)?;
            let created_at_str: String = row.get(14)?;
            Ok(Snapshot {
                id: Some(row.get(0)?),
                project_path: row.get(1)?,
                snapshot_type: if snapshot_type_str == "master" {
                    SnapshotType::Master
                } else {
                    SnapshotType::Agent
                },
                parent_snapshot_id: row.get(3)?,
                message: row.get(4)?,
                user_message: row.get(5)?,
                changed_files: row.get(6)?,
                diff_summary: row.get(7)?,
                metadata: row.get(8)?,
                git_commit_hash: row.get(9)?,
                git_tag: row.get(10)?,
                git_branch: row.get(11)?,
                version_major: row.get(12)?,
                version_minor: row.get(13)?,
                created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            })
        },
    )?;

    if snapshot.snapshot_type != SnapshotType::Master {
        anyhow::bail!("Can only rewind to master snapshots");
    }

    let commit_hash = snapshot.git_commit_hash
        .context("Snapshot does not have git_commit_hash")?;

    // Abrir repositorio
    let repo = Repository::open(&snapshot.project_path)?;

    // Reset hard al commit del snapshot
    let oid = Oid::from_str(&commit_hash)?;
    let commit = repo.find_commit(oid)?;
    repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;

    println!(
        "[Chunking] Rewinded main branch to snapshot V{} (commit: {})",
        snapshot.version_major,
        commit_hash
    );

    // Eliminar snapshots master posteriores de la DB (version_major > snapshot.version_major)
    conn.execute(
        "DELETE FROM snapshots WHERE project_path = ?1 AND snapshot_type = 'master' AND version_major > ?2",
        rusqlite::params![&snapshot.project_path, snapshot.version_major],
    )?;

    println!(
        "[Chunking] Deleted master snapshots with version > V{}",
        snapshot.version_major
    );

    // Las ramas agent paralelas se preservan automáticamente en Git
    // No se eliminan de la DB ni de Git para mantener historial de lo que se intentó

    Ok(())
}
