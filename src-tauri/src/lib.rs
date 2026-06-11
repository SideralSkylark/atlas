use git2::{build::RepoBuilder, CertificateCheckStatus, FetchOptions, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::command;

#[derive(Serialize, Deserialize)]
pub struct CloneResult {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RepoInfo {
    pub id: String,
    pub name: String,
    pub branch: String,
}

const BASE_PATH: &str = "/data/data/com.skylark.atlas/files/repos";

fn repo_path(repo_id: &str) -> std::path::PathBuf {
    Path::new(BASE_PATH).join(repo_id)
}

/// Derives a repo folder name from a clone URL.
/// "https://github.com/user/my-repo.git" → "my-repo"
fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_string()
}

#[command]
fn clone_repo(url: String) -> CloneResult {
    let name = repo_name_from_url(&url);
    let dest = repo_path(&name);

    if dest.exists() {
        return CloneResult {
            success: false,
            message: format!("'{}' already exists.", name),
        };
    }

    let mut callbacks = RemoteCallbacks::new();
    callbacks.certificate_check(|_, _| Ok(CertificateCheckStatus::CertificateOk));

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    match builder.clone(&url, &dest) {
        Ok(_) => CloneResult {
            success: true,
            message: format!("Cloned '{}'.", name),
        },
        Err(e) => CloneResult {
            success: false,
            message: e.message().to_string(),
        },
    }
}

#[command]
fn delete_repo(repo_id: String) -> CloneResult {
    let path = repo_path(&repo_id);
    match std::fs::remove_dir_all(path) {
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

#[command]
fn list_repos() -> Vec<RepoInfo> {
    let mut repos = Vec::new();
    let base = Path::new(BASE_PATH);

    if !base.exists() {
        return repos;
    }

    if let Ok(entries) = std::fs::read_dir(base) {
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

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

#[command]
fn list_files(repo_id: String, relative_path: String) -> Vec<FileEntry> {
    let mut files = Vec::new();
    let root = repo_path(&repo_id);
    let path = if relative_path.is_empty() {
        root
    } else {
        root.join(&relative_path)
    };

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" {
                continue;
            }
            files.push(FileEntry {
                name,
                is_dir: p.is_dir(),
            });
        }
    }

    files.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    files
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            clone_repo,
            delete_repo,
            list_repos,
            list_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
