use git2::Repository;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime};

use crate::repos;

#[derive(Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub date: i64,
}

#[command]
pub fn get_commit_history<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    limit: usize,
) -> Result<Vec<CommitInfo>, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;

    let mut commits = Vec::new();
    for id in revwalk.take(limit) {
        let id = id.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(id).map_err(|e| e.to_string())?;

        commits.push(CommitInfo {
            hash: commit.id().to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            message: commit.message().unwrap_or("").to_string(),
            date: commit.time().seconds(),
        });
    }

    Ok(commits)
}

#[command]
pub fn commit_changes<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    message: String,
    author_name: String,
    author_email: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    let sig = git2::Signature::now(&author_name, &author_email).map_err(|e| e.to_string())?;

    let parent_commit = repo.head().and_then(|h| h.peel_to_commit()).ok();
    let parents = if let Some(ref p) = parent_commit {
        vec![p]
    } else {
        vec![]
    };

    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
        .map_err(|e| e.to_string())?;

    Ok(())
}
