use git2::Repository;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime};

use crate::repos;

#[derive(Serialize, Deserialize)]
pub struct StatusEntry {
    pub path: String,
    pub status: String, // "Modified", "New", "Deleted", "Renamed", etc.
    pub staged: bool,
}

#[command]
pub fn get_status<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
) -> Result<Vec<StatusEntry>, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut status_options = git2::StatusOptions::new();
    status_options
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.is_index_new() || status.is_wt_new() {
            "New"
        } else if status.is_index_modified() || status.is_wt_modified() {
            "Modified"
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            "Deleted"
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            "Renamed"
        } else {
            "Other"
        };

        let staged = status.is_index_new()
            || status.is_index_modified()
            || status.is_index_deleted()
            || status.is_index_renamed();

        entries.push(StatusEntry {
            path,
            status: status_str.to_string(),
            staged,
        });
    }

    Ok(entries)
}

#[command]
pub fn stage_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    let full_path = path.join(&filepath);
    if full_path.exists() {
        index
            .add_path(std::path::Path::new(&filepath))
            .map_err(|e| e.to_string())?;
    } else {
        index
            .remove_path(std::path::Path::new(&filepath))
            .map_err(|e| e.to_string())?;
    }
    index.write().map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn unstage_file<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: String,
) -> Result<(), String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let head = repo.head().map_err(|e| e.to_string())?;
    let commit = head.peel_to_commit().map_err(|e| e.to_string())?;

    repo.reset_default(Some(commit.as_object()), &[filepath])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn get_diff<R: Runtime>(
    app: AppHandle<R>,
    repo_id: String,
    filepath: Option<String>,
    staged: bool,
) -> Result<String, String> {
    let path = repos::repo_path(&app, &repo_id);
    let repo = Repository::open(&path).map_err(|e| e.to_string())?;

    let mut diff_options = git2::DiffOptions::new();
    if let Some(ref f) = filepath {
        diff_options.pathspec(f);
    }

    let diff = if staged {
        let head = repo.head().and_then(|h| h.peel_to_tree()).ok();
        repo.diff_tree_to_index(
            head.as_ref(),
            Some(&repo.index().map_err(|e| e.to_string())?),
            Some(&mut diff_options),
        )
    } else {
        repo.diff_index_to_workdir(None, Some(&mut diff_options))
    }
    .map_err(|e| e.to_string())?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let prefix = match line.origin() {
            '+' => "+",
            '-' => "-",
            ' ' => " ",
            _ => "",
        };
        diff_text.push_str(prefix);
        diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true
    })
    .map_err(|e| e.to_string())?;

    Ok(diff_text)
}
