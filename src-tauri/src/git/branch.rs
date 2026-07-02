use git2::{PushOptions, Repository};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::{command, AppHandle, Runtime};

use crate::git::auth;
use crate::repos;

#[derive(Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

#[command]
pub fn list_branches<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
) -> Result<Vec<BranchInfo>, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let branches = repo.branches(None).map_err(|e| e.to_string())?;

    let mut branch_list = Vec::new();
    for branch_res in branches {
        let (branch, branch_type) = branch_res.map_err(|e| e.to_string())?;
        let name = branch
            .name()
            .map_err(|e| e.to_string())?
            .ok_or("Invalid branch name")?
            .to_string();
        let is_current = branch.is_head();
        let is_remote = branch_type == git2::BranchType::Remote;

        branch_list.push(BranchInfo {
            name,
            is_current,
            is_remote,
        });
    }

    Ok(branch_list)
}

#[command]
pub fn create_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let target = head.peel_to_commit().map_err(|e| e.to_string())?;
    repo.branch(&branch_name, &target, false)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn switch_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let (object, reference) = repo.revparse_ext(&branch_name).map_err(|e| e.to_string())?;

    repo.checkout_tree(
        &object,
        Some(git2::build::CheckoutBuilder::default().force()),
    )
    .map_err(|e| e.to_string())?;

    if let Some(refname) = reference.and_then(|r| r.name().map(|n| n.to_string())) {
        repo.set_head(&refname).map_err(|e| e.to_string())?;
    } else {
        repo.set_head_detached(object.id())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[command]
pub async fn delete_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
    is_remote: bool,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    if is_remote {
        // A remote branch name from git2 is typically "origin/branch-name"
        // We need to parse the remote name and the branch name
        let parts: Vec<&str> = branch_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err("Invalid remote branch name format. Expected 'remote/branch'".to_string());
        }
        let remote_name = parts[0];
        let remote_branch = parts[1];

        // 1. Push ref deletion to the remote server
        let url = {
            let remote = repo.find_remote(remote_name).map_err(|e| e.to_string())?;
            remote.url().unwrap_or("").to_string()
        };
        let callbacks = auth::make_callbacks(&app, &url).await?;

        let mut remote = repo.find_remote(remote_name).map_err(|e| e.to_string())?;
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Refspec for deletion is ":refs/heads/branch_name"
        let refspec = format!(":refs/heads/{}", remote_branch);
        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| e.to_string())?;

        // 2. Also delete the local remote-tracking branch reference
        if let Ok(mut branch) = repo.find_branch(&branch_name, git2::BranchType::Remote) {
            branch.delete().map_err(|e| e.to_string())?;
        }
    } else {
        let mut branch = repo
            .find_branch(&branch_name, git2::BranchType::Local)
            .map_err(|e| e.to_string())?;
        if branch.is_head() {
            return Err("Cannot delete the current checked-out branch".to_string());
        }
        branch.delete().map_err(|e| e.to_string())?;

        // Delete branch config section from config file to allow recreation
        if let Ok(mut config) = repo.config() {
            let prefix = format!("branch.{}.", branch_name);
            let mut keys_to_remove: Vec<String> = Vec::new();
            if let Ok(mut entries) = config.entries(Some(&prefix)) {
                while let Some(Ok(entry)) = entries.next() {
                    if let Some(name) = entry.name() {
                        keys_to_remove.push(name.to_string());
                    }
                }
            }
            for key in keys_to_remove {
                let _ = config.remove(&key);
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub message: String,
}

#[command]
pub async fn merge_branch<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    branch_name: String,
) -> Result<MergeResult, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    // Find the target branch reference
    let ref_obj = repo
        .find_branch(&branch_name, git2::BranchType::Local)
        .or_else(|_| repo.find_branch(&branch_name, git2::BranchType::Remote))
        .map_err(|e| e.to_string())?;

    let reference = ref_obj.get();
    let annotated = repo
        .reference_to_annotated_commit(reference)
        .map_err(|e| e.to_string())?;

    let (analysis, _) = repo
        .merge_analysis(&[&annotated])
        .map_err(|e| e.to_string())?;

    if analysis.is_up_to_date() {
        return Ok(MergeResult {
            success: true,
            message: "Already up to date".to_string(),
        });
    }

    if analysis.is_fast_forward() {
        // Fast-forward merge
        let mut head_ref = repo.head().map_err(|e| e.to_string())?;
        let target_commit = repo
            .find_commit(annotated.id())
            .map_err(|e| e.to_string())?;

        repo.checkout_tree(
            target_commit.as_object(),
            Some(git2::build::CheckoutBuilder::default().force()),
        )
        .map_err(|e| e.to_string())?;

        head_ref
            .set_target(
                annotated.id(),
                &format!("Fast-forward merge: {}", branch_name),
            )
            .map_err(|e| e.to_string())?;

        repo.set_head(head_ref.name().unwrap())
            .map_err(|e| e.to_string())?;

        return Ok(MergeResult {
            success: true,
            message: "Merge successful (Fast-forward)".to_string(),
        });
    }

    Err("Unsupported merge type".to_string())
}

#[command]
pub fn get_conflicts<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
) -> Result<Vec<String>, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let index = repo.index().map_err(|e| e.to_string())?;

    let mut conflicted_files = Vec::new();
    if index.has_conflicts() {
        let conflicts = index.conflicts().map_err(|e| e.to_string())?;
        for conflict_res in conflicts {
            let conflict = conflict_res.map_err(|e| e.to_string())?;
            let match_path = conflict
                .our
                .as_ref()
                .map(|o| &o.path)
                .or_else(|| conflict.their.as_ref().map(|t| &t.path))
                .or_else(|| conflict.ancestor.as_ref().map(|a| &a.path));
            if let Some(p) = match_path {
                conflicted_files.push(String::from_utf8_lossy(p).to_string());
            }
        }
    }
    conflicted_files.sort();
    conflicted_files.dedup();
    Ok(conflicted_files)
}

#[command]
pub fn resolve_conflict<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: String,
    choice: String, // "ours", "theirs", or "merged"
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    // Phase 1: scan conflicts (immutable borrow on `index` via IndexConflicts).
    // Collect everything we need so the borrow is fully released before we mutate.
    let mut found = false;
    let mut blob_id_to_write: Option<git2::Oid> = None;

    {
        let conflicts = index.conflicts().map_err(|e| e.to_string())?;
        for conflict_res in conflicts {
            let conflict = conflict_res.map_err(|e| e.to_string())?;

            let match_path = conflict
                .our
                .as_ref()
                .map(|o| &o.path)
                .or_else(|| conflict.their.as_ref().map(|t| &t.path))
                .or_else(|| conflict.ancestor.as_ref().map(|a| &a.path));

            if let Some(p) = match_path {
                if String::from_utf8_lossy(p) == filepath {
                    found = true;
                    if choice == "ours" || choice == "theirs" {
                        let entry_opt = if choice == "ours" {
                            conflict.our
                        } else {
                            conflict.their
                        };
                        blob_id_to_write = entry_opt.map(|e| e.id);
                    }
                    break;
                }
            }
        }
    } // `conflicts` (and its immutable borrow) is dropped here

    if !found {
        return Err("Conflict not found for the specified file".to_string());
    }

    // Phase 2: write blob to disk (if ours/theirs choice) — no index borrow needed yet.
    if let Some(blob_id) = blob_id_to_write {
        let blob = repo.find_blob(blob_id).map_err(|e| e.to_string())?;
        let full_path = path.join(&filepath);
        fs::write(&full_path, blob.content()).map_err(|e| e.to_string())?;
    }

    // Phase 3: stage the file to resolve the conflict in the index.
    index
        .add_path(Path::new(&filepath))
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;
    Ok(())
}
