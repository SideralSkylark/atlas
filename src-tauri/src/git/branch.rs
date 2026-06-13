use git2::Repository;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime};

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
