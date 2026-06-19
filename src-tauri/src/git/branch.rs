use git2::{Repository, PushOptions};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime};

use crate::repos;
use crate::git::auth;

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
    }

    Ok(())
}
