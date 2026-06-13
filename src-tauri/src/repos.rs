use git2::Repository;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize)]
pub struct RepoInfo {
    pub id: String,
    pub name: String,
    pub branch: String,
}

// this is badly written code this should be named something else
// cause its confusing
#[derive(Serialize, Deserialize)]
pub struct CloneResult {
    pub success: bool,
    pub message: String,
}

#[command]
pub fn list_repos<R: Runtime>(app: AppHandle<R>) -> Vec<RepoInfo> {
    let mut repos = Vec::new();
    let base = get_base_path(&app);

    if !base.exists() {
        return repos;
    }

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(repo) = Repository::open(&path) {
                    let branch = repo
                        .head()
                        .ok()
                        .and_then(|h| h.shorthand().map(|s| s.to_string()))
                        .unwrap_or_else(|| "detached".to_string());
                    repos.push(RepoInfo {
                        id: entry.file_name().to_string_lossy().to_string(),
                        name: entry.file_name().to_string_lossy().to_string(),
                        branch,
                    });
                }
            }
        }
    }

    repos
}

#[command]
pub fn delete_repo<R: Runtime>(app: AppHandle<R>, repo_id: String) -> CloneResult {
    let path = repo_path(&app, &repo_id);
    match fs::remove_dir_all(path) {
        Ok(_) => CloneResult {
            success: true,
            message: "Repo deleted.".to_string(),
        },
        Err(e) => CloneResult {
            success: false,
            message: e.to_string(),
        },
    }
}

pub fn get_base_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to get app data dir")
        .join("repos")
}

pub fn repo_path<R: Runtime>(app: &AppHandle<R>, repo_id: &str) -> PathBuf {
    get_base_path(app).join(repo_id)
}

/// Derives a repo folder name from a clone URL.
/// "https://github.com/user/my-repo.git" → "my-repo"
pub fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_string()
}
